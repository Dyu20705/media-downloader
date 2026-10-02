use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

use crate::diagnostics::DiagnosticsBuffer;
use crate::types::{
    AppSettings, ToolHealth, ToolManifestEntry, ToolStatus, ToolStatusInfo, ToolsManifest,
};

/// Tool specification definition with pinned supply-chain metadata
#[derive(Debug, Clone)]
pub struct PinnedToolSpec {
    pub name: &'static str,
    pub pinned_version: &'static str,
    pub executable_names: &'static [&'static str],
    pub artifacts: &'static [ToolArtifact],
    pub license: &'static str,
    pub license_url: &'static str,
    pub is_required: bool,
    pub description: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Packaging {
    Raw,
    Zip,
    TarXz,
    TarBz2,
    SevenZip,
}

#[derive(Debug, Clone)]
pub struct ToolArtifact {
    pub os: &'static str,
    pub arch: &'static str,
    pub url: &'static str,
    pub sha256: &'static str,
    pub packaging: Packaging,
    pub executable_path: &'static str,
}

pub fn artifact_for(
    spec: &PinnedToolSpec,
    os: &str,
    arch: &str,
) -> Result<&'static ToolArtifact, String> {
    spec.artifacts
        .iter()
        .find(|a| a.os == os && (a.arch == arch || a.arch == "universal"))
        .ok_or_else(|| {
            format!(
                "No managed {} artifact for {os}/{arch}; configure a custom executable",
                spec.name
            )
        })
}

pub static PINNED_TOOLS: &[PinnedToolSpec] = &[
    PinnedToolSpec {
        name: "yt-dlp",
        pinned_version: "2026.08.19",
        executable_names: if cfg!(windows) { &["yt-dlp.exe", "yt-dlp"] } else { &["yt-dlp"] },
        artifacts: &[
            ToolArtifact { os: "windows", arch: "x86_64", url: "https://github.com/yt-dlp/yt-dlp/releases/download/2026.08.19/yt-dlp.exe", sha256: "66674953fe251b89f4d08c5f0e35e0728679bd67ab3d7d05c0562af101dd3e7a", packaging: Packaging::Raw, executable_path: "yt-dlp.exe" },
            ToolArtifact { os: "linux", arch: "x86_64", url: "https://github.com/yt-dlp/yt-dlp/releases/download/2026.08.19/yt-dlp_linux", sha256: "58162f9bfdc27458ea47bfcb311cf47028f17d8154a8bf7d689861d46399230a", packaging: Packaging::Raw, executable_path: "yt-dlp" },
            ToolArtifact { os: "macos", arch: "universal", url: "https://github.com/yt-dlp/yt-dlp/releases/download/2026.08.19/yt-dlp_macos", sha256: "0f192b7ec147ab6288885d6351d9ab67367640029b4377576ef46dd79cf7b202", packaging: Packaging::Raw, executable_path: "yt-dlp" },
        ],
        license: "Unlicense",
        license_url: "https://github.com/yt-dlp/yt-dlp/blob/master/LICENSE",
        is_required: true,
        description: "Primary media extraction engine",
    },
    PinnedToolSpec {
        name: "ffmpeg",
        pinned_version: "9.0.2",
        executable_names: if cfg!(windows) { &["ffmpeg.exe", "ffmpeg"] } else { &["ffmpeg"] },
        artifacts: &[
            ToolArtifact { os: "windows", arch: "x86_64", url: "https://github.com/BtbN/FFmpeg-Builds/releases/download/autobuild-2026-09-28-13-06/ffmpeg-n9.0.2-14-gebafaee10a-win64-gpl-9.0.zip", sha256: "09170e52cb657f184ba4da2f42567cf2841b661cd9bb5f46ffe4481eb8e6d841", packaging: Packaging::Zip, executable_path: "bin/ffmpeg.exe" },
            ToolArtifact { os: "linux", arch: "x86_64", url: "https://github.com/BtbN/FFmpeg-Builds/releases/download/autobuild-2026-09-28-13-06/ffmpeg-n9.0.2-14-gebafaee10a-linux64-gpl-9.0.tar.xz", sha256: "58e27dab85141ff1e08f43488b7700cc4443e17571bbee809edd6486d9dfd9b2", packaging: Packaging::TarXz, executable_path: "ffmpeg" },
            ToolArtifact { os: "macos", arch: "x86_64", url: "https://evermeet.cx/ffmpeg/ffmpeg-9.0.2.zip", sha256: "4acc0be580f9b2788029eb7bd4d645ff87968911b0a62aeeb3940d42d54558d5", packaging: Packaging::Zip, executable_path: "ffmpeg" },
        ],
        license: "GPL-3.0",
        license_url: "https://ffmpeg.org/legal.html",
        is_required: true,
        description: "Audio/video muxing, encoding, and post-processing engine",
    },
    PinnedToolSpec {
        name: "ffprobe",
        pinned_version: "9.0.2",
        executable_names: if cfg!(windows) { &["ffprobe.exe", "ffprobe"] } else { &["ffprobe"] },
        artifacts: &[
            ToolArtifact { os: "windows", arch: "x86_64", url: "https://github.com/BtbN/FFmpeg-Builds/releases/download/autobuild-2026-09-28-13-06/ffmpeg-n9.0.2-14-gebafaee10a-win64-gpl-9.0.zip", sha256: "09170e52cb657f184ba4da2f42567cf2841b661cd9bb5f46ffe4481eb8e6d841", packaging: Packaging::Zip, executable_path: "bin/ffprobe.exe" },
            ToolArtifact { os: "linux", arch: "x86_64", url: "https://github.com/BtbN/FFmpeg-Builds/releases/download/autobuild-2026-09-28-13-06/ffmpeg-n9.0.2-14-gebafaee10a-linux64-gpl-9.0.tar.xz", sha256: "58e27dab85141ff1e08f43488b7700cc4443e17571bbee809edd6486d9dfd9b2", packaging: Packaging::TarXz, executable_path: "ffprobe" },
            ToolArtifact { os: "macos", arch: "x86_64", url: "https://evermeet.cx/ffmpeg/ffprobe-9.0.2.zip", sha256: "24a9c968cd4da72d99c7245e914b921815835eb6dff01d99868031aebaf1d439", packaging: Packaging::Zip, executable_path: "ffprobe" },
        ],
        license: "GPL-3.0",
        license_url: "https://ffmpeg.org/legal.html",
        is_required: true,
        description: "Media stream analyzer and container inspector",
    },
    PinnedToolSpec {
        name: "mediainfo",
        pinned_version: "24.12",
        executable_names: if cfg!(windows) { &["mediainfo.exe", "MediaInfo.exe", "mediainfo"] } else { &["mediainfo"] },
        artifacts: &[
            ToolArtifact { os: "windows", arch: "x86_64", url: "https://mediaarea.net/download/binary/mediainfo/24.12/MediaInfo_CLI_24.12_Windows_x64.zip", sha256: "f9570aa61fdb930e46124564c7ee23696f4244db919b33a595a882a9341496a7", packaging: Packaging::Zip, executable_path: "MediaInfo.exe" },
            ToolArtifact { os: "macos", arch: "x86_64", url: "https://mediaarea.net/download/binary/mediainfo/24.12/MediaInfo_CLI_24.12_Mac.tar.bz2", sha256: "ea8402db3b72c91838f5fbc7d9f7ad9cbb7a1df58ff7fe2ee398dbd71d3eb847", packaging: Packaging::TarBz2, executable_path: "mediainfo" },
        ],
        license: "BSD-2-Clause",
        license_url: "https://mediaarea.net/en/MediaInfo/License",
        is_required: false,
        description: "Deep technical media inspector and verification helper",
    },
];

