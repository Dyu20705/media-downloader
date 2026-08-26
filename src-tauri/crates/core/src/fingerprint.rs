use crate::types::{
    MediaFingerprint, MediaInspection, MediaMetadata, SourceFingerprint, StreamFingerprint,
};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub struct FingerprintEngine;

impl FingerprintEngine {
    /// Generates structured multi-level media fingerprint from metadata & inspection
    pub fn generate(
        metadata: &MediaMetadata,
        inspection: Option<&MediaInspection>,
        file_hash: Option<String>,
    ) -> MediaFingerprint {
        let source_fp = SourceFingerprint {
            extractor: metadata.extractor.clone().unwrap_or_else(|| "generic".to_string()),
            source_url: metadata.webpage_url.clone(),
            source_id: metadata.id.clone(),
            title: metadata.title.clone(),
        };

        let mut streams: Vec<StreamFingerprint> = Vec::new();

        if let Some(insp) = inspection {
            if let Some(vcodec) = &insp.video_codec {
                streams.push(StreamFingerprint {
                    stream_type: "video".to_string(),
                    codec: vcodec.clone(),
                    profile: insp.video_profile.clone(),
                    dimensions_or_channels: match (insp.width, insp.height) {
                        (Some(w), Some(h)) => Some(format!("{}x{}", w, h)),
                        _ => None,
                    },
                    rate: insp.fps.map(|f| format!("{:.2} fps", f)),
                });
            }

            if let Some(acodec) = &insp.audio_codec {
                streams.push(StreamFingerprint {
                    stream_type: "audio".to_string(),
                    codec: acodec.clone(),
                    profile: None,
                    dimensions_or_channels: insp.audio_channels.map(|c| format!("{} ch", c)),
                    rate: insp.audio_sample_rate_hz.map(|hz| format!("{} Hz", hz)),
                });
            }
        }

        // Canonical media identity hash calculation
        let mut hasher = DefaultHasher::new();
        source_fp.extractor.hash(&mut hasher);
        source_fp.source_id.hash(&mut hasher);
        if let Some(d) = metadata.duration {
            (d as u64).hash(&mut hasher);
        }
        for s in &streams {
            s.stream_type.hash(&mut hasher);
            s.codec.hash(&mut hasher);
        }
        let canonical_id = format!("mfp_{:016x}", hasher.finish());

        MediaFingerprint {
            canonical_id,
            source: source_fp,
            duration_seconds: metadata.duration,
            video_codec: inspection.and_then(|i| i.video_codec.clone()),
            audio_codec: inspection.and_then(|i| i.audio_codec.clone()),
            max_resolution: inspection.and_then(|i| match (i.width, i.height) {
                (Some(w), Some(h)) => Some(format!("{}x{}", w, h)),
                _ => None,
            }),
            stream_count: streams.len() as u32,
            streams,
            file_hash,
            created_at: chrono_timestamp(),
        }
    }
}

fn chrono_timestamp() -> String {
    // Standard RFC3339 timestamp format without external chrono dependency
    use std::time::{SystemTime, UNIX_EPOCH};
    let dur = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
    format!("{}.{:03}Z", dur.as_secs(), dur.subsec_millis())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::MediaKind;

    #[test]
    fn test_fingerprint_generation() {
        let meta = MediaMetadata {
            id: "vid_998".to_string(),
            title: "Nature Documentary".to_string(),
            uploader: None,
            uploader_avatar: None,
            channel_id: None,
            uploader_url: None,
            duration: Some(120.0),
            thumbnail: None,
            webpage_url: "https://example.com/watch?v=vid_998".to_string(),
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
            extractor: Some("youtube".to_string()),
            extractor_key: Some("Youtube".to_string()),
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
            source_type: None,
            strategy: None,
            transcoding_cost: None,
            transcoding_explanation: None,
            capabilities: None,
        };

        let inspection = MediaInspection {
            container_format: "mp4".to_string(),
            video_codec: Some("h264".to_string()),
            video_profile: Some("High".to_string()),
            audio_codec: Some("aac".to_string()),
            width: Some(1920),
            height: Some(1080),
            fps: Some(60.0),
            bit_depth: Some(8),
            color_space: Some("bt709".to_string()),
            is_hdr: Some(false),
            bitrate_kbps: Some(4500),
            audio_channels: Some(2),
            audio_sample_rate_hz: Some(48000),
            audio_bitrate_kbps: Some(192),
            audio_language: Some("eng".to_string()),
            file_size_bytes: 65000000,
            duration_seconds: Some(120.0),
            is_lossy_transcode_warning: false,
            stream_count: Some(2),
            chapters_count: Some(0),
        };

        let fp = FingerprintEngine::generate(&meta, Some(&inspection), None);
        assert!(fp.canonical_id.starts_with("mfp_"));
        assert_eq!(fp.source.source_id, "vid_998");
        assert_eq!(fp.video_codec, Some("h264".to_string()));
        assert_eq!(fp.audio_codec, Some("aac".to_string()));
        assert_eq!(fp.max_resolution, Some("1920x1080".to_string()));
        assert_eq!(fp.stream_count, 2);
    }
}
