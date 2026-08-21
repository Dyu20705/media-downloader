use serde_json::Value;
use std::sync::Arc;
use tokio::process::Command;
use crate::tools::ToolResolver;
use crate::types::{
    FormatRecommendation, MediaChapter, MediaFormatSpec, MediaKind, MediaMetadata, PresetType,
    SubtitleTrack, TranscodingCost,
};
use crate::url_validator::validate_media_url;

pub async fn analyze_media_metadata(
    url: &str,
    tool_resolver: &Arc<ToolResolver>,
) -> Result<MediaMetadata, String> {
    let valid_url = validate_media_url(url).map_err(|e| e.to_string())?;

    let ytdlp_tool = tool_resolver
        .resolve_tool("yt-dlp")
        .await
        .ok_or_else(|| "yt-dlp executable not found on system".to_string())?;

    // Execute yt-dlp with machine-readable JSON output
    let output = Command::new(&ytdlp_tool.path)
        .arg("-J")
        .arg("--flat-playlist")
        .arg("--no-warnings")
        .arg(&valid_url)
        .output()
        .await
        .map_err(|e| format!("Failed to spawn yt-dlp for analysis: {}", e))?;

    if !output.status.success() {
        let stderr_err = String::from_utf8_lossy(&output.stderr);
        let first_err = stderr_err
            .lines()
            .find(|l| l.contains("ERROR:"))
            .unwrap_or(stderr_err.lines().next().unwrap_or("Unknown analysis failure"))
            .trim();
        return Err(format!("yt-dlp analysis error: {}", first_err));
    }

    let json_text = String::from_utf8_lossy(&output.stdout);
    parse_ytdlp_json(&json_text, &valid_url)
}