pub fn get_pinned_tool_spec(name: &str) -> Option<&'static PinnedToolSpec> {
    PINNED_TOOLS
        .iter()
        .find(|t| t.name.eq_ignore_ascii_case(name))
}

const TOOL_VALIDATION_TIMEOUT: Duration = Duration::from_secs(10);
const MAX_TOOL_DOWNLOAD_BYTES: u64 = 256 * 1024 * 1024;

fn platform_artifact(spec: &PinnedToolSpec) -> Result<&'static ToolArtifact, String> {
    artifact_for(spec, std::env::consts::OS, std::env::consts::ARCH)
}

fn parse_date_version(value: &str) -> Option<(u32, u32, u32)> {
    value.split_whitespace().find_map(|token| {
        let token =
            token.trim_matches(|character: char| !character.is_ascii_digit() && character != '.');
        let mut parts = token.split('.');
        let year = parts.next()?.parse().ok()?;
        let month = parts.next()?.parse().ok()?;
        let day = parts.next()?.parse().ok()?;
        if parts.next().is_some() || !(1..=12).contains(&month) || !(1..=31).contains(&day) {
            return None;
        }
        Some((year, month, day))
    })
}

fn is_date_version_older(version: &str, pinned_version: &str) -> bool {
    match (
        parse_date_version(version),
        parse_date_version(pinned_version),
    ) {
        (Some(actual), Some(pinned)) => actual < pinned,
        _ => false,
    }
}

#[derive(Debug, Clone)]
pub struct ResolvedExecutable {
    pub name: String,
    pub path: PathBuf,
    pub version: Option<String>,
    pub is_managed: bool,
}

fn invalid_tool_status(
    spec: &PinnedToolSpec,
    source_url: &str,
    tool: ResolvedExecutable,
    sha256: Option<String>,
    message: String,
) -> ToolStatusInfo {
    ToolStatusInfo {
        name: spec.name.to_string(),
        status: ToolStatus::Invalid,
        version: tool.version,
        pinned_version: spec.pinned_version.to_string(),
        path: Some(tool.path.to_string_lossy().to_string()),
        managed: true,
        source_url: Some(source_url.to_string()),
        sha256,
        error_message: Some(message),
        license: spec.license.to_string(),
        license_url: spec.license_url.to_string(),
        is_required: spec.is_required,
    }
}

#[derive(Debug)]
pub struct ToolManager {
    tools_base_dir: PathBuf,
    #[cfg(debug_assertions)]
    project_root: PathBuf,
    #[cfg(debug_assertions)]
    development_search: bool,
    diagnostics: Arc<DiagnosticsBuffer>,
    resolution_cache: Arc<RwLock<HashMap<String, ResolvedExecutable>>>,
}

