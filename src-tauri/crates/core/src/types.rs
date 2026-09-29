use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[cfg_attr(test, derive(ts_rs::TS))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SourceScope {
    #[default]
    SingleMedia,
}

#[cfg_attr(test, derive(ts_rs::TS))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(
    tag = "type",
    rename_all = "SCREAMING_SNAKE_CASE",
    rename_all_fields = "camelCase"
)]
pub enum AcquisitionOperation {
    #[default]
    EntireMedia,
    Clip {
        #[cfg_attr(test, ts(type = "number"))]
        start_ms: u64,
        #[cfg_attr(test, ts(type = "number"))]
        end_ms: u64,
    },
    AudioOnly,
    ThumbnailOnly,
    Chapter {
        chapter_index: u32,
    },
    SubtitlesOnly,
}

#[cfg_attr(test, derive(ts_rs::TS))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OutputProfile {
    BestSource,
    #[default]
    Universal,
    Editing,
    Small,
    Custom,
}

#[cfg_attr(test, derive(ts_rs::TS))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TrackSelection {
    #[cfg_attr(test, ts(optional = nullable))]
    pub audio_language: Option<String>,
    pub subtitle_languages: Vec<String>,
    pub include_auto_subtitles: bool,
}

#[cfg_attr(test, derive(ts_rs::TS))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DuplicatePolicy {
    #[default]
    Rename,
    Skip,
    Overwrite,
}

#[cfg_attr(test, derive(ts_rs::TS))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct MetadataPatch {
    #[cfg_attr(test, ts(optional = nullable))]
    pub title: Option<String>,
    #[cfg_attr(test, ts(optional = nullable))]
    pub artist: Option<String>,
    #[cfg_attr(test, ts(optional = nullable))]
    pub album: Option<String>,
}

/// Product-level intent. Execution arguments are deliberately not part of this contract.
#[cfg_attr(test, derive(ts_rs::TS))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AcquisitionRequest {
    pub source_scope: SourceScope,
    pub operation: AcquisitionOperation,
    pub output_profile: OutputProfile,
    pub track_selection: TrackSelection,
    #[cfg_attr(test, ts(optional = nullable))]
    pub metadata_patch: Option<MetadataPatch>,
    pub duplicate_policy: DuplicatePolicy,
    pub output_directory: String,
    /// Optional user constraint; `None` means the best suitable source height.
    #[cfg_attr(test, ts(optional = nullable))]
    pub max_video_height: Option<u32>,
}

#[cfg_attr(test, derive(ts_rs::TS))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceSummary {
    pub url: String,
    pub title: String,
    pub extractor: String,
    pub media_kind: MediaKind,
    #[cfg_attr(test, ts(optional = nullable))]
    pub duration_seconds: Option<f64>,
}

#[cfg_attr(test, derive(ts_rs::TS))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SelectedStreams {
    #[cfg_attr(test, ts(optional = nullable))]
    pub video_stream_id: Option<String>,
    #[cfg_attr(test, ts(optional = nullable))]
    pub audio_stream_id: Option<String>,
    pub subtitle_languages: Vec<String>,
}

#[cfg_attr(test, derive(ts_rs::TS))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlannedArtifact {
    pub container: String,
    #[cfg_attr(test, ts(optional = nullable))]
    pub video_codec: Option<String>,
    #[cfg_attr(test, ts(optional = nullable))]
    pub audio_codec: Option<String>,
    #[cfg_attr(test, ts(optional = nullable))]
    pub width: Option<u32>,
    #[cfg_attr(test, ts(optional = nullable))]
    pub height: Option<u32>,
    #[cfg_attr(test, ts(optional = nullable))]
    pub fps: Option<f64>,
    pub audio_only: bool,
}

#[cfg_attr(test, derive(ts_rs::TS))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ProcessingClass {
    #[default]
    SourcePreserved,
    MergeOnly,
    RemuxOnly,
    AudioTranscode,
    VideoTranscode,
    FullTranscode,
    Unknown,
}

#[cfg_attr(test, derive(ts_rs::TS))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessingPlan {
    pub class: ProcessingClass,
    pub requires_ffmpeg: bool,
    pub steps: Vec<String>,
}

#[cfg_attr(test, derive(ts_rs::TS))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SizeEstimate {
    #[cfg_attr(test, ts(type = "number"))]
    pub bytes: u64,
    pub confidence: String,
}

#[cfg_attr(test, derive(ts_rs::TS))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanWarning {
    pub code: String,
    pub message: String,
}

#[cfg_attr(test, derive(ts_rs::TS))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanRequirement {
    pub code: String,
    pub message: String,
}

