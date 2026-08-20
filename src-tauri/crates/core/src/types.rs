use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MediaSourceType {
    YtDlpExtractor,
    YtDlpGeneric,
    DirectFile,
    Hls,
    Dash,
    Unsupported,
    Inaccessible,
}

impl Default for MediaSourceType {
    fn default() -> Self {
        MediaSourceType::YtDlpExtractor
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DownloadStrategy {
    DirectCopy,
    YtDlpDownload,
    YtDlpMerge,
    FfmpegRemux,
    FfmpegTranscode,
    HlsDownload,
    DashDownload,
}

impl Default for DownloadStrategy {
    fn default() -> Self {
        DownloadStrategy::YtDlpMerge
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TranscodingCost {
    NoProcessing,
    StreamCopy,
    Remux,
    Merge,
    Transcode,
}

impl Default for TranscodingCost {
    fn default() -> Self {
        TranscodingCost::NoProcessing
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ResolverErrorCategory {
    None,
    InvalidUrl,
    UnsupportedProtocol,
    NoMediaFound,
    RequiresAuthentication,
    DrmProtected,
    NetworkUnreachable,
    Timeout,
    ExtractorFailed,
}

impl Default for ResolverErrorCategory {
    fn default() -> Self {
        ResolverErrorCategory::None
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ResolverErrorDetail {
    pub category: ResolverErrorCategory,
    pub technical_message: String,
    pub user_friendly_message: String,
    pub http_status: Option<u16>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct MediaCapabilities {
    pub video: bool,
    pub audio: bool,
    pub subtitles: bool,
    pub chapters: bool,
    pub thumbnails: bool,
    pub metadata_embedding: bool,
    pub container_support: Vec<String>,
    pub transcoding_required: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PresetType {
    Mp4Compatible,
    BestVideo,
    BestAudio,
    Mp3,
    Flac,
}

impl Default for PresetType {
    fn default() -> Self {
        PresetType::Mp4Compatible
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MediaKind {
    Video,
    Audio,
    Livestream,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SponsorBlockMode {
    Off,
    MarkChapters,
    RemoveSegments,
}

impl Default for SponsorBlockMode {
    fn default() -> Self {
        SponsorBlockMode::Off
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SubtitleMode {
    None,
    Embed,
    DownloadSeparate,
}

impl Default for SubtitleMode {
    fn default() -> Self {
        SubtitleMode::None
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SubtitleTrack {
    pub language: String,
    pub name: Option<String>,
    pub ext: Option<String>,
    pub is_auto: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaChapter {
    pub title: String,
    pub start_time: f64,
    pub end_time: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct MediaFormatSpec {
    pub format_id: String,
    pub ext: String,
    pub resolution: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub fps: Option<f64>,
    pub vcodec: Option<String>,
    pub acodec: Option<String>,
    pub filesize: Option<u64>,
    pub filesize_approx: Option<u64>,
    pub tbr: Option<f64>,
    pub vbr: Option<f64>,
    pub abr: Option<f64>,
    pub hdr: Option<bool>,
    pub dynamic_range: Option<String>,
    pub audio_sample_rate: Option<u32>,
    pub audio_channels: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum UserIntent {
    MaxQuality,
    SmallestSize,
    BestCompatibility,
    Balanced,
}

impl Default for UserIntent {
    fn default() -> Self {
        UserIntent::Balanced
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RecommendationConstraints {
    pub max_filesize_bytes: Option<u64>,
    pub min_height: Option<u32>,
    pub preferred_fps: Option<u32>,
    pub prefer_hdr: Option<bool>,
    pub avoid_transcoding: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoStreamSpec {
    pub stream_id: String,
    pub codec: String,
    pub profile: Option<String>,
    pub width: u32,
    pub height: u32,
    pub fps: f64,
    pub bitrate_kbps: Option<u64>,
    pub is_hdr: bool,
    pub dynamic_range: Option<String>,
    pub aspect_ratio: Option<String>,
    pub filesize_approx: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioStreamSpec {
    pub stream_id: String,
    pub codec: String,
    pub bitrate_kbps: Option<u64>,
    pub sample_rate_hz: Option<u32>,
    pub channels: Option<u32>,
    pub language: Option<String>,
    pub is_default: bool,
    pub filesize_approx: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThumbnailSpec {
    pub url: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceMediaGraph {
    pub source_url: String,
    pub extractor: String,
    pub source_type: MediaSourceType,
    pub title: String,
    pub media_kind: MediaKind,
    pub duration_seconds: Option<f64>,
    pub video_streams: Vec<VideoStreamSpec>,
    pub audio_streams: Vec<AudioStreamSpec>,
    pub subtitle_streams: Vec<SubtitleTrack>,
    pub chapters: Vec<MediaChapter>,
    pub thumbnails: Vec<ThumbnailSpec>,
    pub formats: Vec<MediaFormatSpec>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutputMediaArtifact {
    pub artifact_path: String,
    pub file_name: String,
    pub container: String,
    pub file_size_bytes: u64,
    pub duration_seconds: Option<f64>,
    pub video_codec: Option<String>,
    pub audio_codec: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub fps: Option<f64>,
    pub is_verified: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FormatRecommendation {
    pub preset: PresetType,
    pub label: String,
    pub target_quality: String,
    pub reason: String,
    pub why_reasons: Vec<String>,
    pub is_transcode_free: bool,
    pub transcoding_cost: TranscodingCost,
    pub estimated_size_bytes: Option<u64>,
    pub container: String,
    pub details: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaMetadata {
    pub id: String,
    pub title: String,
    pub uploader: Option<String>,
    pub channel_id: Option<String>,
    pub uploader_url: Option<String>,
    pub duration: Option<f64>,
    pub thumbnail: Option<String>,
    pub webpage_url: String,
    pub media_kind: MediaKind,
    pub upload_date: Option<String>,
    pub release_timestamp: Option<i64>,
    pub view_count: Option<u64>,
    pub like_count: Option<u64>,
    pub description: Option<String>,
    pub categories: Option<Vec<String>>,
    pub tags: Option<Vec<String>>,
    pub language: Option<String>,
    pub is_live: Option<bool>,
    pub was_live: Option<bool>,
    pub extractor: Option<String>,
    pub extractor_key: Option<String>,
    pub playlist_title: Option<String>,
    pub playlist_index: Option<u32>,
    pub playlist_count: Option<u32>,
    pub available_resolutions: Vec<u32>,
    pub available_frame_rates: Vec<u32>,
    pub has_video: bool,
    pub has_audio: bool,
    pub is_hdr: Option<bool>,
    pub subtitles: Option<Vec<SubtitleTrack>>,
    pub automatic_captions: Option<Vec<SubtitleTrack>>,
    pub chapters: Option<Vec<MediaChapter>>,
    pub formats: Option<Vec<MediaFormatSpec>>,
    pub smart_recommendation: Option<FormatRecommendation>,
    pub source_type: Option<MediaSourceType>,
    pub strategy: Option<DownloadStrategy>,
    pub transcoding_cost: Option<TranscodingCost>,
    pub transcoding_explanation: Option<String>,
    pub capabilities: Option<MediaCapabilities>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedMediaSource {
    pub source_type: MediaSourceType,
    pub extractor: Option<String>,
    pub extractor_key: Option<String>,
    pub webpage_url: String,
    pub title: String,
    pub media_kind: MediaKind,
    pub capabilities: MediaCapabilities,
    pub candidates: Vec<MediaFormatSpec>,
    pub strategy: DownloadStrategy,
    pub transcoding_cost: TranscodingCost,
    pub transcoding_explanation: String,
    pub metadata: Option<MediaMetadata>,
    pub error_detail: Option<ResolverErrorDetail>,
    pub is_resolved: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DownloadStatus {
    Idle,
    Analyzing,
    Ready,
    Downloading,
    PostProcessing,
    Verifying,
    Completed,
    Failed,
    Cancelling,
    Cancelled,
}

impl Default for DownloadStatus {
    fn default() -> Self {
        DownloadStatus::Idle
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct DownloadProgress {
    pub percentage: f64,
    pub downloaded_bytes: u64,
    pub total_bytes: u64,
    pub speed_bytes_per_sec: f64,
    pub eta_seconds: Option<u64>,
    pub current_speed: String,
    pub raw_status_line: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct MediaInspection {
    pub container_format: String,
    pub video_codec: Option<String>,
    pub video_profile: Option<String>,
    pub audio_codec: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub fps: Option<f64>,
    pub bit_depth: Option<u32>,
    pub color_space: Option<String>,
    pub is_hdr: Option<bool>,
    pub bitrate_kbps: Option<u64>,
    pub audio_channels: Option<u32>,
    pub audio_sample_rate_hz: Option<u32>,
    pub audio_bitrate_kbps: Option<u64>,
    pub audio_language: Option<String>,
    pub file_size_bytes: u64,
    pub duration_seconds: Option<f64>,
    pub is_lossy_transcode_warning: bool,
    pub stream_count: Option<u32>,
    pub chapters_count: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceFingerprint {
    pub extractor: String,
    pub source_url: String,
    pub source_id: String,
    pub title: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamFingerprint {
    pub stream_type: String, // "video", "audio", "subtitle"
    pub codec: String,
    pub profile: Option<String>,
    pub dimensions_or_channels: Option<String>,
    pub rate: Option<String>, // fps or sample_rate
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaFingerprint {
    pub canonical_id: String,
    pub source: SourceFingerprint,
    pub duration_seconds: Option<f64>,
    pub video_codec: Option<String>,
    pub audio_codec: Option<String>,
    pub max_resolution: Option<String>,
    pub stream_count: u32,
    pub streams: Vec<StreamFingerprint>,
    pub file_hash: Option<String>, // Calculated on-demand or explicit verification
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct VerificationChecklist {
    pub file_exists: bool,
    pub file_size_valid: bool,
    pub duration_valid: bool,
    pub video_stream_valid: bool,
    pub audio_stream_valid: bool,
    pub container_valid: bool,
    pub verified_at: String,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VerificationResult {
    pub is_valid: bool,
    pub checklist: VerificationChecklist,
    pub output_artifact: Option<OutputMediaArtifact>,
    pub fingerprint: Option<MediaFingerprint>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadRecipe {
    pub id: String,
    pub source_url: String,
    pub resolver_type: MediaSourceType,
    pub extractor: String,
    pub selected_candidates: Vec<String>,
    pub strategy: DownloadStrategy,
    pub output_container: String,
    pub transformations: Vec<String>,
    pub intent: Option<UserIntent>,
    pub verification: Option<VerificationChecklist>,
    pub timestamp: String,
    pub resulting_artifact_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExplainableResult {
    pub title: String,
    pub specs_label: String,
    pub why_reasons: Vec<String>,
    pub processing_summary: String,
    pub transcoding_cost: TranscodingCost,
    pub verification_checklist: VerificationChecklist,
    pub recipe_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubtitleOptions {
    pub mode: SubtitleMode,
    pub selected_language: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadJob {
    pub id: String,
    pub url: String,
    pub preset: PresetType,
    pub quality: String,
    pub output_directory: String,
    pub status: DownloadStatus,
    pub progress: DownloadProgress,
    pub metadata: MediaMetadata,
    pub final_file_name: Option<String>,
    pub final_file_path: Option<String>,
    pub inspection: Option<MediaInspection>,
    pub error_message: Option<String>,
    pub created_at: String,
    pub completed_at: Option<String>,
    pub subtitle_options: Option<SubtitleOptions>,
    pub sponsor_block_mode: Option<SponsorBlockMode>,
    pub intent: Option<UserIntent>,
    pub recipe: Option<DownloadRecipe>,
    pub fingerprint: Option<MediaFingerprint>,
    pub explainable_result: Option<ExplainableResult>,
    pub verification: Option<VerificationResult>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub download_directory: String,
    pub last_preset: PresetType,
    pub default_quality: String,
    pub open_folder_after_download: bool,
    pub auto_analyze_on_paste: bool,
    pub embed_metadata: bool,
    pub embed_thumbnail: bool,
    pub embed_chapters: bool,
    pub concurrent_fragments: u32,
    pub trim_filenames: u32,
    pub sponsor_block_mode: SponsorBlockMode,
    pub subtitle_mode: SubtitleMode,
    pub preferred_subtitle_language: String,
    pub custom_ytdlp_path: Option<String>,
    pub custom_ffmpeg_path: Option<String>,
    pub custom_ffprobe_path: Option<String>,
    pub custom_mediainfo_path: Option<String>,
}

impl Default for AppSettings {
    fn default() -> Self {
        let default_dir = dirs::download_dir()
            .or_else(dirs::video_dir())
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| {
                if cfg!(windows) {
                    "C:\\Downloads".to_string()
                } else {
                    "/tmp/downloads".to_string()
                }
            });

        AppSettings {
            download_directory: default_dir,
            last_preset: PresetType::Mp4Compatible,
            default_quality: "auto".to_string(),
            open_folder_after_download: false,
            auto_analyze_on_paste: true,
            embed_metadata: true,
            embed_thumbnail: true,
            embed_chapters: true,
            concurrent_fragments: 1,
            trim_filenames: 180,
            sponsor_block_mode: SponsorBlockMode::Off,
            subtitle_mode: SubtitleMode::None,
            preferred_subtitle_language: "en".to_string(),
            custom_ytdlp_path: None,
            custom_ffmpeg_path: None,
            custom_ffprobe_path: None,
            custom_mediainfo_path: None,
        }
    }
}

/// Tool health status enum matching the specification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ToolStatus {
    Ready,
    Missing,
    Installing,
    Invalid,
    Outdated,
    Error,
}

impl Default for ToolStatus {
    fn default() -> Self {
        ToolStatus::Missing
    }
}

/// Detailed status information for a tool
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolStatusInfo {
    pub name: String,
    pub status: ToolStatus,
    pub version: Option<String>,
    pub pinned_version: String,
    pub path: Option<String>,
    pub managed: bool,
    pub source_url: Option<String>,
    pub sha256: Option<String>,
    pub error_message: Option<String>,
    pub license: String,
    pub license_url: String,
    pub is_required: bool,
}

/// Backward compatibility health structure
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolHealth {
    pub name: String,
    pub available: bool,
    pub path: Option<String>,
    pub version: Option<String>,
    pub repair_message: Option<String>,
}

/// Entry stored inside the managed `manifest.json`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolManifestEntry {
    pub name: String,
    pub version: String,
    pub path: String,
    pub sha256: String,
    pub installed_at: String,
    pub verified: bool,
}

/// Application-local managed tools manifest
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ToolsManifest {
    pub schema_version: u32,
    pub tools: HashMap<String, ToolManifestEntry>,
    pub last_updated: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticLog {
    pub id: String,
    pub timestamp: String,
    pub level: String,
    pub source: String,
    pub message: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartDownloadRequest {
    pub url: String,
    pub metadata: MediaMetadata,
    pub preset: PresetType,
    pub quality: String,
    pub output_directory: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildCommandRequest {
    pub preset: PresetType,
    pub quality: String,
    pub output_directory: String,
    pub url: String,
    pub settings: Option<AppSettings>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildCommandResponse {
    pub command: String,
    pub arguments: Vec<String>,
}

// ==========================================
// P2 Architecture Extensions & Domain Boundaries
// ==========================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceItem {
    pub id: String,
    pub url: String,
    pub title: String,
    pub added_at: String,
    pub status: String,
    pub recipe_id: Option<String>,
    pub fingerprint_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct MediaWorkspace {
    pub id: String,
    pub name: String,
    pub items: Vec<WorkspaceItem>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DuplicateCandidate {
    pub source_id: String,
    pub candidate_id: String,
    pub match_confidence: f64,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaDiff {
    pub original_fingerprint: MediaFingerprint,
    pub downloaded_fingerprint: MediaFingerprint,
    pub duration_diff_seconds: f64,
    pub resolution_changed: bool,
    pub codec_changed: bool,
    pub container_changed: bool,
    pub is_exact_match: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchivePolicy {
    pub auto_deduplicate: bool,
    pub keep_highest_quality: bool,
    pub export_nfo_metadata: bool,
    pub embed_provenance_recipe: bool,
}
