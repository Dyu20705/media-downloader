use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
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
    pub windows_url: &'static str,
    pub windows_sha256: &'static str,
    pub linux_url: &'static str,
    pub linux_sha256: &'static str,
    pub darwin_url: &'static str,
    pub darwin_sha256: &'static str,
    pub license: &'static str,
    pub license_url: &'static str,
    pub is_archive: bool,
    pub archive_bin_path: Option<&'static str>,
    pub is_required: bool,
    pub description: &'static str,
}

pub static PINNED_TOOLS: &[PinnedToolSpec] = &[
    PinnedToolSpec {
        name: "yt-dlp",
        pinned_version: "2025.02.19",
        executable_names: if cfg!(windows) { &["yt-dlp.exe", "yt-dlp"] } else { &["yt-dlp"] },
        windows_url: "https://github.com/yt-dlp/yt-dlp/releases/download/2025.02.19/yt-dlp.exe",
        windows_sha256: "785f73d2a71d7992984ea7c3ea4e1837895e7c8ecba0aa2286e1aafe9d424b91",
        linux_url: "https://github.com/yt-dlp/yt-dlp/releases/download/2025.02.19/yt-dlp",
        linux_sha256: "57ca88402db3b72c91838f5fbc7d9f7ad9cbb7a1df58ff7fe2ee398dbd71d3eb",
        darwin_url: "https://github.com/yt-dlp/yt-dlp/releases/download/2025.02.19/yt-dlp_macos",
        darwin_sha256: "d347ffc93839be9b4f2c0df82811a2164746fce529683679c6d39ad85ffccfce",
        license: "Unlicense",
        license_url: "https://github.com/yt-dlp/yt-dlp/blob/master/LICENSE",
        is_archive: false,
        archive_bin_path: None,
        is_required: true,
        description: "Primary media extraction engine",
    },
    PinnedToolSpec {
        name: "ffmpeg",
        pinned_version: "7.1",
        executable_names: if cfg!(windows) { &["ffmpeg.exe", "ffmpeg"] } else { &["ffmpeg"] },
        windows_url: "https://github.com/GyanD/codexffmpeg/releases/download/7.1/ffmpeg-7.1-essentials_build.zip",
        windows_sha256: "a937a00f2771d9d150ae1ae6e61f22eec74b41fb386eaebdfd5ffcfcfbc99d99",
        linux_url: "https://johnvansickle.com/ffmpeg/releases/ffmpeg-release-amd64-static.tar.xz",
        linux_sha256: "e06fa99e2e604f32386e594d80509a25cefcfe02c11eeeb2d9ee18e244b749ad",
        darwin_url: "https://evermeet.cx/ffmpeg/ffmpeg-7.1.7z",
        darwin_sha256: "c18a00351283c74ee5d14e1f7a1496a71eec8b74dfa98075fbc074558298711e",
        license: "GPL-3.0 / LGPL-2.1+",
        license_url: "https://ffmpeg.org/legal.html",
        is_archive: true,
        archive_bin_path: if cfg!(windows) { Some("bin/ffmpeg.exe") } else { Some("ffmpeg") },
        is_required: true,
        description: "Audio/video muxing, encoding, and post-processing engine",
    },
    PinnedToolSpec {
        name: "ffprobe",
        pinned_version: "7.1",
        executable_names: if cfg!(windows) { &["ffprobe.exe", "ffprobe"] } else { &["ffprobe"] },
        windows_url: "https://github.com/GyanD/codexffmpeg/releases/download/7.1/ffmpeg-7.1-essentials_build.zip",
        windows_sha256: "a937a00f2771d9d150ae1ae6e61f22eec74b41fb386eaebdfd5ffcfcfbc99d99",
        linux_url: "https://johnvansickle.com/ffmpeg/releases/ffmpeg-release-amd64-static.tar.xz",
        linux_sha256: "e06fa99e2e604f32386e594d80509a25cefcfe02c11eeeb2d9ee18e244b749ad",
        darwin_url: "https://evermeet.cx/ffmpeg/ffprobe-7.1.7z",
        darwin_sha256: "d19b33a595a882a9341496a7a937a00f2771d9d150ae1ae6e61f22eec74b41fb",
        license: "GPL-3.0 / LGPL-2.1+",
        license_url: "https://ffmpeg.org/legal.html",
        is_archive: true,
        archive_bin_path: if cfg!(windows) { Some("bin/ffprobe.exe") } else { Some("ffprobe") },
        is_required: true,
        description: "Media stream analyzer and container inspector",
    },
    PinnedToolSpec {
        name: "mediainfo",
        pinned_version: "24.12",
        executable_names: if cfg!(windows) { &["mediainfo.exe", "MediaInfo.exe", "mediainfo"] } else { &["mediainfo"] },
        windows_url: "https://mediaarea.net/download/binary/mediainfo/24.12/MediaInfo_CLI_24.12_Windows_x64.zip",
        windows_sha256: "f9570aa61fdb930e46124564c7ee23696f4244db919b33a595a882a9341496a7",
        linux_url: "https://mediaarea.net/download/binary/mediainfo/24.12/MediaInfo_CLI_24.12_GNU_FromSource.tar.xz",
        linux_sha256: "918ef930263f3503f8f117ceecb38b7e4f9b8c08a957b494676579308b47f6d2",
        darwin_url: "https://mediaarea.net/download/binary/mediainfo/24.12/MediaInfo_CLI_24.12_Mac.tar.bz2",
        darwin_sha256: "ea8402db3b72c91838f5fbc7d9f7ad9cbb7a1df58ff7fe2ee398dbd71d3eb847",
        license: "BSD-2-Clause",
        license_url: "https://mediaarea.net/en/MediaInfo/License",
        is_archive: true,
        archive_bin_path: if cfg!(windows) { Some("MediaInfo.exe") } else { Some("mediainfo") },
        is_required: false,
        description: "Deep technical media inspector and verification helper",
    },
];