#[cfg_attr(test, derive(ts_rs::TS))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AcquisitionPlan {
    pub transforms: Vec<PlannedTransform>,
    pub include_auto_subtitles: bool,
    pub post_process: PostProcessPolicy,
    #[cfg_attr(test, ts(optional = nullable))]
    #[cfg_attr(test, ts(type = "[number, number] | null"))]
    pub time_range_ms: Option<[u64; 2]>,
    pub id: String,
    pub version: u32,
    pub source: SourceSummary,
    pub scope: SourceScope,
    pub operation: AcquisitionOperation,
    pub output_profile: OutputProfile,
    pub selected_streams: SelectedStreams,
    pub output: PlannedArtifact,
    pub processing: ProcessingPlan,
    #[cfg_attr(test, ts(optional = nullable))]
    pub estimated_size: Option<SizeEstimate>,
    pub warnings: Vec<PlanWarning>,
    pub requirements: Vec<PlanRequirement>,
}

#[cfg_attr(test, derive(ts_rs::TS))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "type",
    rename_all = "SCREAMING_SNAKE_CASE",
    rename_all_fields = "camelCase"
)]
pub enum PlannedTransform {
    Merge,
    Remux { container: String },
    ExtractAudio { format: String },
    TranscodeVideo { codec: String },
    TranscodeAudio { codec: String },
    Trim { start_ms: u64, end_ms: u64 },
}

#[cfg_attr(test, derive(ts_rs::TS))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PostProcessPolicy {
    pub embed_metadata: bool,
    pub embed_thumbnail: bool,
    pub embed_chapters: bool,
    pub subtitle_mode: SubtitleMode,
    pub sponsor_block_mode: SponsorBlockMode,
}

impl From<&AppSettings> for PostProcessPolicy {
    fn from(settings: &AppSettings) -> Self {
        Self {
            embed_metadata: settings.embed_metadata,
            embed_thumbnail: settings.embed_thumbnail,
            embed_chapters: settings.embed_chapters,
            subtitle_mode: settings.subtitle_mode,
            sponsor_block_mode: settings.sponsor_block_mode,
        }
    }
}

#[cfg_attr(test, derive(ts_rs::TS))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MediaSourceType {
    #[default]
    YtDlpExtractor,
    YtDlpGeneric,
    DirectFile,
    Hls,
    Dash,
    Unsupported,
    Inaccessible,
}

#[cfg_attr(test, derive(ts_rs::TS))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DownloadStrategy {
    DirectCopy,
    YtDlpDownload,
    #[default]
    YtDlpMerge,
    FfmpegRemux,
    FfmpegTranscode,
    HlsDownload,
    DashDownload,
}

#[cfg_attr(test, derive(ts_rs::TS))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TranscodingCost {
    #[default]
    NoProcessing,
    StreamCopy,
    Remux,
    Merge,
    Transcode,
}

#[cfg_attr(test, derive(ts_rs::TS))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ResolverErrorCategory {
    #[default]
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

#[cfg_attr(test, derive(ts_rs::TS))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ResolverErrorDetail {
    pub category: ResolverErrorCategory,
    pub technical_message: String,
    pub user_friendly_message: String,
    #[cfg_attr(test, ts(optional = nullable))]
    pub http_status: Option<u16>,
}

#[cfg_attr(test, derive(ts_rs::TS))]
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

#[cfg_attr(test, derive(ts_rs::TS))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum PresetType {
    #[default]
    Mp4Compatible,
    BestVideo,
    BestAudio,
    Mp3,
    Flac,
}

#[cfg_attr(test, derive(ts_rs::TS))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MediaKind {
    Video,
    Audio,
    Livestream,
}

#[cfg_attr(test, derive(ts_rs::TS))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum SponsorBlockMode {
    #[default]
    Off,
    MarkChapters,
    RemoveSegments,
}

#[cfg_attr(test, derive(ts_rs::TS))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum SubtitleMode {
    #[default]
    None,
    Embed,
    DownloadSeparate,
}

#[cfg_attr(test, derive(ts_rs::TS))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SubtitleTrack {
    pub language: String,
    #[cfg_attr(test, ts(optional = nullable))]
    pub name: Option<String>,
    #[cfg_attr(test, ts(optional = nullable))]
    pub ext: Option<String>,
    #[cfg_attr(test, ts(optional = nullable))]
    pub is_auto: Option<bool>,
}

#[cfg_attr(test, derive(ts_rs::TS))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaChapter {
    pub title: String,
    pub start_time: f64,
    pub end_time: f64,
}

