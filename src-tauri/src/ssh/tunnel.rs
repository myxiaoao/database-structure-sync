use anyhow::{Result, anyhow};
use log::{info, warn};
use russh::client;
use russh_keys::key::PublicKey;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::Mutex;

use crate::models::{SshAuthMethod, SshConfig};

/// Get the path to the user's known_hosts file.
fn known_hosts_path() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".ssh").join("known_hosts"))
}

/// Check if a host key matches an entry in known_hosts.
/// Returns Ok(true) if matched, Ok(false) if host not found, Err if key mismatch.
fn check_known_hosts(host: &str, port: u16, key: &PublicKey) -> Result<bool> {
    let path = match known_hosts_path() {
        Some(p) if p.exists() => p,
        _ => return Ok(false), // No known_hosts file, host unknown
    };

    let contents = std::fs::read_to_string(&path)?;
    let host_pattern = if port == 22 {
        host.to_string()
    } else {
        format!("[{}]:{}", host, port)
    };

    for line in contents.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let parts: Vec<&str> = line.splitn(3, ' ').collect();
        if parts.len() < 3 {
            continue;
        }

        let hosts_field = parts[0];
        let key_type = parts[1];
        let key_data = parts[2];

        // Check if this line matches our host
        let host_matches = hosts_field
            .split(',')
            .any(|h| h.trim() == host_pattern || h.trim() == host);

        if !host_matches {
            continue;
        }

        // Host found — verify the key matches
        let stored_key_str = format!("{} {}", key_type, key_data);
        let current_key_str = format_public_key(key);

        if stored_key_str == current_key_str {
            return Ok(true); // Key matches
        } else {
            return Err(anyhow!(
                "SSH host key mismatch for {}! \
                 The server's key has changed, which could indicate a MITM attack. \
                 If you trust this server, remove the old entry from ~/.ssh/known_hosts and retry.",
                host_pattern
            ));
        }
    }

    Ok(false) // Host not found
}

/// Append a host key to known_hosts (TOFU — Trust On First Use).
fn save_to_known_hosts(host: &str, port: u16, key: &PublicKey) {
    let path = match known_hosts_path() {
        Some(p) => p,
        None => return,
    };

    // Ensure .ssh directory exists
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let host_pattern = if port == 22 {
        host.to_string()
    } else {
        format!("[{}]:{}", host, port)
    };

    let key_str = format_public_key(key);
    let line = format!("{} {}\n", host_pattern, key_str);

    match std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
    {
        Ok(mut file) => {
            let _ = file.write_all(line.as_bytes());
            info!("Added SSH host key for {} to known_hosts", host_pattern);
        }
        Err(e) => {
            warn!("Failed to save SSH host key to known_hosts: {}", e);
        }
    }
}

/// Format a public key as "type base64data" for known_hosts comparison/storage.
fn format_public_key(key: &PublicKey) -> String {
    match key {
        PublicKey::Ed25519(k) => {
            format!(
                "ssh-ed25519 {}",
                data_encoding::BASE64.encode(&encode_ed25519_pubkey(k.as_bytes()))
            )
        }
        _ => {
            // Fallback: use debug representation for comparison
            // This covers RSA, ECDSA, etc.
            format!("{:?}", key)
        }
    }
}

/// Encode an Ed25519 public key in SSH wire format (type string + key bytes).
fn encode_ed25519_pubkey(raw_bytes: &[u8]) -> Vec<u8> {
    let key_type = b"ssh-ed25519";
    let mut buf = Vec::new();
    buf.extend_from_slice(&(key_type.len() as u32).to_be_bytes());
    buf.extend_from_slice(key_type);
    buf.extend_from_slice(&(raw_bytes.len() as u32).to_be_bytes());
    buf.extend_from_slice(raw_bytes);
    buf
}

struct SshClient {
    host: String,
    port: u16,
}

#[async_trait::async_trait]
impl client::Handler for SshClient {
    type Error = anyhow::Error;

    async fn check_server_key(
        &mut self,
        server_public_key: &PublicKey,
    ) -> Result<bool, Self::Error> {
        match check_known_hosts(&self.host, self.port, server_public_key) {
            Ok(true) => {
                info!("SSH host key verified for {}:{}", self.host, self.port);
                Ok(true)
            }
            Ok(false) => {
                // TOFU: first connection, save and accept
                warn!(
                    "SSH host {}:{} not in known_hosts, accepting key (TOFU)",
                    self.host, self.port
                );
                save_to_known_hosts(&self.host, self.port, server_public_key);
                Ok(true)
            }
            Err(e) => {
                // Key mismatch — reject
                Err(e)
            }
        }
    }
}

pub struct SshTunnel {
    local_port: u16,
    _handle: tokio::task::JoinHandle<()>,
}

impl SshTunnel {
    pub async fn new(ssh_config: &SshConfig, remote_host: &str, remote_port: u16) -> Result<Self> {
        let config = Arc::new(client::Config::default());
        let sh = SshClient {
            host: ssh_config.host.clone(),
            port: ssh_config.port,
        };

        let addr = format!("{}:{}", ssh_config.host, ssh_config.port);
        let mut session = client::connect(config, addr, sh).await?;

        let authenticated = match &ssh_config.auth_method {
            SshAuthMethod::Password { password } => {
                session
                    .authenticate_password(&ssh_config.username, password)
                    .await?
            }
            SshAuthMethod::PrivateKey {
                private_key_path,
                passphrase,
            } => {
                let key_path = shellexpand::tilde(private_key_path).to_string();
                let key_pair = russh_keys::load_secret_key(&key_path, passphrase.as_deref())?;
                session
                    .authenticate_publickey(&ssh_config.username, Arc::new(key_pair))
                    .await?
            }
        };

        if !authenticated {
            return Err(anyhow!("SSH authentication failed"));
        }

        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let local_port = listener.local_addr()?.port();

        let remote_host = remote_host.to_string();
        let session = Arc::new(Mutex::new(session));

        let handle = tokio::spawn(async move {
            loop {
                if let Ok((mut local_stream, _)) = listener.accept().await {
                    let session = session.clone();
                    let remote_host = remote_host.clone();

                    tokio::spawn(async move {
                        let channel = {
                            let session = session.lock().await;
                            match session
                                .channel_open_direct_tcpip(
                                    &remote_host,
                                    remote_port as u32,
                                    "127.0.0.1",
                                    local_port as u32,
                                )
                                .await
                            {
                                Ok(c) => c,
                                Err(_) => return,
                            }
                        };

                        let mut channel_stream = channel.into_stream();
                        let mut buf1 = vec![0u8; 8192];
                        let mut buf2 = vec![0u8; 8192];

                        loop {
                            tokio::select! {
                                result = local_stream.read(&mut buf1) => {
                                    match result {
                                        Ok(0) | Err(_) => break,
                                        Ok(n) => {
                                            if channel_stream.write_all(&buf1[..n]).await.is_err() {
                                                break;
                                            }
                                        }
                                    }
                                }
                                result = channel_stream.read(&mut buf2) => {
                                    match result {
                                        Ok(0) | Err(_) => break,
                                        Ok(n) => {
                                            if local_stream.write_all(&buf2[..n]).await.is_err() {
                                                break;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    });
                }
            }
        });

        Ok(Self {
            local_port,
            _handle: handle,
        })
    }

    pub fn local_port(&self) -> u16 {
        self.local_port
    }
}
