use crate::types::{
    DownloadJob, DownloadRecipe, DownloadStrategy, MediaSourceType, PresetType,
    VerificationChecklist,
};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct RecipeEngine;

impl RecipeEngine {
    /// Creates a reproducible DownloadRecipe from job state and verification
    pub fn create_recipe(
        job: &DownloadJob,
        verification: Option<VerificationChecklist>,
    ) -> DownloadRecipe {
        let resolver_type = job
            .metadata
            .source_type
            .unwrap_or(MediaSourceType::YtDlpExtractor);
        let extractor = job
            .metadata
            .extractor
            .clone()
            .unwrap_or_else(|| "generic".to_string());

        let output_container = match job.preset {
            PresetType::Mp4Compatible => "mp4".to_string(),
            PresetType::BestVideo => "mkv".to_string(),
            PresetType::BestAudio => "m4a".to_string(),
            PresetType::Mp3 => "mp3".to_string(),
            PresetType::Flac => "flac".to_string(),
        };

        let strategy = job
            .metadata
            .strategy
            .unwrap_or(DownloadStrategy::YtDlpMerge);

        let mut transformations = Vec::new();
        match strategy {
            DownloadStrategy::DirectCopy => {
                transformations.push("Direct stream copy (zero re-encoding)".to_string());
            }
            DownloadStrategy::YtDlpMerge => {
                transformations.push("Multiplex video and audio bitstreams".to_string());
                transformations.push(format!(
                    "Target container encapsulation: {}",
                    output_container
                ));
            }
            DownloadStrategy::FfmpegRemux => {
                transformations.push("Container remuxing without bitstream alteration".to_string());
            }
            DownloadStrategy::FfmpegTranscode => {
                transformations.push("Transcode to target audio codec".to_string());
            }
            DownloadStrategy::HlsDownload => {
                transformations.push("HLS segment fetching and unified concatenation".to_string());
            }
            DownloadStrategy::DashDownload => {
                transformations.push("DASH adaptation set fetching and multiplexing".to_string());
            }
            DownloadStrategy::YtDlpDownload => {
                transformations.push("Progressive stream acquisition".to_string());
            }
        }

        if let Some(opts) = &job.subtitle_options {
            if opts.mode == crate::types::SubtitleMode::Embed {
                transformations.push(format!(
                    "Embed subtitle stream ({})",
                    opts.selected_language.as_deref().unwrap_or("auto")
                ));
            }
        }

        let mut hasher = DefaultHasher::new();
        job.id.hash(&mut hasher);
        job.url.hash(&mut hasher);
        let recipe_id = format!("recipe_{:012x}", hasher.finish());

        let timestamp = {
            let dur = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default();
            format!("{}.{:03}Z", dur.as_secs(), dur.subsec_millis())
        };

        DownloadRecipe {
            id: recipe_id,
            source_url: sanitize_url(&job.url),
            resolver_type,
            extractor,
            selected_candidates: vec![job.quality.clone()],
            strategy,
            output_container,
            transformations,
            intent: job.intent,
            verification,
            timestamp,
            resulting_artifact_path: job.final_file_path.clone(),
        }
    }
}

/// Removes sensitive session tokens or signed query params for clean reproducible recipe storage
fn sanitize_url(raw: &str) -> String {
    if let Ok(mut url) = url::Url::parse(raw) {
        let pairs: Vec<(String, String)> = url
            .query_pairs()
            .filter(|(k, _)| {
                !k.starts_with("token")
                    && !k.starts_with("auth")
                    && !k.starts_with("sig")
                    && !k.starts_with("key")
            })
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();

        url.set_query(None);
        for (k, v) in pairs {
            url.query_pairs_mut().append_pair(&k, &v);
        }
        url.to_string()
    } else {
        raw.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;

    #[test]
    fn test_recipe_creation_and_sanitization() {
        let meta = MediaMetadata {
            id: "vid_1".to_string(),
            title: "Test Recipe".to_string(),
            uploader: None,
            uploader_avatar: None,
            channel_id: None,
            uploader_url: None,
            duration: Some(60.0),
            thumbnail: None,
            webpage_url: "https://example.com/watch?v=1&token=SECRET_123".to_string(),
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
            available_frame_rates: vec![30],
            has_video: true,
            has_audio: true,
            is_hdr: Some(false),
            subtitles: None,
            automatic_captions: None,
            chapters: None,
            formats: None,
            smart_recommendation: None,
            source_type: Some(MediaSourceType::YtDlpExtractor),
            strategy: Some(DownloadStrategy::YtDlpMerge),
            transcoding_cost: Some(TranscodingCost::Merge),
            transcoding_explanation: None,
            capabilities: None,
        };

        let job = DownloadJob {
            id: "job_01".to_string(),
            url: "https://example.com/watch?v=1&token=SECRET_123".to_string(),
            preset: PresetType::Mp4Compatible,
            quality: "1080".to_string(),
            output_directory: "/tmp".to_string(),
            status: DownloadStatus::Completed,
            progress: DownloadProgress::default(),
            metadata: meta,
            final_file_name: Some("video.mp4".to_string()),
            final_file_path: Some("/tmp/video.mp4".to_string()),
            inspection: None,
            error_message: None,
            created_at: "now".to_string(),
            completed_at: Some("now".to_string()),
            subtitle_options: None,
            sponsor_block_mode: None,
            intent: Some(UserIntent::Balanced),
            recipe: None,
            fingerprint: None,
            explainable_result: None,
            verification: None,
        };

        let recipe = RecipeEngine::create_recipe(&job, None);
        assert!(recipe.id.starts_with("recipe_"));
        assert!(!recipe.source_url.contains("SECRET_123"));
        assert_eq!(recipe.output_container, "mp4");
        assert_eq!(recipe.strategy, DownloadStrategy::YtDlpMerge);
        assert!(recipe.transformations.len() >= 2);
    }
}
