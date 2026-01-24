// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Arc;
use tokio::sync::Mutex;
use tauri::{Manager, State};

use database_structure_sync_lib::db::{MySqlDriver, PostgresDriver, SchemaReader, SqlGenerator};
use database_structure_sync_lib::diff::compare_schemas;
use database_structure_sync_lib::models::{Connection, ConnectionInput, DbType, DiffResult};
use database_structure_sync_lib::storage::ConfigStore;

pub struct AppState {
    pub config_store: Arc<Mutex<ConfigStore>>,
}

#[tauri::command]
async fn list_connections(state: State<'_, AppState>) -> Result<Vec<Connection>, String> {
    let store = state.config_store.lock().await;
    store.list_connections().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_connection(state: State<'_, AppState>, id: String) -> Result<Option<Connection>, String> {
    let store = state.config_store.lock().await;
    store.get_connection(&id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn save_connection(state: State<'_, AppState>, input: ConnectionInput) -> Result<Connection, String> {
    let store = state.config_store.lock().await;
    store.save_connection(input).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn delete_connection(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let store = state.config_store.lock().await;
    store.delete_connection(&id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn test_connection(input: ConnectionInput) -> Result<(), String> {
    let reader: Box<dyn SchemaReader> = match input.db_type {
        DbType::MySQL | DbType::MariaDB => {
            Box::new(MySqlDriver::new(&input.host, input.port, &input.username, &input.password, &input.database).await.map_err(|e| e.to_string())?)
        }
        DbType::PostgreSQL => {
            Box::new(PostgresDriver::new(&input.host, input.port, &input.username, &input.password, &input.database).await.map_err(|e| e.to_string())?)
        }
    };

    reader.test_connection().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn compare_databases(
    state: State<'_, AppState>,
    source_id: String,
    target_id: String,
) -> Result<DiffResult, String> {
    let store = state.config_store.lock().await;

    let source_conn = store.get_connection(&source_id).await.map_err(|e| e.to_string())?
        .ok_or("Source connection not found")?;
    let target_conn = store.get_connection(&target_id).await.map_err(|e| e.to_string())?
        .ok_or("Target connection not found")?;

    drop(store);

    let source_reader: Box<dyn SchemaReader> = match source_conn.db_type {
        DbType::MySQL | DbType::MariaDB => {
            Box::new(MySqlDriver::new(&source_conn.host, source_conn.port, &source_conn.username, &source_conn.password, &source_conn.database).await.map_err(|e| e.to_string())?)
        }
        DbType::PostgreSQL => {
            Box::new(PostgresDriver::new(&source_conn.host, source_conn.port, &source_conn.username, &source_conn.password, &source_conn.database).await.map_err(|e| e.to_string())?)
        }
    };

    let target_reader: Box<dyn SchemaReader> = match target_conn.db_type {
        DbType::MySQL | DbType::MariaDB => {
            Box::new(MySqlDriver::new(&target_conn.host, target_conn.port, &target_conn.username, &target_conn.password, &target_conn.database).await.map_err(|e| e.to_string())?)
        }
        DbType::PostgreSQL => {
            Box::new(PostgresDriver::new(&target_conn.host, target_conn.port, &target_conn.username, &target_conn.password, &target_conn.database).await.map_err(|e| e.to_string())?)
        }
    };

    let source_tables = source_reader.get_tables().await.map_err(|e| e.to_string())?;
    let target_tables = target_reader.get_tables().await.map_err(|e| e.to_string())?;

    let sql_gen: Box<dyn SqlGenerator> = match target_conn.db_type {
        DbType::MySQL | DbType::MariaDB => {
            Box::new(MySqlDriver::new(&target_conn.host, target_conn.port, &target_conn.username, &target_conn.password, &target_conn.database).await.map_err(|e| e.to_string())?)
        }
        DbType::PostgreSQL => {
            Box::new(PostgresDriver::new(&target_conn.host, target_conn.port, &target_conn.username, &target_conn.password, &target_conn.database).await.map_err(|e| e.to_string())?)
        }
    };

    let items = compare_schemas(&source_tables, &target_tables, sql_gen.as_ref());

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
    let store = state.config_store.lock().await;
    let target_conn = store.get_connection(&target_id).await.map_err(|e| e.to_string())?
        .ok_or("Target connection not found")?;
    drop(store);

    match target_conn.db_type {
        DbType::MySQL | DbType::MariaDB => {
            let driver = MySqlDriver::new(&target_conn.host, target_conn.port, &target_conn.username, &target_conn.password, &target_conn.database).await.map_err(|e| e.to_string())?;
            for sql in sql_statements {
                sqlx::query(&sql).execute(driver.pool()).await.map_err(|e| format!("Failed to execute: {}\nError: {}", sql, e))?;
            }
        }
        DbType::PostgreSQL => {
            let driver = PostgresDriver::new(&target_conn.host, target_conn.port, &target_conn.username, &target_conn.password, &target_conn.database).await.map_err(|e| e.to_string())?;
            for sql in sql_statements {
                sqlx::query(&sql).execute(driver.pool()).await.map_err(|e| format!("Failed to execute: {}\nError: {}", sql, e))?;
            }
        }
    }

    Ok(())
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir().expect("Failed to get app data dir");

            tauri::async_runtime::block_on(async {
                let config_store = ConfigStore::new(app_data_dir).await.expect("Failed to initialize config store");
                app.manage(AppState {
                    config_store: Arc::new(Mutex::new(config_store)),
                });
            });

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
