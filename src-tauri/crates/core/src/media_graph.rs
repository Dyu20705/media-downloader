use crate::core::types::{
    AudioStreamSpec, MediaChapter, MediaFormatSpec, MediaKind, MediaMetadata,
    MediaSourceType, OutputMediaArtifact, SourceMediaGraph, SubtitleTrack, ThumbnailSpec,
    VideoStreamSpec,
};

pub struct MediaGraph;

impl MediaGraph {
    /// Builds a SourceMediaGraph from metadata ensuring strict stream isolation
    pub fn build_source_graph(
        metadata: &MediaMetadata,
        source_type: MediaSourceType,
    ) -> SourceMediaGraph {
        let mut video_streams: Vec<VideoStreamSpec> = Vec::new();
        let mut audio_streams: Vec<AudioStreamSpec> = Vec::new();

        if let Some(formats) = &metadata.formats {
            for f in formats {
                if let (Some(w), Some(h)) = (f.width, f.height) {
                    if w > 0 && h > 0 {
                        video_streams.push(VideoStreamSpec {
                            stream_id: f.format_id.clone(),
                            codec: f.vcodec.clone().unwrap_or_else(|| "unknown".to_string()),
                            profile: None,
                            width: w,
                            height: h,
                            fps: f.fps.unwrap_or(30.0),
                            bitrate_kbps: f.vbr.or(f.tbr).map(|b| b as u64),
                            is_hdr: f.hdr.unwrap_or(false),
                            dynamic_range: f.dynamic_range.clone(),
                            aspect_ratio: Some(format!("{}:{}", w, h)),
                            filesize_approx: f.filesize_approx.or(f.filesize),
                        });
                    }
                }

                if f.acodec.is_some() && f.acodec.as_deref() != Some("none") {
                    audio_streams.push(AudioStreamSpec {
                        stream_id: f.format_id.clone(),
                        codec: f.acodec.clone().unwrap_or_else(|| "unknown".to_string()),
                        bitrate_kbps: f.abr.map(|b| b as u64),
                        sample_rate_hz: f.audio_sample_rate,
                        channels: f.audio_channels,
                        language: None,
                        is_default: false,
                        filesize_approx: f.filesize_approx.or(f.filesize),
                    });
                }
            }
        }

        let thumbnails = if let Some(thumb) = &metadata.thumbnail {
            vec![ThumbnailSpec {
                url: thumb.clone(),
                width: None,
                height: None,
                id: Some("primary".to_string()),
            }]
        } else {
            Vec::new()
        };

        SourceMediaGraph {
            source_url: metadata.webpage_url.clone(),
            extractor: metadata.extractor.clone().unwrap_or_else(|| "generic".to_string()),
            source_type,
            title: metadata.title.clone(),
            media_kind: metadata.media_kind,
            duration_seconds: metadata.duration,
            video_streams,
            audio_streams,
            subtitle_streams: metadata.subtitles.clone().unwrap_or_default(),
            chapters: metadata.chapters.clone().unwrap_or_default(),
            thumbnails,
            formats: metadata.formats.clone().unwrap_or_default(),
        }
    }

    /// Creates a verified OutputMediaArtifact from completed download path & inspection
    pub fn build_output_artifact(
        file_path: &str,
        container: &str,
        file_size_bytes: u64,
        duration: Option<f64>,
        video_codec: Option<String>,
        audio_codec: Option<String>,
        width: Option<u32>,
        height: Option<u32>,
        fps: Option<f64>,
        is_verified: bool,
    ) -> OutputMediaArtifact {
        let file_name = std::path::Path::new(file_path)
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_else(|| "output_media".to_string());

        OutputMediaArtifact {
            artifact_path: file_path.to_string(),
            file_name,
            container: container.to_string(),
            file_size_bytes,
            duration_seconds: duration,
            video_codec,
            audio_codec,
            width,
            height,
            fps,
            is_verified,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_graph_separation() {
        let meta = MediaMetadata {
            id: "test123".to_string(),
            title: "Test Video".to_string(),
            uploader: None,
            channel_id: None,
            uploader_url: None,
            duration: Some(180.0),
            thumbnail: Some("https://example.com/thumb.jpg".to_string()),
            webpage_url: "https://example.com/watch?v=123".to_string(),
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
            formats: Some(vec![
                MediaFormatSpec {
                    format_id: "137".to_string(),
                    ext: "mp4".to_string(),
                    resolution: Some("1920x1080".to_string()),
                    width: Some(1920),
                    height: Some(1080),
                    fps: Some(60.0),
                    vcodec: Some("avc1.64002a".to_string()),
                    acodec: Some("none".to_string()),
                    filesize: Some(50000000),
                    filesize_approx: None,
                    tbr: Some(4000.0),
                    vbr: Some(4000.0),
                    abr: None,
                    hdr: Some(false),
                    dynamic_range: Some("SDR".to_string()),
                    audio_sample_rate: None,
                    audio_channels: None,
                },
                MediaFormatSpec {
                    format_id: "140".to_string(),
                    ext: "m4a".to_string(),
                    resolution: None,
                    width: None,
                    height: None,
                    fps: None,
                    vcodec: Some("none".to_string()),
                    acodec: Some("mp4a.40.2".to_string()),
                    filesize: Some(5000000),
                    filesize_approx: None,
                    tbr: Some(128.0),
                    vbr: None,
                    abr: Some(128.0),
                    hdr: None,
                    dynamic_range: None,
                    audio_sample_rate: Some(44100),
                    audio_channels: Some(2),
                },
            ]),
            smart_recommendation: None,
            source_type: Some(MediaSourceType::YtDlpExtractor),
            strategy: None,
            transcoding_cost: None,
            transcoding_explanation: None,
            capabilities: None,
        };

        let graph = MediaGraph::build_source_graph(&meta, MediaSourceType::YtDlpExtractor);
        assert_eq!(graph.video_streams.len(), 1);
        assert_eq!(graph.audio_streams.len(), 1);
        assert_eq!(graph.video_streams[0].width, 1920);
        assert_eq!(graph.video_streams[0].fps, 60.0);
        assert_eq!(graph.audio_streams[0].channels, Some(2));
        assert_eq!(graph.thumbnails.len(), 1);
    }
}