impl ToolManager {
    pub fn new(custom_tools_dir: Option<PathBuf>, diagnostics: Arc<DiagnosticsBuffer>) -> Self {
        let tools_base_dir = custom_tools_dir.unwrap_or_else(Self::resolve_default_tools_directory);
        #[cfg(debug_assertions)]
        let project_root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

        Self {
            tools_base_dir,
            #[cfg(debug_assertions)]
            project_root,
            #[cfg(debug_assertions)]
            development_search: true,
            diagnostics,
            resolution_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    #[cfg(all(debug_assertions, test))]
    fn new_with_roots(
        tools_base_dir: PathBuf,
        project_root: PathBuf,
        diagnostics: Arc<DiagnosticsBuffer>,
        development_search: bool,
    ) -> Self {
        Self {
            tools_base_dir,
            project_root,
            development_search,
            diagnostics,
            resolution_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Compute default application-local directory without touching Windows PATH or System directories:
    /// `%LOCALAPPDATA%\opendownloader\tools\` on Windows
    /// `~/.local/share/opendownloader/tools/` on Linux/macOS
    pub fn resolve_default_tools_directory() -> PathBuf {
        if let Some(local_app_data) = dirs::data_local_dir() {
            local_app_data.join("opendownloader").join("tools")
        } else if let Some(home) = dirs::home_dir() {
            home.join(".opendownloader").join("tools")
        } else {
            PathBuf::from("tools")
        }
    }

    pub fn get_tools_dir(&self) -> &Path {
        &self.tools_base_dir
    }

    pub fn get_staging_dir(&self) -> PathBuf {
        self.tools_base_dir.join(".staging")
    }

    pub fn get_manifest_path(&self) -> PathBuf {
        self.tools_base_dir.join("manifest.json")
    }

    pub fn get_version_dir(&self, tool_name: &str, version: &str) -> PathBuf {
        self.tools_base_dir.join(tool_name).join(version)
    }

    /// Read manifest from disk
    pub fn load_manifest(&self) -> ToolsManifest {
        let manifest_path = self.get_manifest_path();
        if manifest_path.exists() {
            if let Ok(content) = fs::read_to_string(&manifest_path) {
                if let Ok(manifest) = serde_json::from_str::<ToolsManifest>(&content) {
                    return manifest;
                }
            }
        }
        ToolsManifest {
            schema_version: 1,
            tools: HashMap::new(),
            last_updated: chrono::Utc::now().to_rfc3339(),
        }
    }

    /// Atomically write manifest using a temporary file and rename
    pub fn save_manifest_atomic(&self, manifest: &ToolsManifest) -> Result<(), String> {
        let tools_dir = self.get_tools_dir();
        fs::create_dir_all(tools_dir).map_err(|e| format!("Failed to create tools dir: {}", e))?;

        let manifest_path = self.get_manifest_path();
        let nonce = chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0);
        let tmp_path = tools_dir.join(format!("manifest.{nonce}.tmp"));

        let json_data = serde_json::to_string_pretty(manifest)
            .map_err(|e| format!("Failed to serialize manifest: {}", e))?;

        fs::write(&tmp_path, json_data)
            .map_err(|e| format!("Failed to write manifest temp file: {}", e))?;

        #[cfg(not(windows))]
        {
            fs::rename(&tmp_path, &manifest_path)
                .map_err(|e| format!("Failed to atomically commit manifest: {}", e))?;
        }

        #[cfg(windows)]
        {
            let backup_path = tools_dir.join(format!("manifest.{nonce}.bak"));
            let had_manifest = manifest_path.exists();

            if had_manifest {
                fs::rename(&manifest_path, &backup_path)
                    .map_err(|e| format!("Failed to stage previous manifest: {}", e))?;
            }

            if let Err(error) = fs::rename(&tmp_path, &manifest_path) {
                if had_manifest {
                    let _ = fs::rename(&backup_path, &manifest_path);
                }
                let _ = fs::remove_file(&tmp_path);
                return Err(format!("Failed to commit replacement manifest: {}", error));
            }

            if had_manifest {
                let _ = fs::remove_file(&backup_path);
            }
        }

        Ok(())
    }

    /// Compute SHA-256 of any file
    pub fn compute_sha256(path: &Path) -> Result<String, String> {
        let mut file =
            File::open(path).map_err(|e| format!("Cannot open file for hashing: {}", e))?;
        let mut hasher = Sha256::new();
        let mut buffer = [0u8; 65536];

        loop {
            let count = file
                .read(&mut buffer)
                .map_err(|e| format!("Read error while hashing: {}", e))?;
            if count == 0 {
                break;
            }
            hasher.update(&buffer[..count]);
        }

        let hash_bytes = hasher.finalize();
        Ok(hex::encode(hash_bytes))
    }

    /// Verify file SHA-256 against expected hash
    pub fn verify_sha256(path: &Path, expected_sha256: &str) -> Result<bool, String> {
        let actual = Self::compute_sha256(path)?;
        Ok(actual.eq_ignore_ascii_case(expected_sha256.trim()))
    }

    /// Validate executable by invoking its version argument
    pub async fn validate_executable(tool_name: &str, path: &Path) -> Result<String, String> {
        if !path.exists() || !path.is_file() {
            return Err(format!(
                "Executable does not exist or is not a regular file: {:?}",
                path
            ));
        }

        let version_arg = match tool_name.to_lowercase().as_str() {
            "yt-dlp" => "--version",
            "ffmpeg" => "-version",
            "ffprobe" => "-version",
            "mediainfo" => "--Version",
            _ => "--version",
        };

        let mut command = Command::new(path);
        command.arg(version_arg);
        let output = tokio::time::timeout(TOOL_VALIDATION_TIMEOUT, command.output())
            .await
            .map_err(|_| {
                format!(
                    "Executable validation timed out after {} seconds",
                    TOOL_VALIDATION_TIMEOUT.as_secs()
                )
            })?
            .map_err(|e| format!("Failed to spawn executable validation: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!(
                "Executable returned non-zero exit code: {}",
                stderr.trim()
            ));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let first_line = stdout.lines().next().unwrap_or("").trim().to_string();
        if first_line.is_empty() {
            Ok("Installed".to_string())
        } else {
            Ok(first_line)
        }
    }

    /// Resolution order: explicit user path, verified managed tool, then system PATH.
    /// CWD/repository discovery is available only in debug builds.
    pub async fn resolve_tool(
        &self,
        tool_name: &str,
        settings: Option<&AppSettings>,
    ) -> Option<ResolvedExecutable> {
        // An explicit configured path always outranks a cached resolution.
        if let Some(settings) = settings {
            let custom_path = match tool_name {
                "yt-dlp" => settings.custom_ytdlp_path.as_deref(),
                "ffmpeg" => settings.custom_ffmpeg_path.as_deref(),
                "ffprobe" => settings.custom_ffprobe_path.as_deref(),
                "mediainfo" => settings.custom_mediainfo_path.as_deref(),
                _ => None,
            };

            if let Some(raw_path) = custom_path {
                let path = PathBuf::from(raw_path);
                if path.exists() {
                    if let Ok(version) = Self::validate_executable(tool_name, &path).await {
                        let res = ResolvedExecutable {
                            name: tool_name.to_string(),
                            path,
                            version: Some(version),
                            is_managed: false,
                        };
                        self.cache_resolution(tool_name, &res);
                        return Some(res);
                    }
                }
                return None;
            }
        }

        // Check lifetime resolution cache
        if let Ok(cache) = self.resolution_cache.read() {
            if let Some(tool) = cache.get(tool_name) {
                let valid_managed = if tool.is_managed {
                    self.load_manifest()
                        .tools
                        .get(tool_name)
                        .is_some_and(|entry| {
                            entry.verified
                                && entry.path == tool.path.to_string_lossy()
                                && Self::verify_sha256(&tool.path, &entry.sha256).unwrap_or(false)
                        })
                } else {
                    true
                };
                if tool.path.exists() && valid_managed {
                    return Some(ResolvedExecutable {
                        name: tool.name.clone(),
                        path: tool.path.clone(),
                        version: tool.version.clone(),
                        is_managed: tool.is_managed,
                    });
                }
            }
        }

        let exe_names: Vec<String> = if cfg!(windows) {
            vec![
                format!("{}.exe", tool_name),
                format!("{}.EXE", tool_name),
                tool_name.to_string(),
            ]
        } else {
            vec![tool_name.to_string()]
        };

        // Verified application-local managed tool in tools/<tool>/<version>/
        if let Some(spec) = get_pinned_tool_spec(tool_name) {
            let version_dir = self.get_version_dir(tool_name, spec.pinned_version);
            let manifest = self.load_manifest();
            for exe in &exe_names {
                let managed_path = version_dir.join(exe);
                let verified_entry = manifest.tools.get(tool_name).is_some_and(|entry| {
                    entry.verified
                        && entry.path == managed_path.to_string_lossy()
                        && Self::verify_sha256(&managed_path, &entry.sha256).unwrap_or(false)
                });
                if managed_path.exists() && verified_entry {
                    if let Ok(version) = Self::validate_executable(tool_name, &managed_path).await {
                        let res = ResolvedExecutable {
                            name: tool_name.to_string(),
                            path: managed_path,
                            version: Some(version),
                            is_managed: true,
                        };
                        self.cache_resolution(tool_name, &res);
                        return Some(res);
                    }
                }
            }
        }

        // System PATH is an intentional product fallback.
        if let Ok(path_var) = std::env::var("PATH") {
            let split_char = if cfg!(windows) { ';' } else { ':' };
            for dir in path_var.split(split_char) {
                #[cfg(debug_assertions)]
                let reject_relative_path =
                    !self.development_search && !Path::new(dir).is_absolute();
                #[cfg(not(debug_assertions))]
                let reject_relative_path = !Path::new(dir).is_absolute();
                if dir.is_empty() || reject_relative_path {
                    continue;
                }
                let dir_path = Path::new(dir);
                for exe in &exe_names {
                    let full = dir_path.join(exe);
                    if full.exists() {
                        if let Ok(version) = Self::validate_executable(tool_name, &full).await {
                            let res = ResolvedExecutable {
                                name: tool_name.to_string(),
                                path: full,
                                version: Some(version),
                                is_managed: false,
                            };
                            self.cache_resolution(tool_name, &res);
                            return Some(res);
                        }
                    }
                }
            }
        }

        // Repository-relative discovery is compiled into debug builds only.
        #[cfg(debug_assertions)]
        if self.development_search {
            for exe in &exe_names {
                for path in [
                    self.project_root.join(exe),
                    self.project_root.join("bin").join(exe),
                    self.project_root.join("tools").join(exe),
                ] {
                    if path.exists() {
                        if let Ok(version) = Self::validate_executable(tool_name, &path).await {
                            return Some(ResolvedExecutable {
                                name: tool_name.to_string(),
                                path,
                                version: Some(version),
                                is_managed: false,
                            });
                        }
                    }
                }
            }
            let mut candidates = Vec::new();
            let yt_dir = self.project_root.join("youtube-downloader");
            if yt_dir.is_dir() {
                Self::find_files_bounded(&yt_dir, &exe_names, 3, &mut candidates);
            }
            for path in candidates {
                if let Ok(version) = Self::validate_executable(tool_name, &path).await {
                    return Some(ResolvedExecutable {
                        name: tool_name.to_string(),
                        path,
                        version: Some(version),
                        is_managed: false,
                    });
                }
            }
        }

        None
    }

    fn cache_resolution(&self, tool_name: &str, res: &ResolvedExecutable) {
        if let Ok(mut cache) = self.resolution_cache.write() {
            cache.insert(
                tool_name.to_string(),
                ResolvedExecutable {
                    name: res.name.clone(),
                    path: res.path.clone(),
                    version: res.version.clone(),
                    is_managed: res.is_managed,
                },
            );
        }
    }

    pub fn clear_cache(&self) {
        if let Ok(mut cache) = self.resolution_cache.write() {
            cache.clear();
        }
    }

    fn find_files_bounded(dir: &Path, exe_names: &[String], depth: usize, out: &mut Vec<PathBuf>) {
        if depth == 0 {
            return;
        }
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Some(file_name) = path.file_name().and_then(|f| f.to_str()) {
                        if exe_names
                            .iter()
                            .any(|name| name.eq_ignore_ascii_case(file_name))
                        {
                            out.push(path);
                        }
                    }
                } else if path.is_dir() {
                    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                    if name != ".git" && name != "node_modules" && name != "target" {
                        Self::find_files_bounded(&path, exe_names, depth - 1, out);
                    }
                }
            }
        }
    }

