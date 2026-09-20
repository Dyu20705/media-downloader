pub mod commands;
pub use ocmd_core as core;

use std::sync::Arc;
use commands::AppState;
use core::diagnostics::DiagnosticsBuffer;
use core::download_manager::DownloadManager;
use core::settings::SettingsManager;
use core::tools::ToolResolver;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let tool_resolver = Arc::new(ToolResolver::new());
    let diagnostics = Arc::new(DiagnosticsBuffer::new());
    let settings = Arc::new(SettingsManager::new());
    let download_manager = Arc::new(DownloadManager::new(
        tool_resolver.clone(),
        diagnostics.clone(),
        settings.clone(),
    ));

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
            commands::build_command,
            commands::get_download_plan,
            commands::start_download,
            commands::cancel_download,
            commands::get_active_job,
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