#[cfg_attr(test, derive(ts_rs::TS))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct MediaFormatSpec {
    #[cfg_attr(test, ts(optional = nullable))]
    pub language: Option<String>,
    pub format_id: String,
    pub ext: String,
    #[cfg_attr(test, ts(optional = nullable))]
    pub resolution: Option<String>,
    #[cfg_attr(test, ts(optional = nullable))]
    pub width: Option<u32>,
    #[cfg_attr(test, ts(optional = nullable))]
    pub height: Option<u32>,
    #[cfg_attr(test, ts(optional = nullable))]
    pub fps: Option<f64>,
    #[cfg_attr(test, ts(optional = nullable))]
    pub vcodec: Option<String>,
    #[cfg_attr(test, ts(optional = nullable))]
    pub acodec: Option<String>,
    #[cfg_attr(test, ts(optional = nullable))]
    #[cfg_attr(test, ts(type = "number | null"))]
    pub filesize: Option<u64>,
    #[cfg_attr(test, ts(optional = nullable))]
    #[cfg_attr(test, ts(type = "number | null"))]
    pub filesize_approx: Option<u64>,
    #[cfg_attr(test, ts(optional = nullable))]
    pub tbr: Option<f64>,
    #[cfg_attr(test, ts(optional = nullable))]
    pub vbr: Option<f64>,
    #[cfg_attr(test, ts(optional = nullable))]
    pub abr: Option<f64>,
    #[cfg_attr(test, ts(optional = nullable))]
    pub hdr: Option<bool>,
    #[cfg_attr(test, ts(optional = nullable))]
    pub dynamic_range: Option<String>,
    #[cfg_attr(test, ts(optional = nullable))]
    pub audio_sample_rate: Option<u32>,
    #[cfg_attr(test, ts(optional = nullable))]
    pub audio_channels: Option<u32>,
}

#[cfg_attr(test, derive(ts_rs::TS))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum UserIntent {
    MaxQuality,
    SmallestSize,
    BestCompatibility,
    #[default]
    Balanced,
}

#[cfg_attr(test, derive(ts_rs::TS))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RecommendationConstraints {
    #[cfg_attr(test, ts(optional = nullable))]
    #[cfg_attr(test, ts(type = "number | null"))]
    pub max_filesize_bytes: Option<u64>,
    #[cfg_attr(test, ts(optional = nullable))]
    pub min_height: Option<u32>,
    #[cfg_attr(test, ts(optional = nullable))]
    pub preferred_fps: Option<u32>,
    #[cfg_attr(test, ts(optional = nullable))]
    pub prefer_hdr: Option<bool>,
    pub avoid_transcoding: bool,
}

#[cfg_attr(test, derive(ts_rs::TS))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoStreamSpec {
    pub stream_id: String,
    pub codec: String,
    #[cfg_attr(test, ts(optional = nullable))]
    pub profile: Option<String>,
    pub width: u32,
    pub height: u32,
    #[cfg_attr(test, ts(optional = nullable))]
    pub fps: Option<f64>,
    #[cfg_attr(test, ts(optional = nullable))]
    #[cfg_attr(test, ts(type = "number | null"))]
    pub bitrate_kbps: Option<u64>,
    pub is_hdr: bool,
    #[cfg_attr(test, ts(optional = nullable))]
    pub dynamic_range: Option<String>,
    #[cfg_attr(test, ts(optional = nullable))]
    pub aspect_ratio: Option<String>,
    #[cfg_attr(test, ts(optional = nullable))]
    #[cfg_attr(test, ts(type = "number | null"))]
    pub filesize_approx: Option<u64>,
}

#[cfg_attr(test, derive(ts_rs::TS))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioStreamSpec {
    pub stream_id: String,
    pub codec: String,
    #[cfg_attr(test, ts(optional = nullable))]
    #[cfg_attr(test, ts(type = "number | null"))]
    pub bitrate_kbps: Option<u64>,
    #[cfg_attr(test, ts(optional = nullable))]
    pub sample_rate_hz: Option<u32>,
    #[cfg_attr(test, ts(optional = nullable))]
    pub channels: Option<u32>,
    #[cfg_attr(test, ts(optional = nullable))]
    pub language: Option<String>,
    pub is_default: bool,
    #[cfg_attr(test, ts(optional = nullable))]
    #[cfg_attr(test, ts(type = "number | null"))]
    pub filesize_approx: Option<u64>,
}

#[cfg_attr(test, derive(ts_rs::TS))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThumbnailSpec {
    pub url: String,
    #[cfg_attr(test, ts(optional = nullable))]
    pub width: Option<u32>,
    #[cfg_attr(test, ts(optional = nullable))]
    pub height: Option<u32>,
    #[cfg_attr(test, ts(optional = nullable))]
    pub id: Option<String>,
}

