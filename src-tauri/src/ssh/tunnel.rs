use anyhow::{Result, anyhow};
use russh::client;
use russh_keys::key::PublicKey;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::Mutex;

use crate::models::{SshAuthMethod, SshConfig};

struct Client;

#[async_trait::async_trait]
impl client::Handler for Client {
    type Error = anyhow::Error;

    async fn check_server_key(
        &mut self,
        _server_public_key: &PublicKey,
    ) -> Result<bool, Self::Error> {
        // In production, you should verify the host key
        Ok(true)
    }
}

pub struct SshTunnel {
    local_port: u16,
    _handle: tokio::task::JoinHandle<()>,
}

impl SshTunnel {
    pub async fn new(ssh_config: &SshConfig, remote_host: &str, remote_port: u16) -> Result<Self> {
        let config = Arc::new(client::Config::default());
        let sh = Client;

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
