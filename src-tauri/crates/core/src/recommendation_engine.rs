use crate::core::types::{
    FormatRecommendation, MediaFormatSpec, MediaKind, PresetType, RecommendationConstraints,
    TranscodingCost, UserIntent,
};

pub struct RecommendationEngine;

impl RecommendationEngine {
    /// Evaluates the media stream specifications and user intent to yield a grounded recommendation
    pub fn recommend(
        intent: UserIntent,
        media_kind: MediaKind,
        formats: &[MediaFormatSpec],
        available_resolutions: &[u32],
        is_hdr: bool,
        constraints: &RecommendationConstraints,
    ) -> FormatRecommendation {
        let max_res = available_resolutions.iter().cloned().max().unwrap_or(1080);
        let min_res = constraints.min_height.unwrap_or(0);

        if media_kind == MediaKind::Audio || available_resolutions.is_empty() {
            return Self::recommend_audio(intent, formats, constraints);
        }

        match intent {
            UserIntent::MaxQuality => {
                let target_quality = if max_res >= 2160 {
                    "2160".to_string()
                } else if max_res >= 1440 {
                    "1440".to_string()
                } else if max_res >= 1080 {
                    "1080".to_string()
                } else {
                    "auto".to_string()
                };

                let label = if max_res >= 2160 {
                    if is_hdr {
                        "MKV · 4K 60fps HDR".to_string()
                    } else {
                        "MKV · 4K Ultra HD".to_string()
                    }
                } else if max_res >= 1440 {
                    "MKV · 1440p QHD".to_string()
                } else {
                    "MP4 · 1080p Full HD".to_string()
                };

                let mut why = vec![
                    "✓ highest available source resolution".to_string(),
                    "✓ preserves native bit depth & dynamic range".to_string(),
                ];
                if is_hdr {
                    why.push("✓ HDR stream multiplexing enabled".to_string());
                }
                why.push("✓ multiplexed without lossy downscaling".to_string());

                FormatRecommendation {
                    preset: if max_res > 1080 {
                        PresetType::BestVideo
                    } else {
                        PresetType::Mp4Compatible
                    },
                    label,
                    target_quality,
                    reason: "Optimal master quality without downscaling".to_string(),
                    why_reasons: why,
                    is_transcode_free: true,
                    transcoding_cost: TranscodingCost::Merge,
                    estimated_size_bytes: formats.iter().filter_map(|f| f.filesize_approx).max(),
                    container: if max_res > 1080 { "mkv".to_string() } else { "mp4".to_string() },
                    details: Some("Multiplexes highest resolution video stream with master audio stream.".to_string()),
                }
            }
            UserIntent::SmallestSize => {
                let target_quality = if max_res >= 720 && min_res <= 720 {
                    "720".to_string()
                } else if max_res >= 480 && min_res <= 480 {
                    "480".to_string()
                } else {
                    "720".to_string()
                };

                let why = vec![
                    "✓ lowest storage footprint".to_string(),
                    "✓ efficient AVC/AAC stream selection".to_string(),
                    "✓ no CPU-heavy video transcoding".to_string(),
                    "✓ fast download throughput".to_string(),
                ];

                FormatRecommendation {
                    preset: PresetType::Mp4Compatible,
                    label: "MP4 · 720p Space-Saver".to_string(),
                    target_quality,
                    reason: "Balanced efficiency for storage and quick transfers".to_string(),
                    why_reasons: why,
                    is_transcode_free: true,
                    transcoding_cost: TranscodingCost::StreamCopy,
                    estimated_size_bytes: formats.iter().filter_map(|f| f.filesize_approx).min(),
                    container: "mp4".to_string(),
                    details: Some("Selects compact 720p stream with efficient bitrates.".to_string()),
                }
            }
            UserIntent::BestCompatibility => {
                let target_quality = if max_res >= 1080 {
                    "1080".to_string()
                } else {
                    "720".to_string()
                };

                let why = vec![
                    "✓ universal H.264 / AAC compatibility".to_string(),
                    "✓ plays on iOS, Android, macOS, Windows, TVs".to_string(),
                    "✓ standard MP4 container".to_string(),
                    "✓ zero transcoding artifacting".to_string(),
                ];

                FormatRecommendation {
                    preset: PresetType::Mp4Compatible,
                    label: format!("MP4 · {}p Universal", if max_res >= 1080 { 1080 } else { 720 }),
                    target_quality,
                    reason: "Widely supported MP4 container with H.264/AAC for all devices".to_string(),
                    why_reasons: why,
                    is_transcode_free: true,
                    transcoding_cost: TranscodingCost::Merge,
                    estimated_size_bytes: None,
                    container: "mp4".to_string(),
                    details: Some("Standard MP4 with wide hardware acceleration support.".to_string()),
                }
            }
            UserIntent::Balanced => {
                let target_quality = if max_res >= 1080 {
                    "1080".to_string()
                } else if max_res >= 720 {
                    "720".to_string()
                } else {
                    "auto".to_string()
                };

                let label = if max_res >= 1080 {
                    "MP4 · 1080p (Best Balance)".to_string()
                } else if max_res >= 720 {
                    "MP4 · 720p HD".to_string()
                } else {
                    "MP4 · Auto Quality".to_string()
                };

                let why = vec![
                    "✓ native source quality".to_string(),
                    "✓ MP4 universal playback".to_string(),
                    "✓ no video transcoding".to_string(),
                    "✓ balanced download speed and clarity".to_string(),
                ];

                FormatRecommendation {
                    preset: PresetType::Mp4Compatible,
                    label,
                    target_quality,
                    reason: "Optimal balance of visual fidelity, file size, and compatibility".to_string(),
                    why_reasons: why,
                    is_transcode_free: true,
                    transcoding_cost: TranscodingCost::Merge,
                    estimated_size_bytes: None,
                    container: "mp4".to_string(),
                    details: Some("Combines Full HD visuals with standard audio in MP4 container.".to_string()),
                }
            }
        }
    }