#[cfg_attr(test, derive(ts_rs::TS))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceMediaGraph {
    pub source_url: String,
    pub extractor: String,
    pub source_type: MediaSourceType,
    pub title: String,
    pub media_kind: MediaKind,
    #[cfg_attr(test, ts(optional = nullable))]
    pub duration_seconds: Option<f64>,
    pub video_streams: Vec<VideoStreamSpec>,
    pub audio_streams: Vec<AudioStreamSpec>,
    pub subtitle_streams: Vec<SubtitleTrack>,
    pub chapters: Vec<MediaChapter>,
    pub thumbnails: Vec<ThumbnailSpec>,
    pub formats: Vec<MediaFormatSpec>,
}

#[cfg_attr(test, derive(ts_rs::TS))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutputMediaArtifact {
    pub artifact_path: String,
    pub file_name: String,
    pub container: String,
    #[cfg_attr(test, ts(type = "number"))]
    pub file_size_bytes: u64,
    #[cfg_attr(test, ts(optional = nullable))]
    pub duration_seconds: Option<f64>,
    #[cfg_attr(test, ts(optional = nullable))]
    pub video_codec: Option<String>,
    #[cfg_attr(test, ts(optional = nullable))]
    pub audio_codec: Option<String>,
    #[cfg_attr(test, ts(optional = nullable))]
    pub width: Option<u32>,
    #[cfg_attr(test, ts(optional = nullable))]
    pub height: Option<u32>,
    #[cfg_attr(test, ts(optional = nullable))]
    pub fps: Option<f64>,
    pub is_verified: bool,
}

#[cfg_attr(test, derive(ts_rs::TS))]
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
    #[cfg_attr(test, ts(optional = nullable))]
    #[cfg_attr(test, ts(type = "number | null"))]
    pub estimated_size_bytes: Option<u64>,
    pub container: String,
    #[cfg_attr(test, ts(optional = nullable))]
    pub details: Option<String>,
}

#[cfg_attr(test, derive(ts_rs::TS))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaMetadata {
    pub id: String,
    pub title: String,
    #[cfg_attr(test, ts(optional = nullable))]
    pub uploader: Option<String>,
    #[cfg_attr(test, ts(optional = nullable))]
    pub uploader_avatar: Option<String>,
    #[cfg_attr(test, ts(optional = nullable))]
    pub channel_id: Option<String>,
    #[cfg_attr(test, ts(optional = nullable))]
    pub uploader_url: Option<String>,
    #[cfg_attr(test, ts(optional = nullable))]
    pub duration: Option<f64>,
    #[cfg_attr(test, ts(optional = nullable))]
    pub thumbnail: Option<String>,
    pub webpage_url: String,
    pub media_kind: MediaKind,
    #[cfg_attr(test, ts(optional = nullable))]
    pub upload_date: Option<String>,
    #[cfg_attr(test, ts(optional = nullable))]
    #[cfg_attr(test, ts(type = "number | null"))]
    pub release_timestamp: Option<i64>,
    #[cfg_attr(test, ts(optional = nullable))]
    #[cfg_attr(test, ts(type = "number | null"))]
    pub view_count: Option<u64>,
    #[cfg_attr(test, ts(optional = nullable))]
    #[cfg_attr(test, ts(type = "number | null"))]
    pub like_count: Option<u64>,
    #[cfg_attr(test, ts(optional = nullable))]
    pub description: Option<String>,
    #[cfg_attr(test, ts(optional = nullable))]
    pub categories: Option<Vec<String>>,
    #[cfg_attr(test, ts(optional = nullable))]
    pub tags: Option<Vec<String>>,
    #[cfg_attr(test, ts(optional = nullable))]
    pub language: Option<String>,
    #[cfg_attr(test, ts(optional = nullable))]
    pub is_live: Option<bool>,
    #[cfg_attr(test, ts(optional = nullable))]
    pub was_live: Option<bool>,
    #[cfg_attr(test, ts(optional = nullable))]
    pub extractor: Option<String>,
    #[cfg_attr(test, ts(optional = nullable))]
    pub extractor_key: Option<String>,
    #[cfg_attr(test, ts(optional = nullable))]
    pub playlist_title: Option<String>,
    #[cfg_attr(test, ts(optional = nullable))]
    pub playlist_index: Option<u32>,
    #[cfg_attr(test, ts(optional = nullable))]
    pub playlist_count: Option<u32>,
    pub available_resolutions: Vec<u32>,
    pub available_frame_rates: Vec<u32>,
    pub has_video: bool,
    pub has_audio: bool,
    #[cfg_attr(test, ts(optional = nullable))]
    pub is_hdr: Option<bool>,
    #[cfg_attr(test, ts(optional = nullable))]
    pub subtitles: Option<Vec<SubtitleTrack>>,
    #[cfg_attr(test, ts(optional = nullable))]
    pub automatic_captions: Option<Vec<SubtitleTrack>>,
    #[cfg_attr(test, ts(optional = nullable))]
    pub chapters: Option<Vec<MediaChapter>>,
    #[cfg_attr(test, ts(optional = nullable))]
    pub formats: Option<Vec<MediaFormatSpec>>,
    #[cfg_attr(test, ts(optional = nullable))]
    pub smart_recommendation: Option<FormatRecommendation>,
    #[cfg_attr(test, ts(optional = nullable))]
    pub source_type: Option<MediaSourceType>,
    #[cfg_attr(test, ts(optional = nullable))]
    pub strategy: Option<DownloadStrategy>,
    #[cfg_attr(test, ts(optional = nullable))]
    pub transcoding_cost: Option<TranscodingCost>,
    #[cfg_attr(test, ts(optional = nullable))]
    pub transcoding_explanation: Option<String>,
    #[cfg_attr(test, ts(optional = nullable))]
    pub capabilities: Option<MediaCapabilities>,
}

