// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use log::{error, info};
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tauri::{Manager, State};
use tokio::sync::Mutex;

use database_structure_sync_lib::db::{MySqlDriver, PostgresDriver, SchemaReader, SqlGenerator};
use database_structure_sync_lib::diff::compare_schemas;
use database_structure_sync_lib::error::{AppError, AppResult};
use database_structure_sync_lib::models::{Connection, ConnectionInput, DbType, DiffResult};
use database_structure_sync_lib::ssh::SshTunnel;
use database_structure_sync_lib::storage::ConfigStore;

pub struct AppState {
    pub config_store: Arc<Mutex<ConfigStore>>,
    pub active_tunnels: Arc<Mutex<Vec<SshTunnel>>>,
}

/// Resolve connection host and port, applying SSH tunnel if configured.
/// When SSH is enabled, creates a local tunnel and returns `("127.0.0.1", local_port)`.
async fn resolve_connection_endpoint(
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
enum DatabaseDriver {
    MySql(MySqlDriver),
    Postgres(PostgresDriver),
}

impl DatabaseDriver {
    async fn create(conn: &Connection, tunnels: &Arc<Mutex<Vec<SshTunnel>>>) -> AppResult<Self> {
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

    fn as_reader(&self) -> &dyn SchemaReader {
        match self {
            DatabaseDriver::MySql(d) => d,
            DatabaseDriver::Postgres(d) => d,
        }
    }

    fn as_sql_generator(&self) -> &dyn SqlGenerator {
        match self {
            DatabaseDriver::MySql(d) => d,
            DatabaseDriver::Postgres(d) => d,
        }
    }

    async fn execute_sql(&self, sql: &str) -> Result<(), sqlx::Error> {
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
async fn load_connection(store: &ConfigStore, id: &str, label: &str) -> Result<Connection, String> {
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
async fn create_driver(
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

#[tauri::command]
async fn list_connections(state: State<'_, AppState>) -> Result<Vec<Connection>, String> {
    info!("Listing all connections");
    let store = state.config_store.lock().await;
    store.list_connections().await.map_err(|e| {
        error!("Failed to list connections: {}", e);
        e.to_string()
    })
}

#[tauri::command]
async fn get_connection(
    state: State<'_, AppState>,
    id: String,
) -> Result<Option<Connection>, String> {
    info!("Getting connection: {}", id);
    let store = state.config_store.lock().await;
    store.get_connection(&id).await.map_err(|e| {
        error!("Failed to get connection {}: {}", id, e);
        e.to_string()
    })
}

#[tauri::command]
async fn save_connection(
    state: State<'_, AppState>,
    input: ConnectionInput,
) -> Result<Connection, String> {
    info!("Saving connection: {}", input.name);
    let store = state.config_store.lock().await;
    store.save_connection(input).await.map_err(|e| {
        error!("Failed to save connection: {}", e);
        e.to_string()
    })
}

#[tauri::command]
async fn update_connection(
    state: State<'_, AppState>,
    id: String,
    input: ConnectionInput,
) -> Result<Connection, String> {
    info!("Updating connection: {} ({})", id, input.name);
    let store = state.config_store.lock().await;
    store.update_connection(&id, input).await.map_err(|e| {
        error!("Failed to update connection {}: {}", id, e);
        e.to_string()
    })
}

#[tauri::command]
async fn delete_connection(state: State<'_, AppState>, id: String) -> Result<(), String> {
    info!("Deleting connection: {}", id);
    let store = state.config_store.lock().await;
    store.delete_connection(&id).await.map_err(|e| {
        error!("Failed to delete connection {}: {}", id, e);
        e.to_string()
    })
}

#[tauri::command]
async fn test_connection(state: State<'_, AppState>, input: ConnectionInput) -> Result<(), String> {
    info!("Testing connection: {} ({})", input.name, input.host);

    let temp_conn = Connection {
        id: String::new(),
        name: input.name.clone(),
        db_type: input.db_type,
        host: input.host,
        port: input.port,
        username: input.username,
        password: input.password,
        database: input.database,
        ssh_config: input.ssh_config,
        ssl_config: input.ssl_config,
        created_at: String::new(),
        updated_at: String::new(),
    };

    let driver = DatabaseDriver::create(&temp_conn, &state.active_tunnels)
        .await
        .map_err(|e| {
            error!("Failed to create driver for test: {}", e);
            e.to_string()
        })?;

    driver.as_reader().test_connection().await.map_err(|e| {
        error!("Connection test failed: {}", e);
        e.to_string()
    })?;

    info!("Connection test successful: {}", input.name);
    Ok(())
}

#[tauri::command]
async fn list_databases(
    state: State<'_, AppState>,
    connection_id: String,
) -> Result<Vec<String>, String> {
    info!("Listing databases for connection: {}", connection_id);

    let store = state.config_store.lock().await;
    let mut conn = load_connection(&store, &connection_id, "Connection").await?;
    drop(store);

    let driver = create_driver(&mut conn, None, &state.active_tunnels).await?;
    let databases = driver.as_reader().list_databases().await.map_err(|e| {
        error!("Failed to list databases: {}", e);
        e.to_string()
    })?;

    info!("Found {} databases", databases.len());
    Ok(databases)
}

#[tauri::command]
async fn compare_databases(
    state: State<'_, AppState>,
    source_id: String,
    target_id: String,
    source_database: Option<String>,
    target_database: Option<String>,
) -> Result<DiffResult, String> {
    info!("Comparing databases: {} -> {}", source_id, target_id);

    let store = state.config_store.lock().await;
    let mut source_conn = load_connection(&store, &source_id, "Source connection").await?;
    let mut target_conn = load_connection(&store, &target_id, "Target connection").await?;
    drop(store);

    info!(
        "Connecting to source: {} ({})",
        source_conn.name, source_conn.db_type
    );
    let source_driver =
        create_driver(&mut source_conn, source_database, &state.active_tunnels).await?;

    info!(
        "Connecting to target: {} ({})",
        target_conn.name, target_conn.db_type
    );
    let target_driver =
        create_driver(&mut target_conn, target_database, &state.active_tunnels).await?;

    info!("Fetching source schema...");
    let source_tables = source_driver.as_reader().get_tables().await.map_err(|e| {
        error!("Failed to get source tables: {}", e);
        e.to_string()
    })?;

    info!("Fetching target schema...");
    let target_tables = target_driver.as_reader().get_tables().await.map_err(|e| {
        error!("Failed to get target tables: {}", e);
        e.to_string()
    })?;

    info!(
        "Comparing schemas: {} source tables, {} target tables",
        source_tables.len(),
        target_tables.len()
    );
    let items = compare_schemas(
        &source_tables,
        &target_tables,
        target_driver.as_sql_generator(),
    );

    info!("Comparison complete: {} differences found", items.len());

    Ok(DiffResult {
        items,
        source_tables: source_tables.len(),
        target_tables: target_tables.len(),
    })
}

#[tauri::command]
async fn execute_sync(
    state: State<'_, AppState>,
    target_id: String,
    sql_statements: Vec<String>,
    target_database: Option<String>,
) -> Result<(), String> {
    info!(
        "Executing sync on target {}: {} statements",
        target_id,
        sql_statements.len()
    );

    let store = state.config_store.lock().await;
    let mut target_conn = load_connection(&store, &target_id, "Target connection").await?;
    drop(store);

    let driver = create_driver(&mut target_conn, target_database, &state.active_tunnels).await?;

    for (i, sql) in sql_statements.iter().enumerate() {
        info!("Executing statement {}/{}", i + 1, sql_statements.len());
        driver.execute_sql(sql).await.map_err(|e| {
            error!("Failed to execute SQL: {}\nError: {}", sql, e);
            format!("Failed to execute: {}\nError: {}", sql, e)
        })?;
    }

    info!("Sync execution completed successfully");
    Ok(())
}

#[tauri::command]
async fn save_sql_file(file_path: String, content: String) -> Result<(), String> {
    info!("Saving SQL file to: {}", file_path);

    let path = Path::new(&file_path);

    // Create parent directories if they don't exist
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            error!("Failed to create directory: {}", e);
            e.to_string()
        })?;
    }

    fs::write(path, content).map_err(|e| {
        error!("Failed to write SQL file: {}", e);
        e.to_string()
    })?;

    info!("SQL file saved successfully");
    Ok(())
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(
            tauri_plugin_log::Builder::new()
                .target(tauri_plugin_log::Target::new(
                    tauri_plugin_log::TargetKind::LogDir {
                        file_name: Some("app".into()),
                    },
                ))
                .target(tauri_plugin_log::Target::new(
                    tauri_plugin_log::TargetKind::Stdout,
                ))
                .level(log::LevelFilter::Info)
                .build(),
        )
        .setup(|app| {
            info!("Starting Database Structure Sync application");

            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("Failed to get app data dir");

            info!("App data directory: {:?}", app_data_dir);

            tauri::async_runtime::block_on(async {
                let config_store = ConfigStore::new(app_data_dir)
                    .await
                    .expect("Failed to initialize config store");

                app.manage(AppState {
                    config_store: Arc::new(Mutex::new(config_store)),
                    active_tunnels: Arc::new(Mutex::new(Vec::new())),
                });
            });

            info!("Application setup complete");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_connections,
            get_connection,
            save_connection,
            update_connection,
            delete_connection,
            test_connection,
            list_databases,
            compare_databases,
            execute_sync,
            save_sql_file
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