    /// Check status of a single tool according to specification
    pub async fn check_tool_status(
        &self,
        tool_name: &str,
        settings: Option<&AppSettings>,
    ) -> ToolStatusInfo {
        let spec = match get_pinned_tool_spec(tool_name) {
            Some(s) => s,
            None => {
                return ToolStatusInfo {
                    name: tool_name.to_string(),
                    status: ToolStatus::Error,
                    version: None,
                    pinned_version: "unknown".to_string(),
                    path: None,
                    managed: false,
                    source_url: None,
                    sha256: None,
                    error_message: Some(format!("Unknown tool: {}", tool_name)),
                    license: "Unknown".to_string(),
                    license_url: "".to_string(),
                    is_required: false,
                }
            }
        };

        let resolved = self.resolve_tool(tool_name, settings).await;

        let artifact = platform_artifact(spec).ok();
        let (source_url, expected_sha256) = artifact.map(|a| (a.url, a.sha256)).unwrap_or(("", ""));

        match resolved {
            Some(tool) => {
                // If it is managed, verify checksum or executable integrity
                if tool.is_managed {
                    let actual_sha = match Self::compute_sha256(&tool.path) {
                        Ok(hash) => hash,
                        Err(error) => {
                            return invalid_tool_status(
                                spec,
                                source_url,
                                tool,
                                None,
                                format!("Could not verify managed binary checksum: {error}"),
                            );
                        }
                    };
                    let manifest = self.load_manifest();
                    let is_manifest_match = manifest.tools.get(tool_name).is_some_and(|entry| {
                        entry.verified
                            && entry.path == tool.path.to_string_lossy()
                            && entry.sha256.eq_ignore_ascii_case(&actual_sha)
                    });
                    let is_valid = if artifact.is_some_and(|a| a.packaging != Packaging::Raw) {
                        is_manifest_match
                    } else {
                        actual_sha.eq_ignore_ascii_case(expected_sha256) || is_manifest_match
                    };

                    if !is_valid {
                        return invalid_tool_status(
                            spec,
                            source_url,
                            tool,
                            Some(actual_sha),
                            "Managed binary checksum does not match its trusted installation record. Reinstallation required."
                                .to_string(),
                        );
                    }

                    ToolStatusInfo {
                        name: tool_name.to_string(),
                        status: ToolStatus::Ready,
                        version: tool.version,
                        pinned_version: spec.pinned_version.to_string(),
                        path: Some(tool.path.to_string_lossy().to_string()),
                        managed: true,
                        source_url: Some(source_url.to_string()),
                        sha256: Some(actual_sha),
                        error_message: None,
                        license: spec.license.to_string(),
                        license_url: spec.license_url.to_string(),
                        is_required: spec.is_required,
                    }
                } else {
                    // Check if a managed directory or manifest entry exists but failed validation
                    let version_dir = self.get_version_dir(tool_name, spec.pinned_version);
                    let manifest = self.load_manifest();
                    let is_explicit_configuration = settings
                        .and_then(|settings| match tool_name {
                            "yt-dlp" => settings.custom_ytdlp_path.as_deref(),
                            "ffmpeg" => settings.custom_ffmpeg_path.as_deref(),
                            "ffprobe" => settings.custom_ffprobe_path.as_deref(),
                            "mediainfo" => settings.custom_mediainfo_path.as_deref(),
                            _ => None,
                        })
                        .is_some_and(|path| Path::new(path) == tool.path);
                    if !is_explicit_configuration
                        && (version_dir.exists() || manifest.tools.contains_key(tool_name))
                    {
                        return ToolStatusInfo {
                            name: tool_name.to_string(),
                            status: ToolStatus::Invalid,
                            version: tool.version,
                            pinned_version: spec.pinned_version.to_string(),
                            path: Some(version_dir.to_string_lossy().to_string()),
                            managed: true,
                            source_url: Some(source_url.to_string()),
                            sha256: Some(expected_sha256.to_string()),
                            error_message: Some("Managed tool executable is damaged or non-executable. Repair required.".to_string()),
                            license: spec.license.to_string(),
                            license_url: spec.license_url.to_string(),
                            is_required: spec.is_required,
                        };
                    }

                    // Non-managed (System PATH or unmanaged fallback)
                    // If the tool executable passed validation, it is operational.
                    let is_outdated = if tool_name == "yt-dlp" {
                        tool.version.as_deref().is_some_and(|version| {
                            is_date_version_older(version, spec.pinned_version)
                        })
                    } else {
                        false
                    };

                    let status = if is_outdated {
                        ToolStatus::Outdated
                    } else {
                        ToolStatus::Ready
                    };

                    let err_msg = if is_outdated {
                        Some(format!(
                            "Legacy build detected ({}). YouTube extraction requires a modern build.",
                            tool.version.as_deref().unwrap_or("unknown")
                        ))
                    } else {
                        None
                    };

                    ToolStatusInfo {
                        name: tool_name.to_string(),
                        status,
                        version: tool.version,
                        pinned_version: spec.pinned_version.to_string(),
                        path: Some(tool.path.to_string_lossy().to_string()),
                        managed: false,
                        source_url: Some(source_url.to_string()),
                        sha256: Some(expected_sha256.to_string()),
                        error_message: err_msg,
                        license: spec.license.to_string(),
                        license_url: spec.license_url.to_string(),
                        is_required: spec.is_required,
                    }
                }
            }
            None => {
                // Check if directory exists but corrupted
                let version_dir = self.get_version_dir(tool_name, spec.pinned_version);
                if version_dir.exists() {
                    ToolStatusInfo {
                        name: tool_name.to_string(),
                        status: ToolStatus::Invalid,
                        version: None,
                        pinned_version: spec.pinned_version.to_string(),
                        path: Some(version_dir.to_string_lossy().to_string()),
                        managed: true,
                        source_url: Some(source_url.to_string()),
                        sha256: Some(expected_sha256.to_string()),
                        error_message: Some(
                            "Tool executable is damaged or non-executable. Repair required."
                                .to_string(),
                        ),
                        license: spec.license.to_string(),
                        license_url: spec.license_url.to_string(),
                        is_required: spec.is_required,
                    }
                } else {
                    ToolStatusInfo {
                        name: tool_name.to_string(),
                        status: ToolStatus::Missing,
                        version: None,
                        pinned_version: spec.pinned_version.to_string(),
                        path: None,
                        managed: false,
                        source_url: Some(source_url.to_string()),
                        sha256: Some(expected_sha256.to_string()),
                        error_message: if spec.is_required {
                            Some(format!(
                                "Required component '{}' is not installed.",
                                tool_name
                            ))
                        } else {
                            None
                        },
                        license: spec.license.to_string(),
                        license_url: spec.license_url.to_string(),
                        is_required: spec.is_required,
                    }
                }
            }
        }
    }