#[cfg_attr(test, derive(ts_rs::TS))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedMediaSource {
    pub source_type: MediaSourceType,
    #[cfg_attr(test, ts(optional = nullable))]
    pub extractor: Option<String>,
    #[cfg_attr(test, ts(optional = nullable))]
    pub extractor_key: Option<String>,
    pub webpage_url: String,
    pub title: String,
    pub media_kind: MediaKind,
    pub capabilities: MediaCapabilities,
    pub candidates: Vec<MediaFormatSpec>,
    pub strategy: DownloadStrategy,
    pub transcoding_cost: TranscodingCost,
    pub transcoding_explanation: String,
    #[cfg_attr(test, ts(optional = nullable))]
    pub metadata: Option<MediaMetadata>,
    #[cfg_attr(test, ts(optional = nullable))]
    pub error_detail: Option<ResolverErrorDetail>,
    pub is_resolved: bool,
}

#[cfg_attr(test, derive(ts_rs::TS))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DownloadStatus {
    #[default]
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

#[cfg_attr(test, derive(ts_rs::TS))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct DownloadProgress {
    pub percentage: f64,
    #[cfg_attr(test, ts(type = "number"))]
    pub downloaded_bytes: u64,
    #[cfg_attr(test, ts(type = "number"))]
    pub total_bytes: u64,
    pub speed_bytes_per_sec: f64,
    #[cfg_attr(test, ts(optional = nullable))]
    #[cfg_attr(test, ts(type = "number | null"))]
    pub eta_seconds: Option<u64>,
    pub current_speed: String,
    pub raw_status_line: String,
}

#[cfg_attr(test, derive(ts_rs::TS))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct MediaInspection {
    pub verification_level: VerificationLevel,
    pub container_format: String,
    #[cfg_attr(test, ts(optional = nullable))]
    pub video_codec: Option<String>,
    #[cfg_attr(test, ts(optional = nullable))]
    pub video_profile: Option<String>,
    #[cfg_attr(test, ts(optional = nullable))]
    pub audio_codec: Option<String>,
    #[cfg_attr(test, ts(optional = nullable))]
    pub width: Option<u32>,
    #[cfg_attr(test, ts(optional = nullable))]
    pub height: Option<u32>,
    #[cfg_attr(test, ts(optional = nullable))]
    pub fps: Option<f64>,
    #[cfg_attr(test, ts(optional = nullable))]
    pub bit_depth: Option<u32>,
    #[cfg_attr(test, ts(optional = nullable))]
    pub color_space: Option<String>,
    #[cfg_attr(test, ts(optional = nullable))]
    pub is_hdr: Option<bool>,
    #[cfg_attr(test, ts(optional = nullable))]
    #[cfg_attr(test, ts(type = "number | null"))]
    pub bitrate_kbps: Option<u64>,
    #[cfg_attr(test, ts(optional = nullable))]
    pub audio_channels: Option<u32>,
    #[cfg_attr(test, ts(optional = nullable))]
    pub audio_sample_rate_hz: Option<u32>,
    #[cfg_attr(test, ts(optional = nullable))]
    #[cfg_attr(test, ts(type = "number | null"))]
    pub audio_bitrate_kbps: Option<u64>,
    #[cfg_attr(test, ts(optional = nullable))]
    pub audio_language: Option<String>,
    #[cfg_attr(test, ts(type = "number"))]
    pub file_size_bytes: u64,
    #[cfg_attr(test, ts(optional = nullable))]
    pub duration_seconds: Option<f64>,
    pub is_lossy_transcode_warning: bool,
    #[cfg_attr(test, ts(optional = nullable))]
    pub stream_count: Option<u32>,
    #[cfg_attr(test, ts(optional = nullable))]
    pub chapters_count: Option<u32>,
}

