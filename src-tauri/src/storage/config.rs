use anyhow::Result;
use sqlx::{Pool, Sqlite, sqlite::SqlitePoolOptions};
use std::path::PathBuf;

use crate::models::{Connection, ConnectionInput, DbType, SshAuthMethod, SshConfig, SslConfig};
use crate::storage::crypto;

pub struct ConfigStore {
    pool: Pool<Sqlite>,
}

impl ConfigStore {
    pub async fn new(app_data_dir: PathBuf) -> Result<Self> {
        std::fs::create_dir_all(&app_data_dir)?;
        let db_path = app_data_dir.join("config.db");
        let db_url = format!("sqlite:{}?mode=rwc", db_path.display());

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&db_url)
            .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS connections (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                db_type TEXT NOT NULL,
                host TEXT NOT NULL,
                port INTEGER NOT NULL,
                username TEXT NOT NULL,
                database_name TEXT NOT NULL,
                ssh_enabled INTEGER DEFAULT 0,
                ssh_host TEXT,
                ssh_port INTEGER,
                ssh_username TEXT,
                ssh_auth_method TEXT,
                ssh_private_key_path TEXT,
                ssl_enabled INTEGER DEFAULT 0,
                ssl_ca_cert_path TEXT,
                ssl_client_cert_path TEXT,
                ssl_client_key_path TEXT,
                ssl_verify_server INTEGER DEFAULT 1,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )
            "#,
        )
        .execute(&pool)
        .await?;

        Ok(Self { pool })
    }

    fn fetch_connection_passwords(row: &ConnectionRow) -> (String, Option<String>, Option<String>) {
        let password = crypto::get_password(&row.id).unwrap_or_default();
        let ssh_password =
            if row.ssh_enabled == 1 && row.ssh_auth_method.as_deref() == Some("password") {
                crypto::get_password(&format!("{}_ssh", row.id)).ok()
            } else {
                None
            };
        let ssh_passphrase =
            if row.ssh_enabled == 1 && row.ssh_auth_method.as_deref() == Some("privatekey") {
                crypto::get_password(&format!("{}_ssh_passphrase", row.id)).ok()
            } else {
                None
            };
        (password, ssh_password, ssh_passphrase)
    }

    pub async fn list_connections(&self) -> Result<Vec<Connection>> {
        let rows = sqlx::query_as::<_, ConnectionRow>("SELECT * FROM connections ORDER BY name")
            .fetch_all(&self.pool)
            .await?;

        let connections = rows
            .into_iter()
            .map(|row| {
                let (password, ssh_password, ssh_passphrase) =
                    Self::fetch_connection_passwords(&row);
                row.into_connection(password, ssh_password, ssh_passphrase)
            })
            .collect();

        Ok(connections)
    }

    pub async fn get_connection(&self, id: &str) -> Result<Option<Connection>> {
        let row = sqlx::query_as::<_, ConnectionRow>("SELECT * FROM connections WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.map(|row| {
            let (password, ssh_password, ssh_passphrase) = Self::fetch_connection_passwords(&row);
            row.into_connection(password, ssh_password, ssh_passphrase)
        }))
    }

    pub async fn save_connection(&self, input: ConnectionInput) -> Result<Connection> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        let db_type_str = match input.db_type {
            DbType::MySQL => "mysql",
            DbType::PostgreSQL => "postgresql",
            DbType::MariaDB => "mariadb",
        };

        let (ssh_enabled, ssh_host, ssh_port, ssh_username, ssh_auth_method, ssh_private_key_path) =
            match &input.ssh_config {
                Some(ssh) if ssh.enabled => {
                    let (method, key_path) = match &ssh.auth_method {
                        SshAuthMethod::Password { password } => {
                            crypto::store_password(&format!("{}_ssh", id), password)?;
                            ("password".to_string(), None)
                        }
                        SshAuthMethod::PrivateKey {
                            private_key_path,
                            passphrase,
                        } => {
                            if let Some(pp) = passphrase {
                                crypto::store_password(&format!("{}_ssh_passphrase", id), pp)?;
                            }
                            ("privatekey".to_string(), Some(private_key_path.clone()))
                        }
                    };
                    (
                        1,
                        Some(ssh.host.clone()),
                        Some(ssh.port as i32),
                        Some(ssh.username.clone()),
                        Some(method),
                        key_path,
                    )
                }
                _ => (0, None, None, None, None, None),
            };

        let (ssl_enabled, ssl_ca, ssl_cert, ssl_key, ssl_verify) = match &input.ssl_config {
            Some(ssl) if ssl.enabled => (
                1,
                ssl.ca_cert_path.clone(),
                ssl.client_cert_path.clone(),
                ssl.client_key_path.clone(),
                if ssl.verify_server { 1 } else { 0 },
            ),
            _ => (0, None, None, None, 1),
        };

        crypto::store_password(&id, &input.password)?;

        sqlx::query(
            r#"
            INSERT INTO connections (
                id, name, db_type, host, port, username, database_name,
                ssh_enabled, ssh_host, ssh_port, ssh_username, ssh_auth_method, ssh_private_key_path,
                ssl_enabled, ssl_ca_cert_path, ssl_client_cert_path, ssl_client_key_path, ssl_verify_server,
                created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&id)
        .bind(&input.name)
        .bind(db_type_str)
        .bind(&input.host)
        .bind(input.port as i32)
        .bind(&input.username)
        .bind(&input.database)
        .bind(ssh_enabled)
        .bind(&ssh_host)
        .bind(ssh_port)
        .bind(&ssh_username)
        .bind(&ssh_auth_method)
        .bind(&ssh_private_key_path)
        .bind(ssl_enabled)
        .bind(&ssl_ca)
        .bind(&ssl_cert)
        .bind(&ssl_key)
        .bind(ssl_verify)
        .bind(&now)
        .bind(&now)
        .execute(&self.pool)
        .await?;

        Ok(Connection {
            id,
            name: input.name,
            db_type: input.db_type,
            host: input.host,
            port: input.port,
            username: input.username,
            password: input.password,
            database: input.database,
            ssh_config: input.ssh_config,
            ssl_config: input.ssl_config,
            created_at: now.clone(),
            updated_at: now,
        })
    }

    pub async fn delete_connection(&self, id: &str) -> Result<()> {
        let _ = crypto::delete_password(id);
        let _ = crypto::delete_password(&format!("{}_ssh", id));
        let _ = crypto::delete_password(&format!("{}_ssh_passphrase", id));

        sqlx::query("DELETE FROM connections WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}

#[derive(sqlx::FromRow)]
struct ConnectionRow {
    id: String,
    name: String,
    db_type: String,
    host: String,
    port: i32,
    username: String,
    database_name: String,
    ssh_enabled: i32,
    ssh_host: Option<String>,
    ssh_port: Option<i32>,
    ssh_username: Option<String>,
    ssh_auth_method: Option<String>,
    ssh_private_key_path: Option<String>,
    ssl_enabled: i32,
    ssl_ca_cert_path: Option<String>,
    ssl_client_cert_path: Option<String>,
    ssl_client_key_path: Option<String>,
    ssl_verify_server: i32,
    created_at: String,
    updated_at: String,
}

impl ConnectionRow {
    fn into_connection(
        self,
        password: String,
        ssh_password: Option<String>,
        ssh_passphrase: Option<String>,
    ) -> Connection {
        let db_type = match self.db_type.as_str() {
            "mysql" => DbType::MySQL,
            "postgresql" => DbType::PostgreSQL,
            "mariadb" => DbType::MariaDB,
            _ => DbType::MySQL,
        };

        let ssh_config = if self.ssh_enabled == 1 {
            let auth_method = match self.ssh_auth_method.as_deref() {
                Some("password") => SshAuthMethod::Password {
                    password: ssh_password.unwrap_or_default(),
                },
                Some("privatekey") => SshAuthMethod::PrivateKey {
                    private_key_path: self.ssh_private_key_path.unwrap_or_default(),
                    passphrase: ssh_passphrase,
                },
                _ => SshAuthMethod::Password {
                    password: String::new(),
                },
            };
            Some(SshConfig {
                enabled: true,
                host: self.ssh_host.unwrap_or_default(),
                port: self.ssh_port.unwrap_or(22) as u16,
                username: self.ssh_username.unwrap_or_default(),
                auth_method,
            })
        } else {
            None
        };

        let ssl_config = if self.ssl_enabled == 1 {
            Some(SslConfig {
                enabled: true,
                ca_cert_path: self.ssl_ca_cert_path,
                client_cert_path: self.ssl_client_cert_path,
                client_key_path: self.ssl_client_key_path,
                verify_server: self.ssl_verify_server == 1,
            })
        } else {
            None
        };

        Connection {
            id: self.id,
            name: self.name,
            db_type,
            host: self.host,
            port: self.port as u16,
            username: self.username,
            password,
            database: self.database_name,
            ssh_config,
            ssl_config,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}