    /// Check status of all managed tools
    pub async fn get_all_tool_statuses(
        &self,
        settings: Option<&AppSettings>,
    ) -> Vec<ToolStatusInfo> {
        let mut results = Vec::new();
        for spec in PINNED_TOOLS {
            results.push(self.check_tool_status(spec.name, settings).await);
        }
        results
    }

    /// Backward-compatible health report
    pub async fn get_all_tools_health_with_settings(
        &self,
        settings: Option<&AppSettings>,
    ) -> Vec<ToolHealth> {
        let statuses = self.get_all_tool_statuses(settings).await;
        statuses
            .into_iter()
            .map(|s| {
                let available = s.status == ToolStatus::Ready;
                let repair_msg = if !available {
                    s.error_message
                        .or_else(|| Some(format!("Click Repair to install {}", s.name)))
                } else {
                    None
                };
                ToolHealth {
                    name: s.name,
                    available,
                    path: s.path,
                    version: s.version,
                    repair_message: repair_msg,
                }
            })
            .collect()
    }

    pub async fn get_all_tools_health(&self) -> Vec<ToolHealth> {
        self.get_all_tools_health_with_settings(None).await
    }

    /// Install tool atomically:
    /// 1. Download to temporary file in staging directory
    /// 2. Verify SHA-256
    /// 3. Extract if archive / copy binary
    /// 4. Verify executable execution (`--version`)
    /// 5. Move into versioned directory (`tools/<name>/<version>/`)
    /// 6. Atomically update `manifest.json`
    /// 7. Clean up staging
    pub async fn install_tool(&self, tool_name: &str) -> Result<ToolStatusInfo, String> {
        let spec = get_pinned_tool_spec(tool_name)
            .ok_or_else(|| format!("Unknown tool specification: {}", tool_name))?;

        self.diagnostics.log(
            "info",
            "tool_manager",
            &format!("Starting atomic installation for {}", tool_name),
        );

        let artifact = platform_artifact(spec)?;
        if artifact.packaging == Packaging::SevenZip {
            return Err(
                "Managed 7z extraction is not supported; configure a custom executable".into(),
            );
        }
        let (source_url, expected_sha256) = (artifact.url, artifact.sha256);

        let staging_dir = self.get_staging_dir();
        fs::create_dir_all(&staging_dir)
            .map_err(|e| format!("Failed to create staging dir: {}", e))?;

        let tmp_file_name = format!(
            "{}-{}.tmp",
            tool_name,
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
        );
        let tmp_file_path = staging_dir.join(&tmp_file_name);

        self.diagnostics.log(
            "info",
            "tool_manager",
            &format!("Downloading {} from {}", tool_name, source_url),
        );

        // Perform download
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .user_agent("opendownloader/1.0")
            .redirect(reqwest::redirect::Policy::limited(10))
            .build()
            .map_err(|e| format!("Failed to build HTTP client: {}", e))?;

        let mut response = client
            .get(source_url)
            .send()
            .await
            .map_err(|e| format!("Download request failed for {}: {}", tool_name, e))?;

        if !response.status().is_success() {
            return Err(format!(
                "Download failed with HTTP status: {}",
                response.status()
            ));
        }

        if response
            .content_length()
            .is_some_and(|length| length > MAX_TOOL_DOWNLOAD_BYTES)
        {
            return Err(format!(
                "Tool archive exceeds the {} MiB download limit",
                MAX_TOOL_DOWNLOAD_BYTES / (1024 * 1024)
            ));
        }

        let install_res = async {
            let mut file = tokio::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&tmp_file_path)
                .await
                .map_err(|e| format!("Failed to create staging file: {e}"))?;
            let mut downloaded = 0u64;
            let mut hasher = Sha256::new();
            while let Some(chunk) = response
                .chunk()
                .await
                .map_err(|e| format!("Failed to read response stream: {e}"))?
            {
                downloaded = downloaded
                    .checked_add(chunk.len() as u64)
                    .ok_or("Tool download size overflow")?;
                if downloaded > MAX_TOOL_DOWNLOAD_BYTES {
                    return Err("Tool archive exceeds the 256 MiB download limit".into());
                }
                hasher.update(&chunk);
                file.write_all(&chunk)
                    .await
                    .map_err(|e| format!("Failed to write staging file: {e}"))?;
            }
            file.sync_all()
                .await
                .map_err(|e| format!("Failed to flush staging file: {e}"))?;
            drop(file);
            if !hex::encode(hasher.finalize()).eq_ignore_ascii_case(expected_sha256) {
                return Err("Security checksum validation failed for downloaded tool".into());
            }
            self.atomic_install_from_staging(
                tool_name,
                &tmp_file_path,
                expected_sha256,
                artifact.packaging,
                artifact.executable_path,
            )
            .await
        }
        .await;
        let _ = fs::remove_file(&tmp_file_path);