#[cfg_attr(test, derive(ts_rs::TS))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum VerificationLevel {
    Verified,
    BasicInspection,
    #[default]
    Unverified,
}

#[cfg_attr(test, derive(ts_rs::TS))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceFingerprint {
    pub extractor: String,
    pub source_url: String,
    pub source_id: String,
    pub title: String,
}

#[cfg_attr(test, derive(ts_rs::TS))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamFingerprint {
    pub stream_type: String, // "video", "audio", "subtitle"
    pub codec: String,
    #[cfg_attr(test, ts(optional = nullable))]
    pub profile: Option<String>,
    #[cfg_attr(test, ts(optional = nullable))]
    pub dimensions_or_channels: Option<String>,
    pub rate: Option<String>, // fps or sample_rate
}

#[cfg_attr(test, derive(ts_rs::TS))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaFingerprint {
    pub canonical_id: String,
    pub source: SourceFingerprint,
    #[cfg_attr(test, ts(optional = nullable))]
    pub duration_seconds: Option<f64>,
    #[cfg_attr(test, ts(optional = nullable))]
    pub video_codec: Option<String>,
    #[cfg_attr(test, ts(optional = nullable))]
    pub audio_codec: Option<String>,
    #[cfg_attr(test, ts(optional = nullable))]
    pub max_resolution: Option<String>,
    pub stream_count: u32,
    pub streams: Vec<StreamFingerprint>,
    pub file_hash: Option<String>, // Calculated on-demand or explicit verification
    pub created_at: String,
}

#[cfg_attr(test, derive(ts_rs::TS))]
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

#[cfg_attr(test, derive(ts_rs::TS))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VerificationResult {
    pub plan_verification: PlanVerification,
    pub is_valid: bool,
    pub verification_level: VerificationLevel,
    pub checklist: VerificationChecklist,
    #[cfg_attr(test, ts(optional = nullable))]
    pub output_artifact: Option<OutputMediaArtifact>,
    #[cfg_attr(test, ts(optional = nullable))]
    pub fingerprint: Option<MediaFingerprint>,
}

#[cfg_attr(test, derive(ts_rs::TS))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanVerification {
    pub conforms: bool,
    pub mismatches: Vec<PlanMismatch>,
    pub warnings: Vec<String>,
}

#[cfg_attr(test, derive(ts_rs::TS))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanMismatch {
    pub field: String,
    pub planned: String,
    #[cfg_attr(test, ts(optional = nullable))]
    pub actual: Option<String>,
}

#[cfg_attr(test, derive(ts_rs::TS))]
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
    #[cfg_attr(test, ts(optional = nullable))]
    pub intent: Option<UserIntent>,
    #[cfg_attr(test, ts(optional = nullable))]
    pub verification: Option<VerificationChecklist>,
    pub timestamp: String,
    #[cfg_attr(test, ts(optional = nullable))]
    pub resulting_artifact_path: Option<String>,
}

#[cfg_attr(test, derive(ts_rs::TS))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExplainableResult {
    pub title: String,
    pub specs_label: String,
    pub why_reasons: Vec<String>,
    pub processing_summary: String,
    pub transcoding_cost: TranscodingCost,
    pub verification_checklist: VerificationChecklist,
    #[cfg_attr(test, ts(optional = nullable))]
    pub recipe_id: Option<String>,
}

#[cfg_attr(test, derive(ts_rs::TS))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubtitleOptions {
    pub mode: SubtitleMode,
    #[cfg_attr(test, ts(optional = nullable))]
    pub selected_language: Option<String>,
}

