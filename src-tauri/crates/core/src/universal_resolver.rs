use std::sync::Arc;
use std::time::Duration;
use tokio::process::Command;
use tokio::time::timeout;

use crate::analyzer::parse_ytdlp_json;
use crate::tools::ToolResolver;
use crate::types::{
    DownloadStrategy, MediaCapabilities, MediaKind, MediaMetadata, MediaSourceType, PresetType,
    ResolvedMediaSource, ResolverErrorCategory, ResolverErrorDetail, TranscodingCost,
};
use crate::url_validator::validate_media_url;

pub struct UniversalResolver {
    tool_resolver: Arc<ToolResolver>,
}

impl UniversalResolver {
    pub fn new(tool_resolver: Arc<ToolResolver>) -> Self {
        Self { tool_resolver }
    }

    /// Primary entry point: Resolves any media URL into a structured ResolvedMediaSource
    pub async fn resolve(&self, url: &str) -> Result<ResolvedMediaSource, String> {
        // Step 1: Validate URL formatting and safety
        let validated_url = match validate_media_url(url) {
            Ok(u) => u,
            Err(e) => {
                return Ok(ResolvedMediaSource {
                    source_type: MediaSourceType::Unsupported,
                    extractor: None,
                    extractor_key: None,
                    webpage_url: url.to_string(),
                    title: "Invalid URL".to_string(),
                    media_kind: MediaKind::Video,
                    capabilities: MediaCapabilities::default(),
                    candidates: Vec::new(),
                    strategy: DownloadStrategy::DirectCopy,
                    transcoding_cost: TranscodingCost::NoProcessing,
                    transcoding_explanation: "URL is malformed or uses an unsupported protocol".to_string(),
                    metadata: None,
                    error_detail: Some(ResolverErrorDetail {
                        category: ResolverErrorCategory::InvalidUrl,
                        technical_message: e.to_string(),
                        user_friendly_message: "The provided URL is not valid. Please ensure it begins with http:// or https:// and points to a valid address.".to_string(),
                        http_status: None,
                    }),
                    is_resolved: false,
                });
            }
        };

        // Step 2: Identify URL characteristics (direct media, HLS, DASH, or multi-site)
        let lower_url = validated_url.to_lowercase();
        let is_direct_ext = lower_url.ends_with(".mp4")
            || lower_url.ends_with(".webm")
            || lower_url.ends_with(".mkv")
            || lower_url.ends_with(".mov")
            || lower_url.ends_with(".mp3")
            || lower_url.ends_with(".m4a")
            || lower_url.ends_with(".wav")
            || lower_url.ends_with(".flac")
            || lower_url.ends_with(".aac")
            || lower_url.ends_with(".ogg");

        let is_hls_ext = lower_url.contains(".m3u8") || lower_url.contains("m3u8?");
        let is_dash_ext = lower_url.contains(".mpd") || lower_url.contains("mpd?");

        // Step 3: Attempt yt-dlp multi-site extraction with an explicit 45s timeout policy
        let ytdlp_result = self.execute_ytdlp_extraction(&validated_url).await;

        let ytdlp_result = match ytdlp_result {
            Ok(meta) => Ok(meta),
            Err(e) => {
                if validated_url.contains("tiktok.com") {
                    self.extract_tiktok_direct(&validated_url).await.or(Err(e))
                } else {
                    Err(e)
                }
            }
        };

        match ytdlp_result {
            Ok(mut metadata) => {
                // Determine Source Type based on yt-dlp extractor output
                let raw_extractor = metadata
                    .extractor
                    .clone()
                    .unwrap_or_default()
                    .to_lowercase();
                let is_generic = raw_extractor == "generic" || raw_extractor.is_empty();

                let source_type = if is_hls_ext
                    || (metadata.formats.as_ref().is_some_and(|fmts| {
                        fmts.iter()
                            .any(|f| f.format_id.contains("hls") || f.ext == "m3u8")
                    })) {
                    MediaSourceType::Hls
                } else if is_dash_ext
                    || (metadata.formats.as_ref().is_some_and(|fmts| {
                        fmts.iter()
                            .any(|f| f.format_id.contains("dash") || f.ext == "mpd")
                    }))
                {
                    MediaSourceType::Dash
                } else if is_direct_ext {
                    MediaSourceType::DirectFile
                } else if is_generic {
                    MediaSourceType::YtDlpGeneric
                } else {
                    MediaSourceType::YtDlpExtractor
                };

                // Validate that generic extraction actually found media candidates
                let has_candidates = metadata.formats.as_ref().is_some_and(|f| !f.is_empty())
                    || metadata.has_video
                    || metadata.has_audio;

                if is_generic && !has_candidates {
                    return Ok(ResolvedMediaSource {
                        source_type: MediaSourceType::Unsupported,
                        extractor: Some("generic".to_string()),
                        extractor_key: Some("Generic".to_string()),
                        webpage_url: validated_url.clone(),
                        title: metadata.title.clone(),
                        media_kind: metadata.media_kind,
                        capabilities: MediaCapabilities::default(),
                        candidates: Vec::new(),
                        strategy: DownloadStrategy::DirectCopy,
                        transcoding_cost: TranscodingCost::NoProcessing,
                        transcoding_explanation: "Generic page analysis did not locate any downloadable media streams".to_string(),
                        metadata: None,
                        error_detail: Some(ResolverErrorDetail {
                            category: ResolverErrorCategory::NoMediaFound,
                            technical_message: "yt-dlp generic extractor succeeded with zero media stream candidates".to_string(),
                            user_friendly_message: "No playable or downloadable video/audio streams were detected on this webpage.".to_string(),
                            http_status: None,
                        }),
                        is_resolved: false,
                    });
                }

                // Build capabilities
                let capabilities = MediaCapabilities {
                    video: metadata.has_video,
                    audio: metadata.has_audio,
                    subtitles: metadata.subtitles.as_ref().is_some_and(|s| !s.is_empty())
                        || metadata
                            .automatic_captions
                            .as_ref()
                            .is_some_and(|s| !s.is_empty()),
                    chapters: metadata.chapters.as_ref().is_some_and(|c| !c.is_empty()),
                    thumbnails: metadata.thumbnail.is_some(),
                    metadata_embedding: true,
                    container_support: vec![
                        "mp4".to_string(),
                        "mkv".to_string(),
                        "webm".to_string(),
                        "mp3".to_string(),
                        "flac".to_string(),
                        "m4a".to_string(),
                    ],
                    transcoding_required: false,
                };

                // Select download strategy & evaluate transcoding cost
                let (strategy, cost, explanation) =
                    Self::evaluate_strategy(&source_type, &metadata, &PresetType::Mp4Compatible);

                metadata.source_type = Some(source_type);
                metadata.strategy = Some(strategy);
                metadata.transcoding_cost = Some(cost);
                metadata.transcoding_explanation = Some(explanation.clone());
                metadata.capabilities = Some(capabilities.clone());

                let candidates = metadata.formats.clone().unwrap_or_default();

                Ok(ResolvedMediaSource {
                    source_type,
                    extractor: metadata.extractor.clone(),
                    extractor_key: metadata.extractor_key.clone(),
                    webpage_url: metadata.webpage_url.clone(),
                    title: metadata.title.clone(),
                    media_kind: metadata.media_kind,
                    capabilities,
                    candidates,
                    strategy,
                    transcoding_cost: cost,
                    transcoding_explanation: explanation,
                    metadata: Some(metadata),
                    error_detail: None,
                    is_resolved: true,
                })
            }
            Err(err_msg) => {
                // Step 4: Check if direct file fallback is possible
                if is_direct_ext {
                    let raw_filename = validated_url
                        .split('/')
                        .next_back()
                        .unwrap_or("media_file")
                        .split('?')
                        .next()
                        .unwrap_or("media_file");

                    let clean_filename = raw_filename.replace("%20", " ").replace('+', " ");

                    let display_title = if let Some(dot_idx) = clean_filename.rfind('.') {
                        clean_filename[..dot_idx].trim().to_string()
                    } else {
                        clean_filename.trim().to_string()
                    };

                    let domain_uploader = url::Url::parse(&validated_url).ok().and_then(|u| {
                        u.host_str()
                            .map(|h| h.trim_start_matches("www.").to_string())
                    });

                    let is_audio = lower_url.ends_with(".mp3")
                        || lower_url.ends_with(".m4a")
                        || lower_url.ends_with(".wav")
                        || lower_url.ends_with(".flac")
                        || lower_url.ends_with(".aac")
                        || lower_url.ends_with(".ogg");

                    let kind = if is_audio {
                        MediaKind::Audio
                    } else {
                        MediaKind::Video
                    };

                    let metadata = MediaMetadata {
                        id: format!("direct_{:x}", md5_hash(&validated_url)),
                        title: display_title,
                        uploader: domain_uploader.clone(),
                        uploader_avatar: None,
                        channel_id: None,
                        uploader_url: None,
                        duration: None,
                        thumbnail: None,
                        webpage_url: validated_url.clone(),
                        media_kind: kind,
                        upload_date: None,
                        release_timestamp: None,
                        view_count: None,
                        like_count: None,
                        description: Some("Direct HTTP media file stream".to_string()),
                        categories: None,
                        tags: None,
                        language: None,
                        is_live: Some(false),
                        was_live: Some(false),
                        extractor: Some("direct".to_string()),
                        extractor_key: Some("DirectFile".to_string()),
                        playlist_title: None,
                        playlist_index: None,
                        playlist_count: None,
                        available_resolutions: if is_audio { Vec::new() } else { vec![1080] },
                        available_frame_rates: if is_audio { Vec::new() } else { vec![30] },
                        has_video: !is_audio,
                        has_audio: true,
                        is_hdr: Some(false),
                        subtitles: None,
                        automatic_captions: None,
                        chapters: None,
                        formats: None,
                        smart_recommendation: None,
                        source_type: Some(MediaSourceType::DirectFile),
                        strategy: Some(DownloadStrategy::DirectCopy),
                        transcoding_cost: Some(TranscodingCost::StreamCopy),
                        transcoding_explanation: Some(
                            "Fast · Direct stream copy (no transcoding)".to_string(),
                        ),
                        capabilities: Some(MediaCapabilities {
                            video: !is_audio,
                            audio: true,
                            subtitles: false,
                            chapters: false,
                            thumbnails: false,
                            metadata_embedding: true,
                            container_support: vec!["mp4".to_string(), "mp3".to_string()],
                            transcoding_required: false,
                        }),
                    };

                    return Ok(ResolvedMediaSource {
                        source_type: MediaSourceType::DirectFile,
                        extractor: Some("direct".to_string()),
                        extractor_key: Some("DirectFile".to_string()),
                        webpage_url: validated_url.clone(),
                        title: metadata.title.clone(),
                        media_kind: kind,
                        capabilities: metadata.capabilities.clone().unwrap(),
                        candidates: Vec::new(),
                        strategy: DownloadStrategy::DirectCopy,
                        transcoding_cost: TranscodingCost::StreamCopy,
                        transcoding_explanation: "Fast · Direct stream copy".to_string(),
                        metadata: Some(metadata),
                        error_detail: None,
                        is_resolved: true,
                    });
                }

                // Step 5: Categorize error with structured taxonomy
                let (category, friendly_msg) = Self::categorize_error(&err_msg);

                let source_type = match category {
                    ResolverErrorCategory::RequiresAuthentication
                    | ResolverErrorCategory::DrmProtected
                    | ResolverErrorCategory::NetworkUnreachable => MediaSourceType::Inaccessible,
                    _ => MediaSourceType::Unsupported,
                };

                Ok(ResolvedMediaSource {
                    source_type,
                    extractor: None,
                    extractor_key: None,
                    webpage_url: validated_url.clone(),
                    title: "Media Inaccessible".to_string(),
                    media_kind: MediaKind::Video,
                    capabilities: MediaCapabilities::default(),
                    candidates: Vec::new(),
                    strategy: DownloadStrategy::DirectCopy,
                    transcoding_cost: TranscodingCost::NoProcessing,
                    transcoding_explanation: "Source analysis could not retrieve media streams"
                        .to_string(),
                    metadata: None,
                    error_detail: Some(ResolverErrorDetail {
                        category,
                        technical_message: err_msg,
                        user_friendly_message: friendly_msg,
                        http_status: None,
                    }),
                    is_resolved: false,
                })
            }
        }
    }

