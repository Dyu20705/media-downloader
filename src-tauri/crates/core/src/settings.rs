use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use crate::types::AppSettings;

const SETTINGS_TEMP_SUFFIX: &str = ".tmp";
const SETTINGS_BACKUP_SUFFIX: &str = ".bak";

#[derive(Debug, Clone)]
pub struct SettingsManager {
    file_path: PathBuf,
    cached: Arc<RwLock<AppSettings>>,
}

impl SettingsManager {
    pub fn new() -> Self {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("one-click-media-downloader");

        let file_path = config_dir.join("settings.json");
        let initial_settings = Self::load_from_disk(&file_path);

        Self {
            file_path,
            cached: Arc::new(RwLock::new(initial_settings)),
        }
    }

    fn load_from_disk(path: &Path) -> AppSettings {
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(path) {
                if let Ok(settings) = serde_json::from_str::<AppSettings>(&content) {
                    return settings;
                }
            }

            // Corruption recovery: Try to load from backup file if main is corrupted
            let backup_path = path.with_extension(SETTINGS_BACKUP_SUFFIX);
            if backup_path.exists() {
                if let Ok(content) = std::fs::read_to_string(&backup_path) {
                    if let Ok(settings) = serde_json::from_str::<AppSettings>(&content) {
                        return settings;
                    }
                }
            }
        }
        AppSettings::default()
    }

    pub fn get_settings(&self) -> AppSettings {
        self.cached.read().map(|s| s.clone()).unwrap_or_default()
    }

    /// Atomic write with temporary file, rename, and corruption recovery backup
    pub fn save_settings(&self, new_settings: AppSettings) -> Result<AppSettings, String> {
        if let Some(parent) = self.file_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        let json_str = serde_json::to_string_pretty(&new_settings)
            .map_err(|e| format!("Failed to serialize settings: {}", e))?;

        // 1. Write to temporary file first
        let tmp_file_path = self.file_path.with_extension(format!("json{}", SETTINGS_TEMP_SUFFIX));
        std::fs::write(&tmp_file_path, &json_str)
            .map_err(|e| format!("Failed to write temporary settings: {}", e))?;

        // 2. Create backup of current settings if exists
        let backup_path = self.file_path.with_extension(SETTINGS_BACKUP_SUFFIX);
        if self.file_path.exists() {
            let _ = std::fs::copy(&self.file_path, &backup_path);
        }

        // 3. Atomically rename temporary file over target
        if let Err(rename_err) = std::fs::rename(&tmp_file_path, &self.file_path) {
            // Fallback for filesystems that do not support overwrite rename
            std::fs::copy(&tmp_file_path, &self.file_path)
                .map_err(|e| format!("Failed to atomically commit settings file: {} (rename error: {})", e, rename_err))?;
            let _ = std::fs::remove_file(&tmp_file_path);
        }

        if let Ok(mut cache) = self.cached.write() {
            *cache = new_settings.clone();
        }

        Ok(new_settings)
    }
}
