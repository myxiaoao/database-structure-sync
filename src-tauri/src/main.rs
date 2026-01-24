// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use log::{error, info, warn};
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

/// Create a database driver based on connection configuration
async fn create_reader(conn: &Connection) -> AppResult<Box<dyn SchemaReader>> {
    let (host, port) = if let Some(ssh) = &conn.ssh_config {
        if ssh.enabled {
            info!("Creating SSH tunnel for connection: {}", conn.name);
            // When SSH is enabled, we'd connect through the tunnel
            // For now, use direct connection (tunnel implementation pending)
            warn!("SSH tunnel not yet fully implemented, using direct connection");
            (conn.host.clone(), conn.port)
        } else {
            (conn.host.clone(), conn.port)
        }
    } else {
        (conn.host.clone(), conn.port)
    };

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
            Ok(Box::new(driver))
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
            Ok(Box::new(driver))
        }
    }
}

/// Create a SQL generator based on connection configuration
async fn create_sql_generator(conn: &Connection) -> AppResult<Box<dyn SqlGenerator>> {
    let ssl_config = conn.ssl_config.as_ref();

    match conn.db_type {
        DbType::MySQL | DbType::MariaDB => {
            let driver = MySqlDriver::new_with_ssl(
                &conn.host,
                conn.port,
                &conn.username,
                &conn.password,
                &conn.database,
                ssl_config,
            )
            .await
            .map_err(|e| AppError::Connection(e.to_string()))?;
            Ok(Box::new(driver))
        }
        DbType::PostgreSQL => {
            let driver = PostgresDriver::new_with_ssl(
                &conn.host,
                conn.port,
                &conn.username,
                &conn.password,
                &conn.database,
                ssl_config,
            )
            .await
            .map_err(|e| AppError::Connection(e.to_string()))?;
            Ok(Box::new(driver))
        }
    }
}

#[tauri::command]
async fn list_connections(state: State<'_, AppState>) -> Result<Vec<Connection>, String> {
    info!("Listing all connections");
    let store = state.config_store.lock().await;
    store
        .list_connections()
        .await
        .map_err(|e| {
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
    store
        .get_connection(&id)
        .await
        .map_err(|e| {
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
    store
        .save_connection(input)
        .await
        .map_err(|e| {
            error!("Failed to save connection: {}", e);
            e.to_string()
        })
}

#[tauri::command]
async fn delete_connection(state: State<'_, AppState>, id: String) -> Result<(), String> {
    info!("Deleting connection: {}", id);
    let store = state.config_store.lock().await;
    store
        .delete_connection(&id)
        .await
        .map_err(|e| {
            error!("Failed to delete connection {}: {}", id, e);
            e.to_string()
        })
}

#[tauri::command]
async fn test_connection(input: ConnectionInput) -> Result<(), String> {
    info!("Testing connection: {} ({})", input.name, input.host);

    // Create a temporary connection object for the factory
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

    let reader = create_reader(&temp_conn).await.map_err(|e| {
        error!("Failed to create driver for test: {}", e);
        e.to_string()
    })?;

    reader.test_connection().await.map_err(|e| {
        error!("Connection test failed: {}", e);
        e.to_string()
    })?;

    info!("Connection test successful: {}", input.name);
    Ok(())
}

#[tauri::command]
async fn compare_databases(
    state: State<'_, AppState>,
    source_id: String,
    target_id: String,
) -> Result<DiffResult, String> {
    info!("Comparing databases: {} -> {}", source_id, target_id);

    let store = state.config_store.lock().await;

    let source_conn = store
        .get_connection(&source_id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| {
            error!("Source connection not found: {}", source_id);
            "Source connection not found".to_string()
        })?;

    let target_conn = store
        .get_connection(&target_id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| {
            error!("Target connection not found: {}", target_id);
            "Target connection not found".to_string()
        })?;

    drop(store);

    info!(
        "Connecting to source: {} ({})",
        source_conn.name, source_conn.db_type
    );
    let source_reader = create_reader(&source_conn).await.map_err(|e| {
        error!("Failed to connect to source: {}", e);
        e.to_string()
    })?;

    info!(
        "Connecting to target: {} ({})",
        target_conn.name, target_conn.db_type
    );
    let target_reader = create_reader(&target_conn).await.map_err(|e| {
        error!("Failed to connect to target: {}", e);
        e.to_string()
    })?;

    info!("Fetching source schema...");
    let source_tables = source_reader.get_tables().await.map_err(|e| {
        error!("Failed to get source tables: {}", e);
        e.to_string()
    })?;

    info!("Fetching target schema...");
    let target_tables = target_reader.get_tables().await.map_err(|e| {
        error!("Failed to get target tables: {}", e);
        e.to_string()
    })?;

    let sql_gen = create_sql_generator(&target_conn).await.map_err(|e| {
        error!("Failed to create SQL generator: {}", e);
        e.to_string()
    })?;

    info!(
        "Comparing schemas: {} source tables, {} target tables",
        source_tables.len(),
        target_tables.len()
    );
    let items = compare_schemas(&source_tables, &target_tables, sql_gen.as_ref());

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
) -> Result<(), String> {
    info!(
        "Executing sync on target {}: {} statements",
        target_id,
        sql_statements.len()
    );

    let store = state.config_store.lock().await;
    let target_conn = store
        .get_connection(&target_id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| {
            error!("Target connection not found: {}", target_id);
            "Target connection not found".to_string()
        })?;
    drop(store);

    let ssl_config = target_conn.ssl_config.as_ref();

    match target_conn.db_type {
        DbType::MySQL | DbType::MariaDB => {
            let driver = MySqlDriver::new_with_ssl(
                &target_conn.host,
                target_conn.port,
                &target_conn.username,
                &target_conn.password,
                &target_conn.database,
                ssl_config,
            )
            .await
            .map_err(|e| {
                error!("Failed to connect to target: {}", e);
                e.to_string()
            })?;

            for (i, sql) in sql_statements.iter().enumerate() {
                info!("Executing statement {}/{}", i + 1, sql_statements.len());
                sqlx::query(sql).execute(driver.pool()).await.map_err(|e| {
                    error!("Failed to execute SQL: {}\nError: {}", sql, e);
                    format!("Failed to execute: {}\nError: {}", sql, e)
                })?;
            }
        }
        DbType::PostgreSQL => {
            let driver = PostgresDriver::new_with_ssl(
                &target_conn.host,
                target_conn.port,
                &target_conn.username,
                &target_conn.password,
                &target_conn.database,
                ssl_config,
            )
            .await
            .map_err(|e| {
                error!("Failed to connect to target: {}", e);
                e.to_string()
            })?;

            for (i, sql) in sql_statements.iter().enumerate() {
                info!("Executing statement {}/{}", i + 1, sql_statements.len());
                sqlx::query(sql).execute(driver.pool()).await.map_err(|e| {
                    error!("Failed to execute SQL: {}\nError: {}", sql, e);
                    format!("Failed to execute: {}\nError: {}", sql, e)
                })?;
            }
        }
    }

    info!("Sync execution completed successfully");
    Ok(())
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
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
            delete_connection,
            test_connection,
            compare_databases,
            execute_sync
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