#[cfg_attr(test, derive(ts_rs::TS))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadJob {
    pub id: String,
    pub url: String,
    pub output_directory: String,
    pub status: DownloadStatus,
    pub progress: DownloadProgress,
    pub metadata: MediaMetadata,
    #[cfg_attr(test, ts(optional = nullable))]
    pub final_file_name: Option<String>,
    #[cfg_attr(test, ts(optional = nullable))]
    pub final_file_path: Option<String>,
    #[cfg_attr(test, ts(optional = nullable))]
    pub inspection: Option<MediaInspection>,
    #[cfg_attr(test, ts(optional = nullable))]
    pub error_message: Option<String>,
    pub created_at: String,
    #[cfg_attr(test, ts(optional = nullable))]
    pub completed_at: Option<String>,
    #[cfg_attr(test, ts(optional = nullable))]
    pub subtitle_options: Option<SubtitleOptions>,
    #[cfg_attr(test, ts(optional = nullable))]
    pub sponsor_block_mode: Option<SponsorBlockMode>,
    #[cfg_attr(test, ts(optional = nullable))]
    pub intent: Option<UserIntent>,
    #[cfg_attr(test, ts(optional = nullable))]
    pub recipe: Option<DownloadRecipe>,
    #[cfg_attr(test, ts(optional = nullable))]
    pub fingerprint: Option<MediaFingerprint>,
    #[cfg_attr(test, ts(optional = nullable))]
    pub explainable_result: Option<ExplainableResult>,
    #[cfg_attr(test, ts(optional = nullable))]
    pub verification: Option<VerificationResult>,
    pub acquisition_plan: AcquisitionPlan,
}

#[cfg_attr(test, derive(ts_rs::TS))]
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
    #[cfg_attr(test, ts(optional = nullable))]
    pub custom_ytdlp_path: Option<String>,
    #[cfg_attr(test, ts(optional = nullable))]
    pub custom_ffmpeg_path: Option<String>,
    #[cfg_attr(test, ts(optional = nullable))]
    pub custom_ffprobe_path: Option<String>,
    #[cfg_attr(test, ts(optional = nullable))]
    pub custom_mediainfo_path: Option<String>,
}

impl Default for AppSettings {
    fn default() -> Self {
        let default_dir = dirs::download_dir()
            .or_else(dirs::video_dir)
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
#[cfg_attr(test, derive(ts_rs::TS))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ToolStatus {
    Ready,
    #[default]
    Missing,
    Installing,
    Invalid,
    Outdated,
    Error,
}

/// Detailed status information for a tool
#[cfg_attr(test, derive(ts_rs::TS))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolStatusInfo {
    pub name: String,
    pub status: ToolStatus,
    #[cfg_attr(test, ts(optional = nullable))]
    pub version: Option<String>,
    pub pinned_version: String,
    #[cfg_attr(test, ts(optional = nullable))]
    pub path: Option<String>,
    pub managed: bool,
    #[cfg_attr(test, ts(optional = nullable))]
    pub source_url: Option<String>,
    #[cfg_attr(test, ts(optional = nullable))]
    pub sha256: Option<String>,
    #[cfg_attr(test, ts(optional = nullable))]
    pub error_message: Option<String>,
    pub license: String,
    pub license_url: String,
    pub is_required: bool,
}

/// Backward compatibility health structure
#[cfg_attr(test, derive(ts_rs::TS))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolHealth {
    pub name: String,
    pub available: bool,
    #[cfg_attr(test, ts(optional = nullable))]
    pub path: Option<String>,
    #[cfg_attr(test, ts(optional = nullable))]
    pub version: Option<String>,
    #[cfg_attr(test, ts(optional = nullable))]
    pub repair_message: Option<String>,
}

/// Entry stored inside the managed `manifest.json`
#[cfg_attr(test, derive(ts_rs::TS))]
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
#[cfg_attr(test, derive(ts_rs::TS))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ToolsManifest {
    pub schema_version: u32,
    pub tools: HashMap<String, ToolManifestEntry>,
    pub last_updated: String,
}

#[cfg_attr(test, derive(ts_rs::TS))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticLog {
    pub id: String,
    pub timestamp: String,
    pub level: String,
    pub source: String,
    pub message: String,
}

#[cfg_attr(test, derive(ts_rs::TS))]
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartDownloadRequest {
    pub expected_plan_id: String,
    pub metadata: MediaMetadata,
    pub acquisition: AcquisitionRequest,
}

#[cfg_attr(test, derive(ts_rs::TS))]
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildCommandRequest {
    pub metadata: MediaMetadata,
    pub acquisition: AcquisitionRequest,
    #[cfg_attr(test, ts(optional = nullable))]
    pub settings: Option<AppSettings>,
}

#[cfg_attr(test, derive(ts_rs::TS))]
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildCommandResponse {
    pub command: String,
    pub arguments: Vec<String>,
}

// ==========================================
// P2 Architecture Extensions & Domain Boundaries
// ==========================================

