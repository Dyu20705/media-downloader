use crate::types::{MediaDiff, MediaFingerprint};

/// P2 Extension Point: Media Diff Domain Boundary
pub struct MediaDiffEngine;

impl MediaDiffEngine {
    /// Compares original source fingerprint against downloaded output fingerprint
    pub fn diff(original: &MediaFingerprint, downloaded: &MediaFingerprint) -> MediaDiff {
        let orig_dur = original.duration_seconds.unwrap_or(0.0);
        let down_dur = downloaded.duration_seconds.unwrap_or(0.0);
        let duration_diff_seconds = (orig_dur - down_dur).abs();

        let resolution_changed = original.max_resolution != downloaded.max_resolution;
        let codec_changed = original.video_codec != downloaded.video_codec
            || original.audio_codec != downloaded.audio_codec;
        let container_changed = original.streams != downloaded.streams;

        let is_exact_match = !resolution_changed && !codec_changed && duration_diff_seconds < 0.5;

        MediaDiff {
            original_fingerprint: original.clone(),
            downloaded_fingerprint: downloaded.clone(),
            duration_diff_seconds,
            resolution_changed,
            codec_changed,
            container_changed,
            is_exact_match,
        }
    }
}