    fn recommend_audio(
        intent: UserIntent,
        formats: &[MediaFormatSpec],
        _constraints: &RecommendationConstraints,
    ) -> FormatRecommendation {
        match intent {
            UserIntent::BestCompatibility => FormatRecommendation {
                preset: PresetType::Mp3,
                label: "MP3 · Universal Audio".to_string(),
                target_quality: "auto".to_string(),
                reason: "Standard MP3 format for universal device compatibility".to_string(),
                why_reasons: vec![
                    "✓ compatible with all media players".to_string(),
                    "✓ auto-transcoded to VBR MP3 (highest quality)".to_string(),
                    "✓ lightweight file footprint".to_string(),
                ],
                is_transcode_free: false,
                transcoding_cost: TranscodingCost::Transcode,
                estimated_size_bytes: None,
                container: "mp3".to_string(),
                details: Some("Encodes audio to universal MP3 format.".to_string()),
            },
            UserIntent::MaxQuality => {
                let has_flac = formats.iter().any(|f| f.ext == "flac");
                if has_flac {
                    FormatRecommendation {
                        preset: PresetType::Flac,
                        label: "FLAC · Lossless Audio".to_string(),
                        target_quality: "auto".to_string(),
                        reason: "Preserve uncompressed lossless audio fidelity".to_string(),
                        why_reasons: vec![
                            "✓ lossless fidelity bit-for-bit".to_string(),
                            "✓ native studio master preservation".to_string(),
                            "✓ lossless FLAC container".to_string(),
                        ],
                        is_transcode_free: true,
                        transcoding_cost: TranscodingCost::StreamCopy,
                        estimated_size_bytes: None,
                        container: "flac".to_string(),
                        details: Some("Preserves uncompressed audio tracks.".to_string()),
                    }
                } else {
                    FormatRecommendation {
                        preset: PresetType::BestAudio,
                        label: "M4A · Source Audio (Opus/AAC)".to_string(),
                        target_quality: "auto".to_string(),
                        reason: "Preserve original audio bitstream without lossy re-encoding".to_string(),
                        why_reasons: vec![
                            "✓ original unaltered audio bitstream".to_string(),
                            "✓ zero generational transcoding loss".to_string(),
                            "✓ highest source bitrate".to_string(),
                        ],
                        is_transcode_free: true,
                        transcoding_cost: TranscodingCost::StreamCopy,
                        estimated_size_bytes: None,
                        container: "m4a".to_string(),
                        details: Some("Directly copies native audio stream without recompression.".to_string()),
                    }
                }
            }
            _ => FormatRecommendation {
                preset: PresetType::BestAudio,
                label: "M4A · Direct Stream Audio".to_string(),
                target_quality: "auto".to_string(),
                reason: "Direct stream copy of highest bitrate source audio".to_string(),
                why_reasons: vec![
                    "✓ untouched source stream".to_string(),
                    "✓ no transcoding required".to_string(),
                    "✓ fast instant acquisition".to_string(),
                ],
                is_transcode_free: true,
                transcoding_cost: TranscodingCost::StreamCopy,
                estimated_size_bytes: None,
                container: "m4a".to_string(),
                details: Some("Stream copy of source audio without re-encoding.".to_string()),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recommend_max_quality_4k() {
        let rec = RecommendationEngine::recommend(
            UserIntent::MaxQuality,
            MediaKind::Video,
            &[],
            &[2160, 1080, 720],
            true,
            &RecommendationConstraints::default(),
        );

        assert_eq!(rec.preset, PresetType::BestVideo);
        assert_eq!(rec.target_quality, "2160");
        assert!(rec.label.contains("4K"));
        assert!(rec.why_reasons.iter().any(|w| w.contains("HDR")));
        assert_eq!(rec.transcoding_cost, TranscodingCost::Merge);
    }

    #[test]
    fn test_recommend_smallest_size() {
        let rec = RecommendationEngine::recommend(
            UserIntent::SmallestSize,
            MediaKind::Video,
            &[],
            &[1080, 720, 480],
            false,
            &RecommendationConstraints::default(),
        );

        assert_eq!(rec.preset, PresetType::Mp4Compatible);
        assert_eq!(rec.target_quality, "720");
        assert!(rec.label.contains("Space-Saver"));
        assert!(rec.why_reasons.iter().any(|w| w.contains("lowest storage footprint")));
    }

    #[test]
    fn test_recommend_compatibility() {
        let rec = RecommendationEngine::recommend(
            UserIntent::BestCompatibility,
            MediaKind::Video,
            &[],
            &[1080, 720],
            false,
            &RecommendationConstraints::default(),
        );

        assert_eq!(rec.preset, PresetType::Mp4Compatible);
        assert_eq!(rec.container, "mp4");
        assert!(rec.why_reasons.iter().any(|w| w.contains("universal H.264")));
    }

    #[test]
    fn test_recommend_audio_preservation() {
        let rec = RecommendationEngine::recommend(
            UserIntent::MaxQuality,
            MediaKind::Audio,
            &[],
            &[],
            false,
            &RecommendationConstraints::default(),
        );

        assert_eq!(rec.preset, PresetType::BestAudio);
        assert!(rec.is_transcode_free);
        assert_eq!(rec.transcoding_cost, TranscodingCost::StreamCopy);
        assert!(rec.why_reasons.iter().any(|w| w.contains("zero generational transcoding loss")));
    }
}