        match install_res {
            Ok(status_info) => {
                self.clear_cache();
                self.diagnostics.log(
                    "info",
                    "tool_manager",
                    &format!(
                        "Successfully installed {} v{}",
                        tool_name, spec.pinned_version
                    ),
                );
                Ok(status_info)
            }
            Err(e) => {
                self.diagnostics.log(
                    "error",
                    "tool_manager",
                    &format!("Installation failed for {}: {}", tool_name, e),
                );
                Err(e)
            }
        }
    }

    /// Direct atomic install from raw bytes (supports embedded / mock / offline installs)
    pub async fn install_from_bytes(
        &self,
        tool_name: &str,
        bytes: &[u8],
        expected_sha256: &str,
        is_archive: bool,
    ) -> Result<ToolStatusInfo, String> {
        let staging_dir = self.get_staging_dir();
        fs::create_dir_all(&staging_dir)
            .map_err(|e| format!("Failed to create staging dir: {}", e))?;

        let tmp_file_name = format!(
            "{}-{}.tmp",
            tool_name,
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
        );
        let tmp_file_path = staging_dir.join(&tmp_file_name);

        fs::write(&tmp_file_path, bytes)
            .map_err(|e| format!("Failed to write staging file: {}", e))?;

        let res = self
            .atomic_install_from_staging(
                tool_name,
                &tmp_file_path,
                expected_sha256,
                if is_archive {
                    Packaging::Zip
                } else {
                    Packaging::Raw
                },
                if cfg!(windows) {
                    spec_binary_name(tool_name)
                } else {
                    tool_name
                },
            )
            .await;
        let _ = fs::remove_file(&tmp_file_path);
        if res.is_ok() {
            self.clear_cache();
        }
        res
    }

    /// Internal atomic pipeline from staging file
    async fn atomic_install_from_staging(
        &self,
        tool_name: &str,
        staged_file: &Path,
        expected_sha256: &str,
        packaging: Packaging,
        executable_path: &str,
    ) -> Result<ToolStatusInfo, String> {
        let spec = get_pinned_tool_spec(tool_name)
            .ok_or_else(|| format!("Unknown tool specification: {}", tool_name))?;

        self.diagnostics.log(
            "info",
            "tool_manager",
            &format!("Verifying SHA-256 checksum for {}", tool_name),
        );
        let matches = Self::verify_sha256(staged_file, expected_sha256)?;
        if !matches {
            let actual = Self::compute_sha256(staged_file)?;
            return Err(format!(
                "Security checksum validation failed for {}. Expected {}, got {}",
                tool_name, expected_sha256, actual
            ));
        }

        let target_version_dir = self.get_version_dir(tool_name, spec.pinned_version);
        let staging_extract_dir = self.get_staging_dir().join(format!(
            "{}-extracted-{}",
            tool_name,
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
        ));

        fs::create_dir_all(&staging_extract_dir)
            .map_err(|e| format!("Failed to create staging extraction dir: {}", e))?;

        let final_bin_name = if cfg!(windows) {
            format!("{}.exe", tool_name)
        } else {
            tool_name.to_string()
        };

        let staged_bin_path = staging_extract_dir.join(&final_bin_name);

        if let Err(error) =
            extract_binary(staged_file, &staged_bin_path, packaging, executable_path).await
        {
            let _ = fs::remove_dir_all(&staging_extract_dir);
            return Err(error);
        }

        // Ensure executable permissions on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(metadata) = fs::metadata(&staged_bin_path) {
                let mut perms = metadata.permissions();
                perms.set_mode(0o755);
                let _ = fs::set_permissions(&staged_bin_path, perms);
            }
        }

        self.diagnostics.log(
            "info",
            "tool_manager",
            &format!("Validating executable binary for {}", tool_name),
        );
        let version_str = match Self::validate_executable(tool_name, &staged_bin_path).await {
            Ok(v) => v,
            Err(e) => {
                let _ = fs::remove_dir_all(&staging_extract_dir);
                return Err(format!(
                    "Executable validation test failed for {}: {}",
                    tool_name, e
                ));
            }
        };

        fs::create_dir_all(&target_version_dir)
            .map_err(|e| format!("Failed to create target version dir: {}", e))?;

        let final_destination = target_version_dir.join(&final_bin_name);
        let backup = target_version_dir.join(format!(
            ".{final_bin_name}.{}.bak",
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
        ));
        let had_previous = final_destination.exists();
        if had_previous {
            fs::rename(&final_destination, &backup)
                .map_err(|e| format!("Could not preserve previous executable: {e}"))?;
        }
        if let Err(error) = fs::rename(&staged_bin_path, &final_destination) {
            if had_previous {
                let _ = fs::rename(&backup, &final_destination);
            }
            let _ = fs::remove_dir_all(&staging_extract_dir);
            return Err(format!("Failed to activate verified executable: {error}"));
        }

        // Ensure final destination has executable permissions on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(metadata) = fs::metadata(&final_destination) {
                let mut perms = metadata.permissions();
                perms.set_mode(0o755);
                let _ = fs::set_permissions(&final_destination, perms);
            }
        }

        let _ = fs::remove_dir_all(&staging_extract_dir);

        let mut manifest = self.load_manifest();
        let bin_sha = Self::compute_sha256(&final_destination)?;

        manifest.tools.insert(
            tool_name.to_string(),
            ToolManifestEntry {
                name: tool_name.to_string(),
                version: spec.pinned_version.to_string(),
                path: final_destination.to_string_lossy().to_string(),
                sha256: bin_sha.clone(),
                installed_at: chrono::Utc::now().to_rfc3339(),
                verified: true,
            },
        );
        manifest.last_updated = chrono::Utc::now().to_rfc3339();
        if let Err(error) = self.save_manifest_atomic(&manifest) {
            let _ = fs::remove_file(&final_destination);
            if had_previous {
                let _ = fs::rename(&backup, &final_destination);
            }
            return Err(error);
        }
        if had_previous {
            let _ = fs::remove_file(&backup);
        }

        Ok(ToolStatusInfo {
            name: tool_name.to_string(),
            status: ToolStatus::Ready,
            version: Some(version_str),
            pinned_version: spec.pinned_version.to_string(),
            path: Some(final_destination.to_string_lossy().to_string()),
            managed: true,
            source_url: platform_artifact(spec).ok().map(|a| a.url.to_string()),
            sha256: Some(bin_sha),
            error_message: None,
            license: spec.license.to_string(),
            license_url: spec.license_url.to_string(),
            is_required: spec.is_required,
        })
    }

    /// Repair or reinstall a tool
    pub async fn repair_tool(&self, tool_name: &str) -> Result<ToolStatusInfo, String> {
        get_pinned_tool_spec(tool_name).ok_or_else(|| format!("Unknown tool: {}", tool_name))?;

        self.diagnostics.log(
            "warn",
            "tool_manager",
            &format!("Initiating repair/reinstall for {}", tool_name),
        );

        // Keep the existing installation until its replacement has passed validation.
        self.clear_cache();
        self.install_tool(tool_name).await
    }

    /// Install or upgrade all missing, invalid, or outdated tools
    pub async fn install_all_missing(&self) -> Result<Vec<ToolStatusInfo>, String> {
        let statuses = self.get_all_tool_statuses(None).await;
        let mut results = Vec::new();

        for status in statuses {
            if status.status == ToolStatus::Missing
                || status.status == ToolStatus::Invalid
                || status.status == ToolStatus::Outdated
            {
                match self.install_tool(&status.name).await {
                    Ok(installed) => results.push(installed),
                    Err(e) => {
                        results.push(ToolStatusInfo {
                            name: status.name.clone(),
                            status: ToolStatus::Error,
                            version: None,
                            pinned_version: status.pinned_version,
                            path: None,
                            managed: false,
                            source_url: status.source_url,
                            sha256: status.sha256,
                            error_message: Some(e),
                            license: status.license,
                            license_url: status.license_url,
                            is_required: status.is_required,
                        });
                    }
                }
            } else {
                results.push(status);
            }
        }

        Ok(results)
    }

    /// Automatically bootstrap required tools on startup if missing or unmanaged
    pub async fn auto_bootstrap_required_tools(&self) -> Result<Vec<ToolStatusInfo>, String> {
        self.diagnostics.log(
            "info",
            "tool_manager",
            "Checking and auto-bootstrapping required engine tools",
        );
        self.install_all_missing().await
    }
}

