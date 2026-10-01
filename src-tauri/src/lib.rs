pub mod commands;
pub use ocmd_core as core;

use commands::AppState;
use core::diagnostics::DiagnosticsBuffer;
use core::download_manager::DownloadManager;
use core::persistence::JobStore;
use core::settings::SettingsManager;
use core::tools::ToolResolver;
use std::sync::Arc;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let tool_resolver = Arc::new(ToolResolver::new());
    let diagnostics = Arc::new(DiagnosticsBuffer::new());
    let settings = Arc::new(SettingsManager::new());
    let database_path = dirs::data_local_dir()
        .unwrap_or_else(std::env::temp_dir)
        .join("openDownloader")
        .join("downloads.sqlite3");
    let job_store = Arc::new(
        JobStore::open(&database_path)
            .unwrap_or_else(|error| panic!("Cannot open download history database: {error}")),
    );
    job_store
        .recover_interrupted()
        .unwrap_or_else(|error| panic!("Cannot recover download history: {error}"));
    let download_manager = Arc::new(
        DownloadManager::new(tool_resolver.clone(), diagnostics.clone(), settings.clone())
            .with_job_store(job_store),
    );

    let app_state = AppState {
        tool_resolver,
        diagnostics,
        settings,
        download_manager,
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            commands::resolve_media,
            commands::analyze_media,
            commands::plan_acquisition,
            commands::build_command,
            commands::start_download,
            commands::cancel_download,
            commands::get_active_job,
            commands::get_download_history,
            commands::retry_download,
            commands::get_tool_status,
            commands::get_detailed_tool_status,
            commands::install_tool,
            commands::repair_tool,
            commands::install_all_missing_tools,
            commands::auto_bootstrap_tools,
            commands::get_tools_manifest,
            commands::get_settings,
            commands::save_settings,
            commands::get_diagnostics,
            commands::clear_diagnostics,
            commands::open_folder,
            commands::open_file,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
