use log::{error, info};
use std::fs;
use std::path::Path;
use tauri::State;

use database_structure_sync_lib::diff::{compare_schemas, compare_schemas_cross};
use database_structure_sync_lib::models::{Connection, ConnectionInput, DiffResult};

use crate::driver::{AppState, DatabaseDriver, create_driver, load_connection};

#[tauri::command]
pub(crate) async fn list_connections(
    state: State<'_, AppState>,
) -> Result<Vec<Connection>, String> {
    info!("Listing all connections");
    let store = state.config_store.lock().await;
    store.list_connections().await.map_err(|e| {
        error!("Failed to list connections: {}", e);
        e.to_string()
    })
}

#[tauri::command]
pub(crate) async fn get_connection(
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
pub(crate) async fn save_connection(
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
pub(crate) async fn update_connection(
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
pub(crate) async fn delete_connection(
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    info!("Deleting connection: {}", id);
    let store = state.config_store.lock().await;
    store.delete_connection(&id).await.map_err(|e| {
        error!("Failed to delete connection {}: {}", id, e);
        e.to_string()
    })
}

#[tauri::command]
pub(crate) async fn test_connection(
    state: State<'_, AppState>,
    input: ConnectionInput,
) -> Result<(), String> {
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
pub(crate) async fn list_databases(
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
pub(crate) async fn compare_databases(
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
    let items = if source_conn.db_type == target_conn.db_type {
        compare_schemas(
            &source_tables,
            &target_tables,
            target_driver.as_sql_generator(),
        )
    } else {
        let source_mapper = source_driver.as_type_mapper(&source_conn.db_type);
        let target_mapper = target_driver.as_type_mapper(&target_conn.db_type);
        compare_schemas_cross(
            &source_tables,
            &target_tables,
            target_driver.as_sql_generator(),
            source_mapper.as_ref(),
            target_mapper.as_ref(),
        )
    };

    info!("Comparison complete: {} differences found", items.len());

    Ok(DiffResult {
        items,
        source_tables: source_tables.len(),
        target_tables: target_tables.len(),
    })
}

#[tauri::command]
pub(crate) async fn execute_sync(
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
pub(crate) async fn save_sql_file(file_path: String, content: String) -> Result<(), String> {
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