#[cfg(test)]
mod version_tests {
    use super::*;

    #[test]
    fn date_versions_are_compared_to_the_pin() {
        assert!(is_date_version_older("2025.12.31", "2026.08.19"));
        assert!(!is_date_version_older("2026.08.19", "2026.08.19"));
        assert!(!is_date_version_older("stable 2026.09.02", "2026.08.19"));
        assert!(!is_date_version_older("unknown", "2026.08.19"));
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn production_resolution_never_selects_cwd_binaries() {
        use std::os::unix::fs::PermissionsExt;
        let cwd = tempfile::tempdir().unwrap();
        let tools = tempfile::tempdir().unwrap();
        let marker = cwd.path().join("executed");
        for name in ["yt-dlp", "ffmpeg", "ffprobe", "mediainfo"] {
            let binary = cwd.path().join(name);
            fs::write(
                &binary,
                format!("#!/bin/sh\ntouch '{}'\necho fake\n", marker.display()),
            )
            .unwrap();
            let mut permissions = fs::metadata(&binary).unwrap().permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(&binary, permissions).unwrap();
        }
        let manager = ToolManager::new_with_roots(
            tools.path().to_path_buf(),
            cwd.path().to_path_buf(),
            Arc::new(DiagnosticsBuffer::new()),
            false,
        );
        for name in ["yt-dlp", "ffmpeg", "ffprobe", "mediainfo"] {
            if let Some(tool) = manager.resolve_tool(name, None).await {
                assert!(!tool.path.starts_with(cwd.path()), "selected CWD {name}");
            }
        }
        assert!(!marker.exists(), "untrusted CWD executable was invoked");
    }
}

fn spec_binary_name(name: &str) -> &str {
    match name {
        "yt-dlp" => "yt-dlp.exe",
        "ffmpeg" => "ffmpeg.exe",
        "ffprobe" => "ffprobe.exe",
        "mediainfo" => "mediainfo.exe",
        other => other,
    }
}

fn matching_member(name: &str, expected: &str) -> bool {
    let path = Path::new(name);
    !path.is_absolute()
        && !name.contains('\\')
        && !name.contains(':')
        && path.components().all(|p| {
            matches!(
                p,
                std::path::Component::Normal(_) | std::path::Component::CurDir
            )
        })
        && (name.eq_ignore_ascii_case(expected)
            || name
                .to_lowercase()
                .ends_with(&format!("/{}", expected.to_lowercase())))
}

async fn extract_binary(
    archive: &Path,
    output: &Path,
    packaging: Packaging,
    expected: &str,
) -> Result<(), String> {
    match packaging {
        Packaging::Raw => {
            fs::copy(archive, output).map_err(|e| e.to_string())?;
        }
        Packaging::SevenZip => {
            return Err(
                "Managed 7z extraction is not supported; configure a custom executable".into(),
            )
        }
        Packaging::Zip => {
            let file = File::open(archive).map_err(|e| e.to_string())?;
            let mut zip =
                zip::ZipArchive::new(file).map_err(|e| format!("Invalid ZIP archive: {e}"))?;
            let mut matches = Vec::new();
            for i in 0..zip.len() {
                let entry = zip.by_index(i).map_err(|e| e.to_string())?;
                if entry.is_file() && matching_member(entry.name(), expected) {
                    matches.push(i);
                }
            }
            if matches.len() != 1 {
                return Err(format!(
                    "Expected one '{expected}' executable in ZIP, found {}",
                    matches.len()
                ));
            }
            let mut entry = zip.by_index(matches[0]).map_err(|e| e.to_string())?;
            if entry.size() > MAX_TOOL_DOWNLOAD_BYTES {
                return Err("Extracted executable exceeds size limit".into());
            }
            let mut target = File::create(output).map_err(|e| e.to_string())?;
            std::io::copy(&mut entry, &mut target).map_err(|e| e.to_string())?;
        }
        Packaging::TarXz | Packaging::TarBz2 => {
            let listing = tokio::time::timeout(
                TOOL_VALIDATION_TIMEOUT,
                Command::new("tar")
                    .kill_on_drop(true)
                    .arg("-tf")
                    .arg(archive)
                    .output(),
            )
            .await
            .map_err(|_| "Archive listing timed out")?
            .map_err(|e| e.to_string())?;
            if !listing.status.success() {
                return Err("Unable to list tar archive".into());
            }
            let names = String::from_utf8(listing.stdout).map_err(|e| e.to_string())?;
            let matches: Vec<_> = names
                .lines()
                .filter(|name| matching_member(name, expected))
                .collect();
            if matches.len() != 1 {
                return Err(format!(
                    "Expected one '{expected}' executable in tar, found {}",
                    matches.len()
                ));
            }
            let mut child = Command::new("tar")
                .kill_on_drop(true)
                .arg("-xOf")
                .arg(archive)
                .arg("--")
                .arg(matches[0])
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::null())
                .spawn()
                .map_err(|e| e.to_string())?;
            let mut stdout = child.stdout.take().ok_or("Missing tar stdout")?;
            let mut target = tokio::fs::File::create(output)
                .await
                .map_err(|e| e.to_string())?;
            let copy = async {
                use tokio::io::AsyncReadExt;
                let count = tokio::io::copy(
                    &mut (&mut stdout).take(MAX_TOOL_DOWNLOAD_BYTES + 1),
                    &mut target,
                )
                .await
                .map_err(|e| e.to_string())?;
                if count > MAX_TOOL_DOWNLOAD_BYTES {
                    return Err("Extracted executable exceeds size limit".to_string());
                }
                if !child.wait().await.map_err(|e| e.to_string())?.success() {
                    return Err("Tar extraction failed".into());
                }
                Ok(())
            };
            tokio::time::timeout(Duration::from_secs(60), copy)
                .await
                .map_err(|_| "Archive extraction timed out")??;
        }
    }
    Ok(())
}

