pub mod commands;
pub use opendownloader_core as core;

use commands::AppState;
use core::diagnostics::DiagnosticsBuffer;
use core::download_manager::DownloadManager;
use core::persistence::JobStore;
use core::settings::SettingsManager;
use core::tools::ToolResolver;
use std::path::Path;
use std::sync::Arc;

fn initialize_job_store(
    database_path: &Path,
    diagnostics: &DiagnosticsBuffer,
) -> Option<Arc<JobStore>> {
    match JobStore::open(database_path) {
        Ok(store) => match store.recover_interrupted() {
            Ok(_) => return Some(Arc::new(store)),
            Err(error) => diagnostics.log(
                "ERROR",
                "PERSISTENCE",
                &format!(
                    "Cannot recover download history; existing database was preserved: {error}"
                ),
            ),
        },
        Err(error) => diagnostics.log(
                "ERROR",
                "PERSISTENCE",
                &format!(
                    "Cannot open the local download history database; existing data was preserved: {error}"
                ),
        ),
    };

    match JobStore::open_in_memory() {
        Ok(store) => Some(Arc::new(store)),
        Err(error) => {
            diagnostics.log(
                "ERROR",
                "PERSISTENCE",
                &format!("Temporary download history is unavailable: {error}"),
            );
            None
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let diagnostics = Arc::new(DiagnosticsBuffer::new());
    let tool_resolver = Arc::new(ToolResolver::with_diagnostics(diagnostics.clone()));
    let settings = Arc::new(SettingsManager::new());
    let database_path = dirs::data_local_dir()
        .unwrap_or_else(std::env::temp_dir)
        .join("opendownloader")
        .join("downloads.sqlite3");
    let job_store = initialize_job_store(&database_path, &diagnostics);
    let mut download_manager =
        DownloadManager::new(tool_resolver.clone(), diagnostics.clone(), settings.clone());
    if let Some(store) = job_store {
        download_manager = download_manager.with_job_store(store);
    }
    let download_manager = Arc::new(download_manager);

    let app_state = AppState {
        tool_resolver,
        diagnostics,
        settings,
        download_manager,
    };

    tauri::Builder::default()
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

#[cfg(test)]
mod tests {
    use super::initialize_job_store;
    use crate::core::diagnostics::DiagnosticsBuffer;

    #[test]
    fn persistence_startup_uses_temporary_history_and_records_failure() {
        let directory = tempfile::tempdir().unwrap();
        let blocked_parent = directory.path().join("not-a-directory");
        std::fs::write(&blocked_parent, b"preserve me").unwrap();
        let database_path = blocked_parent.join("downloads.sqlite3");
        let diagnostics = DiagnosticsBuffer::new();

        let store = initialize_job_store(&database_path, &diagnostics)
            .expect("in-memory persistence should keep the application usable");

        assert!(store.list(10).unwrap().is_empty());
        assert_eq!(std::fs::read(&blocked_parent).unwrap(), b"preserve me");
        assert!(diagnostics
            .get_logs()
            .iter()
            .any(|entry| entry.source == "PERSISTENCE"));
    }

    #[test]
    fn corrupt_database_is_preserved_when_startup_uses_temporary_history() {
        let directory = tempfile::tempdir().unwrap();
        let database_path = directory.path().join("downloads.sqlite3");
        let corrupt_contents = b"preserve this unrecognized database data";
        std::fs::write(&database_path, corrupt_contents).unwrap();
        let diagnostics = DiagnosticsBuffer::new();

        let store = initialize_job_store(&database_path, &diagnostics)
            .expect("in-memory persistence should keep the application usable");

        assert!(store.list(10).unwrap().is_empty());
        assert_eq!(std::fs::read(&database_path).unwrap(), corrupt_contents);
        assert!(diagnostics
            .get_logs()
            .iter()
            .any(|entry| entry.source == "PERSISTENCE"));
    }
}