pub fn parse_ytdlp_json(json_text: &str, original_url: &str) -> Result<MediaMetadata, String> {
    let root: Value = serde_json::from_str(json_text)
        .map_err(|e| format!("Failed to parse yt-dlp JSON output: {}", e))?;

    let id = root
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown_id")
        .to_string();

    let title = root
        .get("title")
        .and_then(|v| v.as_str())
        .unwrap_or("Untitled Media")
        .to_string();

    let uploader = root
        .get("uploader")
        .or_else(|| root.get("channel"))
        .or_else(|| root.get("artist"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let channel_id = root
        .get("channel_id")
        .or_else(|| root.get("uploader_id"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let uploader_url = root
        .get("uploader_url")
        .or_else(|| root.get("channel_url"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let duration = root.get("duration").and_then(|v| v.as_f64());

    let thumbnail = root
        .get("thumbnail")
        .or_else(|| {
            root.get("thumbnails")
                .and_then(|t| t.as_array())
                .and_then(|arr| arr.last())
                .and_then(|obj| obj.get("url"))
        })
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let webpage_url = root
        .get("webpage_url")
        .and_then(|v| v.as_str())
        .unwrap_or(original_url)
        .to_string();

    let upload_date = root
        .get("upload_date")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let release_timestamp = root.get("release_timestamp").and_then(|v| v.as_i64());

    let view_count = root.get("view_count").and_then(|v| v.as_u64());
    let like_count = root.get("like_count").and_then(|v| v.as_u64());

    let description = root
        .get("description")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let categories = root
        .get("categories")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|c| c.as_str().map(|s| s.to_string()))
                .collect()
        });

    let tags = root
        .get("tags")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|t| t.as_str().map(|s| s.to_string()))
                .collect()
        });

    let language = root
        .get("language")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let is_live = root
        .get("is_live")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let was_live = root.get("was_live").and_then(|v| v.as_bool());

    let extractor = root
        .get("extractor")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let extractor_key = root
        .get("extractor_key")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let playlist_title = root
        .get("playlist_title")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let playlist_index = root.get("playlist_index").and_then(|v| v.as_u64()).map(|u| u as u32);
    let playlist_count = root.get("playlist_count").and_then(|v| v.as_u64()).map(|u| u as u32);

    // Subtitles extraction
    let subtitles = parse_subtitles_map(root.get("subtitles"), false);
    let automatic_captions = parse_subtitles_map(root.get("automatic_captions"), true);

    // Chapters extraction
    let mut chapters: Option<Vec<MediaChapter>> = None;
    if let Some(ch_arr) = root.get("chapters").and_then(|v| v.as_array()) {
        let list: Vec<MediaChapter> = ch_arr
            .iter()
            .filter_map(|ch| {
                let start_time = ch.get("start_time").and_then(|v| v.as_f64())?;
                let end_time = ch.get("end_time").and_then(|v| v.as_f64()).unwrap_or(start_time);
                let ch_title = ch
                    .get("title")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Chapter")
                    .to_string();
                Some(MediaChapter {
                    title: ch_title,
                    start_time,
                    end_time,
                })
            })
            .collect();
        if !list.is_empty() {
            chapters = Some(list);
        }
    }

    // Formats & Resolutions extraction
    let mut heights: Vec<u32> = Vec::new();
    let mut frame_rates: Vec<u32> = Vec::new();
    let mut has_video = false;
    let mut has_audio = false;
    let mut is_hdr = false;
    let mut parsed_formats: Vec<MediaFormatSpec> = Vec::new();

    if let Some(formats) = root.get("formats").and_then(|f| f.as_array()) {
        for fmt in formats {
            let format_id = fmt
                .get("format_id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let ext = fmt
                .get("ext")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let resolution = fmt
                .get("resolution")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let width = fmt.get("width").and_then(|v| v.as_u64()).map(|u| u as u32);
            let height = fmt.get("height").and_then(|v| v.as_u64()).map(|u| u as u32);
            let fps = fmt.get("fps").and_then(|v| v.as_f64());
            let vcodec = fmt
                .get("vcodec")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let acodec = fmt
                .get("acodec")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let filesize = fmt.get("filesize").and_then(|v| v.as_u64());
            let filesize_approx = fmt.get("filesize_approx").and_then(|v| v.as_u64());
            let tbr = fmt.get("tbr").and_then(|v| v.as_f64());
            let vbr = fmt.get("vbr").and_then(|v| v.as_f64());
            let abr = fmt.get("abr").and_then(|v| v.as_f64());
            let dynamic_range = fmt
                .get("dynamic_range")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let audio_sample_rate = fmt.get("asr").and_then(|v| v.as_u64()).map(|u| u as u32);
            let audio_channels = fmt
                .get("audio_channels")
                .and_then(|v| v.as_u64())
                .map(|u| u as u32);

            let fmt_is_hdr = dynamic_range
                .as_ref()
                .map(|dr| {
                    let d = dr.to_lowercase();
                    d.contains("hdr") || d.contains("hlg") || d.contains("dv") || d.contains("10")
                })
                .unwrap_or(false);

            if fmt_is_hdr {
                is_hdr = true;
            }

            if let Some(ref vc) = vcodec {
                if vc != "none" {
                    has_video = true;
                    if let Some(h) = height {
                        if h > 0 && h <= 8640 {
                            heights.push(h);
                        }
                    }
                    if let Some(f) = fps {
                        let rounded_fps = f.round() as u32;
                        if rounded_fps >= 10 && rounded_fps <= 240 {
                            frame_rates.push(rounded_fps);
                        }
                    }
                }
            }

            if let Some(ref ac) = acodec {
                if ac != "none" {
                    has_audio = true;
                }
            }

            parsed_formats.push(MediaFormatSpec {
                format_id,
                ext,
                resolution,
                width,
                height,
                fps,
                vcodec,
                acodec,
                filesize,
                filesize_approx,
                tbr,
                vbr,
                abr,
                hdr: Some(fmt_is_hdr),
                dynamic_range,
                audio_sample_rate,
                audio_channels,
            });
        }
    }

    // Sort descending and dedup resolutions
    heights.sort_unstable();
    heights.dedup();
    heights.reverse();

    // Sort descending and dedup framerates
    frame_rates.sort_unstable();
    frame_rates.dedup();
    frame_rates.reverse();

    let media_kind = if is_live {
        MediaKind::Livestream
    } else if has_video || !heights.is_empty() {
        MediaKind::Video
    } else if has_audio {
        MediaKind::Audio
    } else {
        MediaKind::Video
    };

    // Calculate smart recommendation
    let smart_recommendation = generate_smart_recommendation(
        media_kind,
        &heights,
        &frame_rates,
        is_hdr,
        &parsed_formats,
    );

    Ok(MediaMetadata {
        id,
        title,
        uploader,
        channel_id,
        uploader_url,
        duration,
        thumbnail,
        webpage_url,
        media_kind,
        upload_date,
        release_timestamp,
        view_count,
        like_count,
        description,
        source_type: None,
        strategy: None,
        transcoding_cost: None,
        transcoding_explanation: None,
        capabilities: None,
        categories,
        tags,
        language,
        is_live: Some(is_live),
        was_live,
        extractor,
        extractor_key,
        playlist_title,
        playlist_index,
        playlist_count,
        available_resolutions: heights,
        available_frame_rates: frame_rates,
        has_video,
        has_audio,
        is_hdr: Some(is_hdr),
        subtitles,
        automatic_captions,
        chapters,
        formats: if parsed_formats.is_empty() {
            None
        } else {
            Some(parsed_formats)
        },
        smart_recommendation,
    })
}

fn parse_subtitles_map(value: Option<&Value>, is_auto: bool) -> Option<Vec<SubtitleTrack>> {
    let map = value.and_then(|v| v.as_object())?;
    let mut tracks: Vec<SubtitleTrack> = Vec::new();

    for (lang, list_val) in map {
        let name = list_val
            .as_array()
            .and_then(|arr| arr.first())
            .and_then(|obj| obj.get("name"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let ext = list_val
            .as_array()
            .and_then(|arr| arr.first())
            .and_then(|obj| obj.get("ext"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        tracks.push(SubtitleTrack {
            language: lang.clone(),
            name,
            ext,
            is_auto: Some(is_auto),
        });
    }

    if tracks.is_empty() {
        None
    } else {
        // Sort alphabetically by language code
        tracks.sort_by(|a, b| a.language.cmp(&b.language));
        Some(tracks)
    }
}

pub fn generate_smart_recommendation(
    media_kind: MediaKind,
    resolutions: &[u32],
    frame_rates: &[u32],
    is_hdr: bool,
    formats: &[MediaFormatSpec],
) -> Option<FormatRecommendation> {
    if media_kind == MediaKind::Audio {
        return Some(FormatRecommendation {
            preset: PresetType::BestAudio,
            label: "Best Audio (Source Copy)".to_string(),
            target_quality: "auto".to_string(),
            reason: "Extracts untouched source audio stream (Opus/AAC) without lossy transcoding".to_string(),
            is_transcode_free: true,
            details: Some("Direct container copy".to_string()),
            why_reasons: vec![],
            transcoding_cost: TranscodingCost::NoProcessing,
            estimated_size_bytes: None,
            container: "m4a/opus".to_string(),
        });
    }

    let max_res = resolutions.first().copied().unwrap_or(1080);
    let max_fps = frame_rates.first().copied().unwrap_or(30);

    // Check if high-resolution (4K/8K) or HDR
    if max_res >= 1440 || is_hdr {
        let fps_label = if max_fps > 30 { format!("{}fps ", max_fps) } else { "".to_string() };
        let hdr_label = if is_hdr { " HDR" } else { "" };
        return Some(FormatRecommendation {
            preset: PresetType::BestVideo,
            label: format!("Best Video · {}p{}{}", max_res, fps_label, hdr_label),
            target_quality: max_res.to_string(),
            reason: "Preserves pristine source bitrates, wide-color HDR, and modern VP9/AV1 codecs in MKV".to_string(),
            is_transcode_free: true,
            details: Some("No video transcoding required".to_string()),
            why_reasons: vec![],
            transcoding_cost: TranscodingCost::Merge,
            estimated_size_bytes: None,
            container: "mkv".to_string(),
        });
    }

    // Standard video (1080p, 720p, etc.)
    // Check if native MP4/H264 stream exists
    let has_native_mp4_h264 = formats.iter().any(|f| {
        let vc = f.vcodec.as_deref().unwrap_or("");
        let ext = f.ext.as_str();
        (ext == "mp4" || vc.starts_with("avc1") || vc.starts_with("h264")) && f.height.unwrap_or(0) >= max_res
    });

    let fps_suffix = if max_fps > 30 { format!("{} ", max_fps) } else { "".to_string() };

    Some(FormatRecommendation {
        preset: PresetType::Mp4Compatible,
        label: format!("MP4 · {}p{}", max_res, fps_suffix),
        target_quality: max_res.to_string(),
        reason: if has_native_mp4_h264 {
            "Direct stream copy with broad playback compatibility on phones, tablets, and TVs".to_string()
        } else {
            "Broad compatibility across all modern video players and devices".to_string()
        },
        is_transcode_free: has_native_mp4_h264,
        details: Some("Universal H.264 / AAC MP4 container".to_string()),
        why_reasons: vec![],
        transcoding_cost: if has_native_mp4_h264 { TranscodingCost::Merge } else { TranscodingCost::Transcode },
        estimated_size_bytes: None,
        container: "mp4".to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ytdlp_json_full() {
        let sample_json = r#"{
            "id": "LXb3EKWsInQ",
            "title": "COSTA RICA IN 4K 60fps HDR",
            "uploader": "Jacob + Katie Schwarz",
            "channel_id": "UC_channel_123",
            "uploader_url": "https://www.youtube.com/@jacobkatieschwarz",
            "duration": 314.5,
            "thumbnail": "https://i.ytimg.com/vi/LXb3EKWsInQ/maxresdefault.jpg",
            "webpage_url": "https://www.youtube.com/watch?v=LXb3EKWsInQ",
            "upload_date": "20151123",
            "view_count": 89000000,
            "like_count": 450000,
            "description": "Filmed in Costa Rica in Ultra HD 4K 60 FPS HDR.",
            "categories": ["Travel & Events"],
            "tags": ["Costa Rica", "4K", "HDR", "60fps"],
            "language": "en",
            "is_live": false,
            "extractor": "youtube",
            "subtitles": {
                "en": [{ "name": "English", "ext": "vtt" }],
                "es": [{ "name": "Spanish", "ext": "vtt" }]
            },
            "automatic_captions": {
                "ja": [{ "name": "Japanese (auto)", "ext": "vtt" }]
            },
            "chapters": [
                { "start_time": 0.0, "end_time": 45.0, "title": "Introduction" },
                { "start_time": 45.0, "end_time": 180.0, "title": "Rainforest Wildlife" },
                { "start_time": 180.0, "end_time": 314.5, "title": "Pacific Coastline" }
            ],
            "formats": [
                { "format_id": "137", "vcodec": "avc1.64002a", "acodec": "none", "height": 1080, "fps": 60.0, "ext": "mp4" },
                { "format_id": "313", "vcodec": "vp9", "acodec": "none", "height": 2160, "fps": 60.0, "ext": "webm", "dynamic_range": "HDR" },
                { "format_id": "271", "vcodec": "vp9", "acodec": "none", "height": 1440, "fps": 60.0, "ext": "webm" },
                { "format_id": "140", "vcodec": "none", "acodec": "mp4a.40.2", "asr": 44100, "audio_channels": 2, "ext": "m4a" },
                { "format_id": "136", "vcodec": "avc1.4d401f", "acodec": "none", "height": 720, "fps": 30.0, "ext": "mp4" }
            ]
        }"#;

        let meta = parse_ytdlp_json(sample_json, "https://www.youtube.com/watch?v=LXb3EKWsInQ").unwrap();
        assert_eq!(meta.id, "LXb3EKWsInQ");
        assert_eq!(meta.title, "COSTA RICA IN 4K 60fps HDR");
        assert_eq!(meta.uploader, Some("Jacob + Katie Schwarz".to_string()));
        assert_eq!(meta.channel_id, Some("UC_channel_123".to_string()));
        assert_eq!(meta.duration, Some(314.5));
        assert_eq!(meta.media_kind, MediaKind::Video);
        assert_eq!(meta.available_resolutions, vec![2160, 1440, 1080, 720]);
        assert_eq!(meta.available_frame_rates, vec![60, 30]);
        assert!(meta.has_video);
        assert!(meta.has_audio);
        assert_eq!(meta.is_hdr, Some(true));

        // Subtitles
        let subs = meta.subtitles.unwrap();
        assert_eq!(subs.len(), 2);
        assert_eq!(subs[0].language, "en");

        let auto_subs = meta.automatic_captions.unwrap();
        assert_eq!(auto_subs.len(), 1);
        assert_eq!(auto_subs[0].language, "ja");

        // Chapters
        let ch = meta.chapters.unwrap();
        assert_eq!(ch.len(), 3);
        assert_eq!(ch[0].title, "Introduction");
        assert_eq!(ch[1].start_time, 45.0);

        // Smart recommendation
        let rec = meta.smart_recommendation.unwrap();
        assert_eq!(rec.preset, PresetType::BestVideo);
        assert!(rec.is_transcode_free);
    }

    #[test]
    fn test_parse_missing_fields_no_fakes() {
        let minimal_json = r#"{
            "id": "abc123xyz",
            "title": "Minimal Video"
        }"#;

        let meta = parse_ytdlp_json(minimal_json, "https://example.com/video").unwrap();
        assert_eq!(meta.id, "abc123xyz");
        assert_eq!(meta.title, "Minimal Video");
        assert_eq!(meta.uploader, None);
        assert_eq!(meta.duration, None);
        assert_eq!(meta.thumbnail, None);
        assert_eq!(meta.view_count, None);
        assert_eq!(meta.subtitles, None);
        assert_eq!(meta.chapters, None);
    }

    #[test]
    fn test_parse_livestream() {
        let live_json = r#"{
            "id": "live_stream_1",
            "title": "Live 24/7 News Stream",
            "is_live": true,
            "uploader": "News Corp"
        }"#;

        let meta = parse_ytdlp_json(live_json, "https://example.com/live").unwrap();
        assert_eq!(meta.media_kind, MediaKind::Livestream);
        assert_eq!(meta.is_live, Some(true));
        assert_eq!(meta.duration, None);
    }
}