#[cfg(test)]
mod archive_tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn artifact_matrix_rejects_wrong_arch_and_source_packages() {
        let ffmpeg = get_pinned_tool_spec("ffmpeg").unwrap();
        assert_eq!(
            artifact_for(ffmpeg, "windows", "x86_64").unwrap().packaging,
            Packaging::Zip
        );
        assert_eq!(
            artifact_for(ffmpeg, "linux", "x86_64").unwrap().packaging,
            Packaging::TarXz
        );
        assert_eq!(
            artifact_for(ffmpeg, "macos", "x86_64").unwrap().packaging,
            Packaging::Zip
        );
        assert!(artifact_for(ffmpeg, "linux", "aarch64").is_err());
        assert!(artifact_for(ffmpeg, "macos", "aarch64").is_err());
        assert!(artifact_for(get_pinned_tool_spec("yt-dlp").unwrap(), "linux", "aarch64").is_err());
        assert!(artifact_for(ffmpeg, "freebsd", "x86_64").is_err());
        assert!(artifact_for(
            get_pinned_tool_spec("mediainfo").unwrap(),
            "linux",
            "x86_64"
        )
        .is_err());
        assert!(artifact_for(get_pinned_tool_spec("yt-dlp").unwrap(), "macos", "aarch64").is_ok());
    }

    #[tokio::test]
    async fn zip_extracts_only_unique_declared_executable() {
        let temp = tempfile::tempdir().unwrap();
        let archive = temp.path().join("fixture.zip");
        let output = temp.path().join("binary");
        let create_zip = |names: &[&str]| {
            let mut zip = zip::ZipWriter::new(File::create(&archive).unwrap());
            for name in names {
                zip.start_file(*name, zip::write::SimpleFileOptions::default())
                    .unwrap();
                zip.write_all(b"fixture").unwrap();
            }
            zip.finish().unwrap();
        };
        create_zip(&["release/bin/ffmpeg.exe", "release/readme.txt"]);
        extract_binary(&archive, &output, Packaging::Zip, "bin/ffmpeg.exe")
            .await
            .unwrap();
        assert_eq!(fs::read(&output).unwrap(), b"fixture");
        assert!(!temp.path().join("release").exists());
        create_zip(&["a/bin/ffmpeg.exe", "b/bin/ffmpeg.exe"]);
        assert!(
            extract_binary(&archive, &output, Packaging::Zip, "bin/ffmpeg.exe")
                .await
                .unwrap_err()
                .contains("found 2")
        );
        create_zip(&["../bin/ffmpeg.exe"]);
        assert!(
            extract_binary(&archive, &output, Packaging::Zip, "bin/ffmpeg.exe")
                .await
                .is_err()
        );
        assert!(
            extract_binary(&archive, &output, Packaging::SevenZip, "ffmpeg")
                .await
                .unwrap_err()
                .contains("7z")
        );
    }

    #[test]
    fn rejects_unsafe_member_names() {
        for name in [
            "/bin/ffmpeg",
            "../ffmpeg",
            "a/../../ffmpeg",
            "C:/bin/ffmpeg",
            "a\\..\\ffmpeg",
        ] {
            assert!(!matching_member(name, "ffmpeg"), "{name}");
        }
        assert!(matching_member("release/ffmpeg", "ffmpeg"));
    }
}
