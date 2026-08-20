use std::sync::Arc;
use tauri::State;

use crate::core::diagnostics::DiagnosticsBuffer;
use crate::core::download_manager::DownloadManager;
use crate::core::presets::compile_download_args;
use crate::core::settings::SettingsManager;
use crate::core::tools::ToolResolver;
use crate::core::types::{
    AppSettings, BuildCommandRequest, BuildCommandResponse, DiagnosticLog, DownloadJob,
    MediaMetadata, ResolvedMediaSource, StartDownloadRequest, ToolHealth, ToolStatusInfo,
    ToolsManifest,
};
use crate::core::universal_resolver::UniversalResolver;

pub struct AppState {
    pub tool_resolver: Arc<ToolResolver>,
    pub diagnostics: Arc<DiagnosticsBuffer>,
    pub settings: Arc<SettingsManager>,
    pub download_manager: Arc<DownloadManager>,
}

#[tauri::command]
pub async fn resolve_media(
    url: String,
    state: State<'_, AppState>,
) -> Result<ResolvedMediaSource, String> {
    state.diagnostics.log("INFO", "RESOLVER", &format!("Universal resolving URL: {}", url));
    let resolver = UniversalResolver::new(Arc::clone(&state.tool_resolver));
    let result = resolver.resolve(&url).await?;
    state.diagnostics.log(
        "INFO",
        "RESOLVER",
        &format!(
            "Resolved: {:?} | Extractor: {:?} | Strategy: {:?} | Cost: {:?}",
            result.source_type, result.extractor, result.strategy, result.transcoding_cost
        ),
    );
    Ok(result)
}

#[tauri::command]
pub async fn analyze_media(
    url: String,
    state: State<'_, AppState>,
) -> Result<MediaMetadata, String> {
    state.diagnostics.log("INFO", "IPC", &format!("Analyzing URL: {}", url));
    let resolver = UniversalResolver::new(Arc::clone(&state.tool_resolver));
    let resolved = resolver.resolve(&url).await?;
    if let Some(metadata) = resolved.metadata {
        Ok(metadata)
    } else if let Some(err) = resolved.error_detail {
        Err(err.user_friendly_message)
    } else {
        Err("No media found on source".to_string())
    }
}

#[tauri::command]
pub async fn build_command(
    request: BuildCommandRequest,
    state: State<'_, AppState>,
) -> Result<BuildCommandResponse, String> {
    let settings = request
        .settings
        .unwrap_or_else(|| state.settings.get_settings());

    let compiled = compile_download_args(
        request.preset,
        &request.quality,
        &request.output_directory,
        &request.url,
        &settings,
    );

    let full_display = format!("yt-dlp {}", compiled.arguments.join(" "));

    Ok(BuildCommandResponse {
        command: full_display,
        arguments: compiled.arguments,
    })
}

#[tauri::command]
pub async fn start_download(
    request: StartDownloadRequest,
    state: State<'_, AppState>,
) -> Result<DownloadJob, String> {
    state.download_manager.start_download(request).await
}

#[tauri::command]
pub async fn cancel_download(
    job_id: String,
    state: State<'_, AppState>,
) -> Result<DownloadJob, String> {
    state.download_manager.cancel_download(&job_id).await
}

#[tauri::command]
pub async fn get_active_job(
    state: State<'_, AppState>,
) -> Result<Option<DownloadJob>, String> {
    Ok(state.download_manager.get_active_job().await)
}

#[tauri::command]
pub async fn get_tool_status(
    state: State<'_, AppState>,
) -> Result<Vec<ToolHealth>, String> {
    Ok(state.tool_resolver.get_all_tools_health().await)
}

#[tauri::command]
pub async fn get_detailed_tool_status(
    state: State<'_, AppState>,
) -> Result<Vec<ToolStatusInfo>, String> {
    let settings = state.settings.get_settings();
    Ok(state.tool_resolver.get_all_tool_statuses(Some(&settings)).await)
}

#[tauri::command]
pub async fn install_tool(
    name: String,
    state: State<'_, AppState>,
) -> Result<ToolStatusInfo, String> {
    state.tool_resolver.install_tool(&name).await
}

#[tauri::command]
pub async fn repair_tool(
    name: String,
    state: State<'_, AppState>,
) -> Result<ToolStatusInfo, String> {
    state.tool_resolver.repair_tool(&name).await
}

#[tauri::command]
pub async fn install_all_missing_tools(
    state: State<'_, AppState>,
) -> Result<Vec<ToolStatusInfo>, String> {
    state.tool_resolver.install_all_missing().await
}

#[tauri::command]
pub async fn get_tools_manifest(
    state: State<'_, AppState>,
) -> Result<ToolsManifest, String> {
    Ok(state.tool_resolver.manager().load_manifest())
}

#[tauri::command]
pub async fn get_settings(
    state: State<'_, AppState>,
) -> Result<AppSettings, String> {
    Ok(state.settings.get_settings())
}

#[tauri::command]
pub async fn save_settings(
    settings: AppSettings,
    state: State<'_, AppState>,
) -> Result<AppSettings, String> {
    state.settings.save_settings(settings)
}

#[tauri::command]
pub async fn get_diagnostics(
    state: State<'_, AppState>,
) -> Result<Vec<DiagnosticLog>, String> {
    Ok(state.diagnostics.get_logs())
}

#[tauri::command]
pub async fn clear_diagnostics(
    state: State<'_, AppState>,
) -> Result<(), String> {
    state.diagnostics.clear();
    Ok(())
}

#[tauri::command]
pub async fn open_folder(path: String) -> Result<(), String> {
    let trimmed = path.trim();
    if trimmed.is_empty() || trimmed.contains('\0') || trimmed.chars().any(|c| c.is_control()) {
        return Err("Invalid path provided".to_string());
    }

    let p = std::path::Path::new(trimmed);
    if !p.exists() {
        return Err("Target path does not exist on disk".to_string());
    }

    let target_dir = if p.is_file() {
        p.parent().unwrap_or(p)
    } else {
        p
    };

    #[cfg(windows)]
    {
        std::process::Command::new("explorer")
            .arg(target_dir)
            .spawn()
            .map_err(|e| format!("Failed to open explorer: {}", e))?;
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(target_dir)
            .spawn()
            .map_err(|e| format!("Failed to open directory: {}", e))?;
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(target_dir)
            .spawn()
            .map_err(|e| format!("Failed to open directory: {}", e))?;
    }

    Ok(())
}

#[tauri::command]
pub async fn open_file(path: String) -> Result<(), String> {
    let trimmed = path.trim();
    if trimmed.is_empty() || trimmed.contains('\0') || trimmed.chars().any(|c| c.is_control()) {
        return Err("Invalid path provided".to_string());
    }

    let p = std::path::Path::new(trimmed);
    if !p.is_file() {
        return Err("Target file does not exist or is not a regular file".to_string());
    }

    #[cfg(windows)]
    {
        // Safe explorer selection or direct launch without cmd shell
        std::process::Command::new("explorer")
            .arg(format!("/select,{}", p.display()))
            .spawn()
            .map_err(|e| format!("Failed to open file: {}", e))?;
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(p)
            .spawn()
            .map_err(|e| format!("Failed to open file: {}", e))?;
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(p)
            .spawn()
            .map_err(|e| format!("Failed to open file: {}", e))?;
    }

    Ok(())
}
