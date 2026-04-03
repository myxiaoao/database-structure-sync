use log::{error, info};
use std::sync::Arc;
use tokio::sync::Mutex;

use database_structure_sync_lib::db::{MySqlDriver, PostgresDriver, SchemaReader, SqlGenerator};
use database_structure_sync_lib::error::{AppError, AppResult};
use database_structure_sync_lib::models::{Connection, DbType};
use database_structure_sync_lib::ssh::SshTunnel;
use database_structure_sync_lib::storage::ConfigStore;
use database_structure_sync_lib::types::{
    MariaDbTypeMapper, MySqlTypeMapper, PostgresTypeMapper, TypeMapper,
};

pub struct AppState {
    pub config_store: Arc<Mutex<ConfigStore>>,
    pub active_tunnels: Arc<Mutex<Vec<SshTunnel>>>,
}

/// Resolve connection host and port, applying SSH tunnel if configured.
/// When SSH is enabled, creates a local tunnel and returns `("127.0.0.1", local_port)`.
pub(crate) async fn resolve_connection_endpoint(
    conn: &Connection,
    tunnels: &Arc<Mutex<Vec<SshTunnel>>>,
) -> Result<(String, u16), AppError> {
    if let Some(ssh) = &conn.ssh_config {
        if ssh.enabled {
            info!("Creating SSH tunnel for connection: {}", conn.name);
            let tunnel = SshTunnel::new(ssh, &conn.host, conn.port)
                .await
                .map_err(|e| AppError::Connection(format!("SSH tunnel failed: {}", e)))?;
            let local_port = tunnel.local_port();
            tunnels.lock().await.push(tunnel);
            info!("SSH tunnel established on 127.0.0.1:{}", local_port);
            return Ok(("127.0.0.1".to_string(), local_port));
        }
    }
    Ok((conn.host.clone(), conn.port))
}

/// Database driver that implements both SchemaReader and SqlGenerator
pub(crate) enum DatabaseDriver {
    MySql(MySqlDriver),
    Postgres(PostgresDriver),
}

impl DatabaseDriver {
    pub(crate) async fn create(
        conn: &Connection,
        tunnels: &Arc<Mutex<Vec<SshTunnel>>>,
    ) -> AppResult<Self> {
        let (host, port) = resolve_connection_endpoint(conn, tunnels).await?;
        let ssl_config = conn.ssl_config.as_ref();

        match conn.db_type {
            DbType::MySQL | DbType::MariaDB => {
                info!("Creating MySQL/MariaDB driver for: {}", conn.name);
                let driver = MySqlDriver::new_with_ssl(
                    &host,
                    port,
                    &conn.username,
                    &conn.password,
                    &conn.database,
                    ssl_config,
                )
                .await
                .map_err(|e| AppError::Connection(e.to_string()))?;
                Ok(DatabaseDriver::MySql(driver))
            }
            DbType::PostgreSQL => {
                info!("Creating PostgreSQL driver for: {}", conn.name);
                let driver = PostgresDriver::new_with_ssl(
                    &host,
                    port,
                    &conn.username,
                    &conn.password,
                    &conn.database,
                    ssl_config,
                )
                .await
                .map_err(|e| AppError::Connection(e.to_string()))?;
                Ok(DatabaseDriver::Postgres(driver))
            }
        }
    }

    pub(crate) fn as_reader(&self) -> &dyn SchemaReader {
        match self {
            DatabaseDriver::MySql(d) => d,
            DatabaseDriver::Postgres(d) => d,
        }
    }

    pub(crate) fn as_sql_generator(&self) -> &dyn SqlGenerator {
        match self {
            DatabaseDriver::MySql(d) => d,
            DatabaseDriver::Postgres(d) => d,
        }
    }

    pub(crate) fn as_type_mapper(&self, actual_db_type: &DbType) -> Box<dyn TypeMapper> {
        match actual_db_type {
            DbType::MySQL => Box::new(MySqlTypeMapper),
            DbType::MariaDB => Box::new(MariaDbTypeMapper),
            DbType::PostgreSQL => Box::new(PostgresTypeMapper),
        }
    }

    pub(crate) async fn execute_sql(&self, sql: &str) -> Result<(), sqlx::Error> {
        for stmt in sql.split(';') {
            let stmt = stmt.trim();
            if stmt.is_empty() {
                continue;
            }
            let full = format!("{};", stmt);
            match self {
                DatabaseDriver::MySql(d) => {
                    sqlx::query(&full).execute(d.pool()).await?;
                }
                DatabaseDriver::Postgres(d) => {
                    sqlx::query(&full).execute(d.pool()).await?;
                }
            }
        }
        Ok(())
    }
}

/// Load a connection by ID from the store, returning a descriptive error if not found.
pub(crate) async fn load_connection(
    store: &ConfigStore,
    id: &str,
    label: &str,
) -> Result<Connection, String> {
    store
        .get_connection(id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| {
            error!("{} not found: {}", label, id);
            format!("{} not found", label)
        })
}

/// Create a DatabaseDriver for a connection, optionally overriding the database name.
pub(crate) async fn create_driver(
    conn: &mut Connection,
    database_override: Option<String>,
    tunnels: &Arc<Mutex<Vec<SshTunnel>>>,
) -> Result<DatabaseDriver, String> {
    if let Some(db) = database_override {
        conn.database = db;
    }
    DatabaseDriver::create(conn, tunnels).await.map_err(|e| {
        error!("Failed to connect ({}): {}", conn.name, e);
        e.to_string()
    })
}