#[cfg_attr(test, derive(ts_rs::TS))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceItem {
    pub id: String,
    pub url: String,
    pub title: String,
    pub added_at: String,
    pub status: String,
    #[cfg_attr(test, ts(optional = nullable))]
    pub recipe_id: Option<String>,
    #[cfg_attr(test, ts(optional = nullable))]
    pub fingerprint_id: Option<String>,
}

#[cfg_attr(test, derive(ts_rs::TS))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct MediaWorkspace {
    pub id: String,
    pub name: String,
    pub items: Vec<WorkspaceItem>,
    pub created_at: String,
    pub updated_at: String,
}

#[cfg_attr(test, derive(ts_rs::TS))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DuplicateCandidate {
    pub source_id: String,
    pub candidate_id: String,
    pub match_confidence: f64,
    pub reason: String,
}

#[cfg_attr(test, derive(ts_rs::TS))]
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

#[cfg_attr(test, derive(ts_rs::TS))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchivePolicy {
    pub auto_deduplicate: bool,
    pub keep_highest_quality: bool,
    pub export_nfo_metadata: bool,
    pub embed_provenance_recipe: bool,
}

#[cfg(test)]
mod bindings {
    use super::*;
    use ts_rs::TS;

    #[test]
    fn ipc_bindings_are_current() {
        let declarations = [
            SourceScope::decl(),
            AcquisitionOperation::decl(),
            OutputProfile::decl(),
            TrackSelection::decl(),
            DuplicatePolicy::decl(),
            MetadataPatch::decl(),
            AcquisitionRequest::decl(),
            SourceSummary::decl(),
            SelectedStreams::decl(),
            PlannedArtifact::decl(),
            ProcessingClass::decl(),
            ProcessingPlan::decl(),
            SizeEstimate::decl(),
            PlanWarning::decl(),
            PlanRequirement::decl(),
            AcquisitionPlan::decl(),
            PlannedTransform::decl(),
            PostProcessPolicy::decl(),
            MediaSourceType::decl(),
            DownloadStrategy::decl(),
            TranscodingCost::decl(),
            ResolverErrorCategory::decl(),
            ResolverErrorDetail::decl(),
            MediaCapabilities::decl(),
            PresetType::decl(),
            MediaKind::decl(),
            SponsorBlockMode::decl(),
            SubtitleMode::decl(),
            SubtitleTrack::decl(),
            MediaChapter::decl(),
            MediaFormatSpec::decl(),
            UserIntent::decl(),
            RecommendationConstraints::decl(),
            VideoStreamSpec::decl(),
            AudioStreamSpec::decl(),
            ThumbnailSpec::decl(),
            SourceMediaGraph::decl(),
            OutputMediaArtifact::decl(),
            FormatRecommendation::decl(),
            MediaMetadata::decl(),
            ResolvedMediaSource::decl(),
            DownloadStatus::decl(),
            DownloadProgress::decl(),
            MediaInspection::decl(),
            VerificationLevel::decl(),
            SourceFingerprint::decl(),
            StreamFingerprint::decl(),
            MediaFingerprint::decl(),
            VerificationChecklist::decl(),
            VerificationResult::decl(),
            PlanVerification::decl(),
            PlanMismatch::decl(),
            DownloadRecipe::decl(),
            ExplainableResult::decl(),
            SubtitleOptions::decl(),
            DownloadJob::decl(),
            AppSettings::decl(),
            ToolStatus::decl(),
            ToolStatusInfo::decl(),
            ToolHealth::decl(),
            ToolManifestEntry::decl(),
            ToolsManifest::decl(),
            DiagnosticLog::decl(),
            StartDownloadRequest::decl(),
            BuildCommandRequest::decl(),
            BuildCommandResponse::decl(),
            WorkspaceItem::decl(),
            MediaWorkspace::decl(),
            DuplicateCandidate::decl(),
            MediaDiff::decl(),
            ArchivePolicy::decl(),
        ];
        let output = format!("// Generated from core/src/types.rs. Do not edit.\n// Regenerate: UPDATE_BINDINGS=1 cargo test --manifest-path src-tauri/crates/core/Cargo.toml ipc_bindings_are_current\n\n{}\n", declarations.iter().map(|d| format!("export {d}")).collect::<Vec<_>>().join("\n\n"));
        let path =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../src/generated/ipc.ts");
        if std::env::var_os("UPDATE_BINDINGS").is_some() {
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(&path, &output).unwrap();
        }
        assert_eq!(
            std::fs::read_to_string(path).expect("Generate IPC bindings first"),
            output,
            "Rust IPC types changed; regenerate TypeScript bindings"
        );
    }
}
