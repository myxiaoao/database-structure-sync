use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DbType {
    MySQL,
    PostgreSQL,
    MariaDB,
}

impl DbType {
    pub fn default_port(&self) -> u16 {
        match self {
            DbType::MySQL | DbType::MariaDB => 3306,
            DbType::PostgreSQL => 5432,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SshAuthMethod {
    Password { password: String },
    PrivateKey {
        private_key_path: String,
        passphrase: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshConfig {
    pub enabled: bool,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub auth_method: SshAuthMethod,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SslConfig {
    pub enabled: bool,
    pub ca_cert_path: Option<String>,
    pub client_cert_path: Option<String>,
    pub client_key_path: Option<String>,
    pub verify_server: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connection {
    pub id: String,
    pub name: String,
    pub db_type: DbType,
    pub host: String,
    pub port: u16,
    pub username: String,
    #[serde(skip_serializing)]
    pub password: String,
    pub database: String,
    pub ssh_config: Option<SshConfig>,
    pub ssl_config: Option<SslConfig>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionInput {
    pub name: String,
    pub db_type: DbType,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database: String,
    pub ssh_config: Option<SshConfig>,
    pub ssl_config: Option<SslConfig>,
}