    /// Evaluates the cheapest download pipeline strategy
    pub fn evaluate_strategy(
        source_type: &MediaSourceType,
        metadata: &MediaMetadata,
        requested_preset: &PresetType,
    ) -> (DownloadStrategy, TranscodingCost, String) {
        match source_type {
            MediaSourceType::DirectFile => (
                DownloadStrategy::DirectCopy,
                TranscodingCost::StreamCopy,
                "Original · Direct stream copy (no transcoding)".to_string(),
            ),
            MediaSourceType::Hls => (
                DownloadStrategy::HlsDownload,
                TranscodingCost::StreamCopy,
                "Fast · HLS stream demux & copy".to_string(),
            ),
            MediaSourceType::Dash => (
                DownloadStrategy::DashDownload,
                TranscodingCost::Remux,
                "Fast · DASH manifest remux".to_string(),
            ),
            MediaSourceType::YtDlpExtractor | MediaSourceType::YtDlpGeneric => {
                let is_audio_preset =
                    matches!(requested_preset, PresetType::Mp3 | PresetType::Flac);
                if is_audio_preset {
                    (
                        DownloadStrategy::FfmpegTranscode,
                        TranscodingCost::Transcode,
                        "Processing required · Audio transcode to requested container".to_string(),
                    )
                } else if metadata.has_video && metadata.has_audio {
                    (
                        DownloadStrategy::YtDlpMerge,
                        TranscodingCost::Merge,
                        "Fast · Multiplexing native video + audio streams".to_string(),
                    )
                } else {
                    (
                        DownloadStrategy::YtDlpDownload,
                        TranscodingCost::StreamCopy,
                        "Original · Direct download without re-encoding".to_string(),
                    )
                }
            }
            _ => (
                DownloadStrategy::DirectCopy,
                TranscodingCost::NoProcessing,
                "None".to_string(),
            ),
        }
    }