pub fn get_pinned_tool_spec(name: &str) -> Option<&'static PinnedToolSpec> {
    PINNED_TOOLS
        .iter()
        .find(|t| t.name.eq_ignore_ascii_case(name))
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
    project_root: PathBuf,
    diagnostics: Arc<DiagnosticsBuffer>,
    resolution_cache: Arc<RwLock<HashMap<String, ResolvedExecutable>>>,
}

impl ToolManager {
    pub fn new(custom_tools_dir: Option<PathBuf>, diagnostics: Arc<DiagnosticsBuffer>) -> Self {
        let tools_base_dir = custom_tools_dir.unwrap_or_else(Self::resolve_default_tools_directory);
        let project_root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

        Self {
            tools_base_dir,
            project_root,
            diagnostics,
            resolution_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Compute default application-local directory without touching Windows PATH or System directories:
    /// `%LOCALAPPDATA%\OneClickMediaDownloader\tools\` on Windows
    /// `~/.local/share/one-click-media-downloader/tools/` on Linux/macOS
    pub fn resolve_default_tools_directory() -> PathBuf {
        if let Some(local_app_data) = dirs::data_local_dir() {
            local_app_data.join("OneClickMediaDownloader").join("tools")
        } else if let Some(home) = dirs::home_dir() {
            home.join(".one-click-media-downloader").join("tools")
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
        let tmp_path = tools_dir.join(format!(
            "manifest.{}.tmp",
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
        ));

        let json_data = serde_json::to_string_pretty(manifest)
            .map_err(|e| format!("Failed to serialize manifest: {}", e))?;

        fs::write(&tmp_path, json_data)
            .map_err(|e| format!("Failed to write manifest temp file: {}", e))?;

        fs::rename(&tmp_path, &manifest_path)
            .map_err(|e| format!("Failed to atomically commit manifest: {}", e))?;

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
        if !path.exists() {
            return Err(format!("Executable does not exist at {:?}", path));
        }

        // Ensure executable permissions on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(metadata) = fs::metadata(path) {
                let mut perms = metadata.permissions();
                if perms.mode() & 0o111 == 0 {
                    perms.set_mode(perms.mode() | 0o755);
                    let _ = fs::set_permissions(path, perms);
                }
            }
        }

        let version_arg = match tool_name.to_lowercase().as_str() {
            "yt-dlp" => "--version",
            "ffmpeg" => "-version",
            "ffprobe" => "-version",
            "mediainfo" => "--Version",
            _ => "--version",
        };

        let output = Command::new(path)
            .arg(version_arg)
            .output()
            .await
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

    /// Resolution Hierarchy:
    /// 1. Explicit configured path
    /// 2. Project-local development tool (e.g. ./bin, ./tools)
    /// 3. Bounded repository search (depth <= 3)
    /// 4. System PATH
    /// 5. Application-local managed tool
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
                if tool.path.exists() {
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

        // Priority 2: Project-local development tool (./bin, ./tools)
        for exe in &exe_names {
            let paths = [
                self.project_root.join(exe),
                self.project_root.join("bin").join(exe),
                self.project_root.join("tools").join(exe),
            ];
            for path in paths {
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
            }
        }

        // Priority 3: Application-local managed tool in tools/<tool>/<version>/
        if let Some(spec) = get_pinned_tool_spec(tool_name) {
            let version_dir = self.get_version_dir(tool_name, spec.pinned_version);
            for exe in &exe_names {
                let managed_path = version_dir.join(exe);
                if managed_path.exists() {
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

        // Priority 4: Bounded repository search (depth <= 3, e.g. youtube-downloader/)
        let mut candidates = Vec::new();
        let yt_dir = self.project_root.join("youtube-downloader");
        if yt_dir.exists() && yt_dir.is_dir() {
            Self::find_files_bounded(&yt_dir, &exe_names, 3, &mut candidates);
            for path in candidates {
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
            }
        }

        // Priority 5: System PATH (fallback when managed tool is not installed)
        if let Ok(path_var) = std::env::var("PATH") {
            let split_char = if cfg!(windows) { ';' } else { ':' };
            for dir in path_var.split(split_char) {
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

        let source_url = if cfg!(windows) {
            spec.windows_url
        } else if cfg!(target_os = "macos") {
            spec.darwin_url
        } else {
            spec.linux_url
        };

        let expected_sha256 = if cfg!(windows) {
            spec.windows_sha256
        } else if cfg!(target_os = "macos") {
            spec.darwin_sha256
        } else {
            spec.linux_sha256
        };

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
                    let is_valid = if spec.is_archive {
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
    pub async fn get_all_tools_health(&self) -> Vec<ToolHealth> {
        let statuses = self.get_all_tool_statuses(None).await;
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

        let source_url = if cfg!(windows) {
            spec.windows_url
        } else if cfg!(target_os = "macos") {
            spec.darwin_url
        } else {
            spec.linux_url
        };

        let expected_sha256 = if cfg!(windows) {
            spec.windows_sha256
        } else if cfg!(target_os = "macos") {
            spec.darwin_sha256
        } else {
            spec.linux_sha256
        };

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
            .user_agent("OneClickMediaDownloader/1.0 (Linux; x86_64)")
            .redirect(reqwest::redirect::Policy::limited(10))
            .build()
            .map_err(|e| format!("Failed to build HTTP client: {}", e))?;

        let response = client
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

        let bytes = response
            .bytes()
            .await
            .map_err(|e| format!("Failed to read response stream: {}", e))?;

        fs::write(&tmp_file_path, &bytes)
            .map_err(|e| format!("Failed to write staging file: {}", e))?;

        // Atomic Installation from verified file
        let install_res = self
            .atomic_install_from_staging(
                tool_name,
                &tmp_file_path,
                expected_sha256,
                spec.is_archive,
            )
            .await;

        // Cleanup staging file regardless of outcome
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
            .atomic_install_from_staging(tool_name, &tmp_file_path, expected_sha256, is_archive)
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
        is_archive: bool,
    ) -> Result<ToolStatusInfo, String> {
        let spec = get_pinned_tool_spec(tool_name)
            .ok_or_else(|| format!("Unknown tool specification: {}", tool_name))?;

        // 1. Verify Checksum
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

        // 2. Prepare Version Target Directory
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

        if is_archive {
            let mut extracted = false;

            // Try Zip extraction first
            if let Ok(file) = File::open(staged_file) {
                if let Ok(mut archive) = zip::ZipArchive::new(file) {
                    for i in 0..archive.len() {
                        if let Ok(mut zip_file) = archive.by_index(i) {
                            let enclosed = zip_file.enclosed_name().map(|p| p.to_owned());
                            if let Some(rel_path) = enclosed {
                                let file_name =
                                    rel_path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                                if file_name.eq_ignore_ascii_case(&final_bin_name) {
                                    if let Ok(mut out_file) = File::create(&staged_bin_path) {
                                        if std::io::copy(&mut zip_file, &mut out_file).is_ok() {
                                            extracted = true;
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // If not extracted via zip, try system tar on Unix for tar.xz / tar.gz / tar.bz2
            #[cfg(unix)]
            if !extracted {
                let tar_output = Command::new("tar")
                    .arg("-xf")
                    .arg(staged_file)
                    .arg("-C")
                    .arg(&staging_extract_dir)
                    .output()
                    .await;

                if let Ok(out) = tar_output {
                    if out.status.success() {
                        let mut found_paths = Vec::new();
                        Self::find_files_bounded(
                            &staging_extract_dir,
                            std::slice::from_ref(&final_bin_name),
                            5,
                            &mut found_paths,
                        );
                        if let Some(first_path) = found_paths.into_iter().next() {
                            if first_path != staged_bin_path {
                                let _ = fs::copy(&first_path, &staged_bin_path);
                            }
                            extracted = staged_bin_path.exists();
                        }
                    }
                }
            }

            if !extracted || !staged_bin_path.exists() {
                let _ = fs::remove_dir_all(&staging_extract_dir);
                return Err(format!(
                    "Binary '{}' could not be extracted from archive",
                    final_bin_name
                ));
            }
        } else {
            // Direct binary
            fs::copy(staged_file, &staged_bin_path)
                .map_err(|e| format!("Failed to copy binary to staging: {}", e))?;
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

        // 3. Executable Validation Check
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

        // 4. Move to version directory atomically
        fs::create_dir_all(&target_version_dir)
            .map_err(|e| format!("Failed to create target version dir: {}", e))?;

        let final_destination = target_version_dir.join(&final_bin_name);
        if final_destination.exists() {
            let _ = fs::remove_file(&final_destination);
        }

        fs::rename(&staged_bin_path, &final_destination)
            .or_else(|_| {
                fs::copy(&staged_bin_path, &final_destination)
                    .and_then(|_| fs::remove_file(&staged_bin_path))
            })
            .map_err(|e| {
                format!(
                    "Failed to activate binary into {:?}: {}",
                    final_destination, e
                )
            })?;

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

        // 5. Atomically update manifest.json
        let mut manifest = self.load_manifest();
        let bin_sha = Self::compute_sha256(&final_destination)
            .unwrap_or_else(|_| expected_sha256.to_string());

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
        self.save_manifest_atomic(&manifest)?;

        Ok(ToolStatusInfo {
            name: tool_name.to_string(),
            status: ToolStatus::Ready,
            version: Some(version_str),
            pinned_version: spec.pinned_version.to_string(),
            path: Some(final_destination.to_string_lossy().to_string()),
            managed: true,
            source_url: Some(
                if cfg!(windows) {
                    spec.windows_url
                } else {
                    spec.linux_url
                }
                .to_string(),
            ),
            sha256: Some(bin_sha),
            error_message: None,
            license: spec.license.to_string(),
            license_url: spec.license_url.to_string(),
            is_required: spec.is_required,
        })
    }

    /// Repair or reinstall a tool
    pub async fn repair_tool(&self, tool_name: &str) -> Result<ToolStatusInfo, String> {
        let spec = get_pinned_tool_spec(tool_name)
            .ok_or_else(|| format!("Unknown tool: {}", tool_name))?;

        self.diagnostics.log(
            "warn",
            "tool_manager",
            &format!("Initiating repair/reinstall for {}", tool_name),
        );

        // Remove active managed files if corrupted
        let version_dir = self.get_version_dir(tool_name, spec.pinned_version);
        if version_dir.exists() {
            let _ = fs::remove_dir_all(&version_dir);
        }

        let mut manifest = self.load_manifest();
        manifest.tools.remove(tool_name);
        let _ = self.save_manifest_atomic(&manifest);

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
        assert!(is_date_version_older("2024.12.31", "2025.02.19"));
        assert!(!is_date_version_older("2025.02.19", "2025.02.19"));
        assert!(!is_date_version_older("stable 2026.01.02", "2025.02.19"));
        assert!(!is_date_version_older("unknown", "2025.02.19"));
    }
}
