pub mod commands;
pub mod domain;
pub mod downloader;
pub mod errors;
pub mod events;
pub mod media;
pub mod persistence;
pub mod providers;
pub mod recovery;
pub mod scheduler;

use std::sync::Arc;
use tauri::Manager;
use crate::commands::AppState;
use crate::events::AppEventEmitter;
use crate::persistence::Database;
use crate::providers::ProviderRegistry;
use crate::recovery::RecoveryManager;
use crate::scheduler::Scheduler;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // Determine database path in app data directory
            let app_data_dir = app
                .path()
                .app_data_dir()
                .unwrap_or_else(|_| std::env::current_dir().unwrap_or_else(|_| ".".into()));

            let db_path = app_data_dir.join("bvd_storage.db");
            tracing::info!("Initializing SQLite database at {:?}", db_path);

            let db = Arc::new(Database::new(&db_path).expect("Failed to initialize SQLite database"));
            let emitter = AppEventEmitter::new(app.handle().clone());
            let scheduler = Arc::new(Scheduler::new(db.clone(), emitter.clone()));
            let provider_registry = Arc::new(ProviderRegistry::new());
            let recovery_manager = Arc::new(RecoveryManager::new(db.clone()));

            // Run startup recovery automatically
            match recovery_manager.run_startup_recovery() {
                Ok(summary) => {
                    tracing::info!("Startup recovery complete: {:?}", summary);
                    emitter.emit(
                        "recovery.completed",
                        "system",
                        serde_json::to_value(&summary).unwrap_or_default(),
                    );
                }
                Err(e) => {
                    tracing::error!("Startup recovery error: {:?}", e);
                }
            }

            // Start background download scheduler so queued downloads process automatically
            scheduler.start();

            // Set window title bar icon explicitly to our custom brand logo
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_icon(tauri::include_image!("icons/icon.png"));
            }

            let app_state = AppState {
                db,
                scheduler,
                provider_registry,
                recovery_manager,
                events: emitter,
            };

            app.manage(app_state);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::preview_import,
            commands::commit_import,
            commands::create_batch,
            commands::list_batches,
            commands::get_batch,
            commands::update_batch,
            commands::start_batch,
            commands::pause_batch,
            commands::stop_batch,
            commands::delete_batch,
            commands::delete_batches,
            commands::clear_all_batches,
            commands::start_queue,
            commands::pause_queue,
            commands::list_downloads,
            commands::get_download,
            commands::pause_download,
            commands::resume_download,
            commands::cancel_download,
            commands::retry_download,
            commands::retry_failed_downloads,
            commands::skip_download,
            commands::remove_download,
            commands::change_download_priority,
            commands::open_download_folder,
            commands::get_settings,
            commands::update_settings,
            commands::list_profiles,
            commands::create_profile,
            commands::update_profile,
            commands::delete_profile,
            commands::set_default_profile,
            commands::get_recovery_summary,
            commands::check_ffmpeg,
            commands::install_ffmpeg,
            commands::check_ytdlp,
            commands::install_ytdlp,
            commands::check_storage,
            commands::get_system_status,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