    /// Executes yt-dlp metadata extraction with a non-blocking timeout
    async fn execute_ytdlp_extraction(&self, url: &str) -> Result<MediaMetadata, String> {
        let ytdlp_tool = self
            .tool_resolver
            .resolve_tool("yt-dlp")
            .await
            .ok_or_else(|| "yt-dlp tool binary not available".to_string())?;

        let timeout_duration = Duration::from_secs(45);

        let process_future = async {
            Command::new(&ytdlp_tool.path)
                .arg("-J")
                .arg("--flat-playlist")
                .arg("--no-warnings")
                .arg("--user-agent")
                .arg("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0.0.0 Safari/537.36")
                .arg("--referer")
                .arg("https://www.google.com/")
                .arg(url)
                .output()
                .await
                .map_err(|e| format!("Failed to spawn yt-dlp: {}", e))
        };

        let output = timeout(timeout_duration, process_future)
            .await
            .map_err(|_| "Metadata extraction timed out after 45 seconds. The remote server may be slow or unresponsive.".to_string())??;

        if !output.status.success() {
            let stderr_err = String::from_utf8_lossy(&output.stderr);
            let first_err = stderr_err
                .lines()
                .find(|l| l.contains("ERROR:"))
                .unwrap_or_else(|| stderr_err.lines().next().unwrap_or("Analysis failed"))
                .trim();
            return Err(first_err.to_string());
        }

        let json_text = String::from_utf8_lossy(&output.stdout);
        parse_ytdlp_json(&json_text, url)
    }

