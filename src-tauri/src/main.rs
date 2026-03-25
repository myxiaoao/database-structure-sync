// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod driver;

use log::info;
use std::sync::Arc;
use tauri::Manager;
use tokio::sync::Mutex;

use database_structure_sync_lib::storage::ConfigStore;
use driver::AppState;

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
            commands::list_connections,
            commands::get_connection,
            commands::save_connection,
            commands::update_connection,
            commands::delete_connection,
            commands::test_connection,
            commands::list_databases,
            commands::compare_databases,
            commands::execute_sync,
            commands::save_sql_file
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