    /// Direct fallback parser for TikTok when yt-dlp extractor is challenged or blocked
    async fn extract_tiktok_direct(&self, url: &str) -> Result<MediaMetadata, String> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(15))
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0.0.0 Safari/537.36")
            .build()
            .map_err(|e| format!("Failed to initialize HTTP client: {}", e))?;

        let resp = client
            .get(url)
            .header(
                "Accept",
                "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
            )
            .header("Accept-Language", "en-US,en;q=0.9")
            .header("Referer", "https://www.tiktok.com/")
            .send()
            .await
            .map_err(|e| format!("Failed to reach TikTok: {}", e))?;

        let html = resp
            .text()
            .await
            .map_err(|e| format!("Failed to read TikTok response: {}", e))?;

        let marker = "__UNIVERSAL_DATA_FOR_REHYDRATION__\" type=\"application/json\">";
        let start_pos = html
            .find(marker)
            .ok_or_else(|| "Could not locate TikTok rehydration metadata in page".to_string())?
            + marker.len();
        let end_pos = html[start_pos..]
            .find("</script>")
            .ok_or_else(|| "Malformed TikTok script payload".to_string())?
            + start_pos;
        let json_str = &html[start_pos..end_pos];

        let val: serde_json::Value = serde_json::from_str(json_str)
            .map_err(|e| format!("Failed to parse TikTok JSON: {}", e))?;
        let scope = val
            .get("__DEFAULT_SCOPE__")
            .ok_or_else(|| "Missing default scope in TikTok data".to_string())?;
        let detail = scope
            .get("webapp.video-detail")
            .ok_or_else(|| "Video detail not found in TikTok data".to_string())?;
        let item = detail
            .get("itemInfo")
            .and_then(|i| i.get("itemStruct"))
            .ok_or_else(|| {
                "Video struct not found (video may be private or removed)".to_string()
            })?;

        let id = item
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("tiktok_video")
            .to_string();
        let raw_desc = item
            .get("desc")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim();
        let author = item.get("author");
        let nickname = author
            .and_then(|a| a.get("nickname"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let unique_id = author
            .and_then(|a| a.get("uniqueId"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let avatar = author
            .and_then(|a| a.get("avatarThumb").or_else(|| a.get("avatarLarger")))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let video = item.get("video");
        let cover = video
            .and_then(|v| v.get("cover"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let duration = video
            .and_then(|v| v.get("duration"))
            .and_then(|v| v.as_f64());
        let play_addr = video
            .and_then(|v| v.get("playAddr"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let uploader_display = if !nickname.is_empty() && !unique_id.is_empty() {
            Some(format!("{} (@{})", nickname, unique_id))
        } else if !nickname.is_empty() {
            Some(nickname.to_string())
        } else if !unique_id.is_empty() {
            Some(format!("@{}", unique_id))
        } else {
            None
        };

        let title = if !raw_desc.is_empty() {
            raw_desc.to_string()
        } else if let Some(ref u) = uploader_display {
            format!("TikTok by {}", u)
        } else {
            "TikTok Video".to_string()
        };

        let format_spec = if !play_addr.is_empty() {
            Some(vec![crate::types::MediaFormatSpec {
                format_id: "direct_mp4".to_string(),
                ext: "mp4".to_string(),
                resolution: Some("1080p".to_string()),
                width: Some(1080),
                height: Some(1920),
                fps: Some(30.0),
                vcodec: Some("h264".to_string()),
                acodec: Some("aac".to_string()),
                filesize: None,
                filesize_approx: None,
                tbr: None,
                vbr: None,
                abr: None,
                hdr: Some(false),
                dynamic_range: None,
                audio_sample_rate: Some(44100),
                audio_channels: Some(2),
            }])
        } else {
            None
        };

        Ok(MediaMetadata {
            id,
            title,
            uploader: uploader_display,
            uploader_avatar: avatar,
            channel_id: if !unique_id.is_empty() {
                Some(unique_id.to_string())
            } else {
                None
            },
            uploader_url: if !unique_id.is_empty() {
                Some(format!("https://www.tiktok.com/@{}", unique_id))
            } else {
                None
            },
            duration,
            thumbnail: cover,
            webpage_url: url.to_string(),
            media_kind: MediaKind::Video,
            upload_date: None,
            release_timestamp: None,
            view_count: None,
            like_count: None,
            description: Some(raw_desc.to_string()),
            categories: None,
            tags: None,
            language: None,
            is_live: Some(false),
            was_live: Some(false),
            extractor: Some("tiktok".to_string()),
            extractor_key: Some("TikTok".to_string()),
            playlist_title: None,
            playlist_index: None,
            playlist_count: None,
            available_resolutions: vec![1080, 720],
            available_frame_rates: vec![30],
            has_video: true,
            has_audio: true,
            is_hdr: Some(false),
            subtitles: None,
            automatic_captions: None,
            chapters: None,
            formats: format_spec,
            smart_recommendation: None,
            source_type: Some(MediaSourceType::DirectFile),
            strategy: Some(DownloadStrategy::DirectCopy),
            transcoding_cost: Some(TranscodingCost::StreamCopy),
            transcoding_explanation: Some("Fast · Direct TikTok stream copy".to_string()),
            capabilities: Some(MediaCapabilities {
                video: true,
                audio: true,
                subtitles: false,
                chapters: false,
                thumbnails: true,
                metadata_embedding: true,
                container_support: vec!["mp4".to_string()],
                transcoding_required: false,
            }),
        })
    }

    /// Categorizes raw extractor errors into human-understandable categories
    fn categorize_error(err: &str) -> (ResolverErrorCategory, String) {
        let lower = err.to_lowercase();

        if lower.contains("drm")
            || lower.contains("protected")
            || lower.contains("encrypted")
            || lower.contains("widevine")
        {
            (
                ResolverErrorCategory::DrmProtected,
                "This media is protected by Digital Rights Management (DRM) encryption and cannot be downloaded.".to_string(),
            )
        } else if lower.contains("login")
            || lower.contains("sign in")
            || lower.contains("account")
            || lower.contains("private")
            || lower.contains("403")
            || lower.contains("401")
            || lower.contains("forbidden")
        {
            (
                ResolverErrorCategory::RequiresAuthentication,
                "This media is private or requires account sign-in. Downloading protected/private media without credentials is not supported.".to_string(),
            )
        } else if lower.contains("timeout") || lower.contains("timed out") {
            (
                ResolverErrorCategory::Timeout,
                "The remote media server took too long to respond. Please check your internet connection and try again.".to_string(),
            )
        } else if lower.contains("name or service not known")
            || lower.contains("getaddrinfo")
            || lower.contains("connection refused")
            || lower.contains("network unreachable")
        {
            (
                ResolverErrorCategory::NetworkUnreachable,
                "Unable to connect to the remote host. Please verify the URL or domain accessibility.".to_string(),
            )
        } else if lower.contains("unsupported url") || lower.contains("no media found") {
            (
                ResolverErrorCategory::NoMediaFound,
                "No downloadable media streams were detected at this address.".to_string(),
            )
        } else {
            (
                ResolverErrorCategory::ExtractorFailed,
                "Could not extract media information from this webpage. The source layout may have changed or contains no standard streams.".to_string(),
            )
        }
    }
}

fn md5_hash(input: &str) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    input.hash(&mut hasher);
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::MediaKind;

    #[test]
    fn test_scenario_b_direct_mp4() {
        let url =
            "https://commondatastorage.googleapis.com/gtv-videos-bucket/sample/BigBuckBunny.mp4";
        let validation = crate::url_validator::validate_url(url);
        assert!(validation.is_ok());

        let (strategy, cost, explanation) = UniversalResolver::evaluate_strategy(
            &MediaSourceType::DirectFile,
            &MediaMetadata {
                id: "test".to_string(),
                title: "BigBuckBunny.mp4".to_string(),
                uploader: None,
                uploader_avatar: None,
                channel_id: None,
                uploader_url: None,
                duration: Some(60.0),
                thumbnail: None,
                webpage_url: url.to_string(),
                media_kind: MediaKind::Video,
                upload_date: None,
                release_timestamp: None,
                view_count: None,
                like_count: None,
                description: None,
                categories: None,
                tags: None,
                language: None,
                is_live: Some(false),
                was_live: Some(false),
                extractor: Some("direct".to_string()),
                extractor_key: Some("DirectFile".to_string()),
                playlist_title: None,
                playlist_index: None,
                playlist_count: None,
                available_resolutions: vec![1080],
                available_frame_rates: vec![30],
                has_video: true,
                has_audio: true,
                is_hdr: Some(false),
                subtitles: None,
                automatic_captions: None,
                chapters: None,
                formats: None,
                smart_recommendation: None,
                source_type: Some(MediaSourceType::DirectFile),
                strategy: Some(DownloadStrategy::DirectCopy),
                transcoding_cost: Some(TranscodingCost::StreamCopy),
                transcoding_explanation: None,
                capabilities: None,
            },
            &PresetType::Mp4Compatible,
        );

        assert_eq!(strategy, DownloadStrategy::DirectCopy);
        assert_eq!(cost, TranscodingCost::StreamCopy);
        assert!(explanation.contains("Direct stream copy"));
    }

    #[test]
    fn test_scenario_c_direct_mp3() {
        let url = "https://example.com/audio/sample_podcast.mp3";
        let (strategy, cost, _) = UniversalResolver::evaluate_strategy(
            &MediaSourceType::DirectFile,
            &MediaMetadata {
                id: "test_audio".to_string(),
                title: "sample_podcast.mp3".to_string(),
                uploader: None,
                uploader_avatar: None,
                channel_id: None,
                uploader_url: None,
                duration: Some(120.0),
                thumbnail: None,
                webpage_url: url.to_string(),
                media_kind: MediaKind::Audio,
                upload_date: None,
                release_timestamp: None,
                view_count: None,
                like_count: None,
                description: None,
                categories: None,
                tags: None,
                language: None,
                is_live: Some(false),
                was_live: Some(false),
                extractor: Some("direct".to_string()),
                extractor_key: Some("DirectFile".to_string()),
                playlist_title: None,
                playlist_index: None,
                playlist_count: None,
                available_resolutions: Vec::new(),
                available_frame_rates: Vec::new(),
                has_video: false,
                has_audio: true,
                is_hdr: Some(false),
                subtitles: None,
                automatic_captions: None,
                chapters: None,
                formats: None,
                smart_recommendation: None,
                source_type: Some(MediaSourceType::DirectFile),
                strategy: Some(DownloadStrategy::DirectCopy),
                transcoding_cost: Some(TranscodingCost::StreamCopy),
                transcoding_explanation: None,
                capabilities: None,
            },
            &PresetType::BestAudio,
        );

        assert_eq!(strategy, DownloadStrategy::DirectCopy);
        assert_eq!(cost, TranscodingCost::StreamCopy);
    }

    #[test]
    fn test_scenario_d_hls_stream() {
        let (strategy, cost, _) = UniversalResolver::evaluate_strategy(
            &MediaSourceType::Hls,
            &MediaMetadata {
                id: "test_hls".to_string(),
                title: "HLS Stream".to_string(),
                uploader: None,
                uploader_avatar: None,
                channel_id: None,
                uploader_url: None,
                duration: None,
                thumbnail: None,
                webpage_url: "https://example.com/live/master.m3u8".to_string(),
                media_kind: MediaKind::Video,
                upload_date: None,
                release_timestamp: None,
                view_count: None,
                like_count: None,
                description: None,
                categories: None,
                tags: None,
                language: None,
                is_live: Some(true),
                was_live: Some(false),
                extractor: Some("hls".to_string()),
                extractor_key: Some("HLS".to_string()),
                playlist_title: None,
                playlist_index: None,
                playlist_count: None,
                available_resolutions: vec![1080],
                available_frame_rates: vec![60],
                has_video: true,
                has_audio: true,
                is_hdr: Some(false),
                subtitles: None,
                automatic_captions: None,
                chapters: None,
                formats: None,
                smart_recommendation: None,
                source_type: Some(MediaSourceType::Hls),
                strategy: Some(DownloadStrategy::HlsDownload),
                transcoding_cost: Some(TranscodingCost::StreamCopy),
                transcoding_explanation: None,
                capabilities: None,
            },
            &PresetType::BestVideo,
        );

        assert_eq!(strategy, DownloadStrategy::HlsDownload);
        assert_eq!(cost, TranscodingCost::StreamCopy);
    }

    #[test]
    fn test_scenario_e_dash_manifest() {
        let (strategy, cost, _) = UniversalResolver::evaluate_strategy(
            &MediaSourceType::Dash,
            &MediaMetadata {
                id: "test_dash".to_string(),
                title: "DASH Stream".to_string(),
                uploader: None,
                uploader_avatar: None,
                channel_id: None,
                uploader_url: None,
                duration: None,
                thumbnail: None,
                webpage_url: "https://example.com/manifest.mpd".to_string(),
                media_kind: MediaKind::Video,
                upload_date: None,
                release_timestamp: None,
                view_count: None,
                like_count: None,
                description: None,
                categories: None,
                tags: None,
                language: None,
                is_live: Some(false),
                was_live: Some(false),
                extractor: Some("dash".to_string()),
                extractor_key: Some("DASH".to_string()),
                playlist_title: None,
                playlist_index: None,
                playlist_count: None,
                available_resolutions: vec![1080],
                available_frame_rates: vec![30],
                has_video: true,
                has_audio: true,
                is_hdr: Some(false),
                subtitles: None,
                automatic_captions: None,
                chapters: None,
                formats: None,
                smart_recommendation: None,
                source_type: Some(MediaSourceType::Dash),
                strategy: Some(DownloadStrategy::DashDownload),
                transcoding_cost: Some(TranscodingCost::Remux),
                transcoding_explanation: None,
                capabilities: None,
            },
            &PresetType::BestVideo,
        );

        assert_eq!(strategy, DownloadStrategy::DashDownload);
        assert_eq!(cost, TranscodingCost::Remux);
    }

    #[test]
    fn test_scenario_error_categorization() {
        // DRM protected
        let (cat, msg) =
            UniversalResolver::categorize_error("ERROR: This video is DRM protected with Widevine");
        assert_eq!(cat, ResolverErrorCategory::DrmProtected);
        assert!(msg.contains("Digital Rights Management"));

        // Private / login required
        let (cat, msg) = UniversalResolver::categorize_error(
            "ERROR: Sign in to confirm your age / Private video",
        );
        assert_eq!(cat, ResolverErrorCategory::RequiresAuthentication);
        assert!(msg.contains("private or requires account sign-in"));

        // Network unreachable
        let (cat, _) = UniversalResolver::categorize_error(
            "ERROR: Name or service not known getaddrinfo failed",
        );
        assert_eq!(cat, ResolverErrorCategory::NetworkUnreachable);

        // Timeout
        let (cat, _) =
            UniversalResolver::categorize_error("ERROR: Connection timed out after 15000ms");
        assert_eq!(cat, ResolverErrorCategory::Timeout);

        // No media found
        let (cat, _) =
            UniversalResolver::categorize_error("ERROR: Unsupported URL: No video formats found");
        assert_eq!(cat, ResolverErrorCategory::NoMediaFound);
    }

    #[test]
    fn test_scenario_m_security_validation() {
        // file:// scheme rejection
        assert!(crate::url_validator::validate_url("file:///etc/passwd").is_err());

        // gopher:// scheme rejection
        assert!(crate::url_validator::validate_url("gopher://floodgap.com").is_err());

        // Private localhost IP rejection
        assert!(crate::url_validator::validate_url("http://127.0.0.1:8080/exploit.mp4").is_err());
        assert!(crate::url_validator::validate_url("http://localhost:3000/media.mp4").is_err());
        assert!(crate::url_validator::validate_url("http://192.168.1.1/stream.m3u8").is_err());
    }
}
