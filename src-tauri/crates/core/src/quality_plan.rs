use crate::types::{
    DownloadPlan, MediaFormatSpec, MediaMetadata, PlanMediaSummary, PlanProcessing, PresetType,
    ProcessingClass, StreamSummary,
};

fn has_codec(codec: Option<&str>) -> bool {
    codec
        .map(|value| !value.is_empty() && value != "none")
        .unwrap_or(false)
}

fn estimated_bytes(format: &MediaFormatSpec) -> Option<u64> {
    format.filesize.or(format.filesize_approx)
}

fn video_summary(format: &MediaFormatSpec) -> StreamSummary {
    StreamSummary {
        codec: format.vcodec.clone().filter(|codec| codec != "none"),
        bitrate_kbps: format.vbr.or(format.tbr).map(|value| value.round() as u64),
        width: format.width,
        height: format.height,
        fps: format.fps,
        hdr: format.hdr,
        language: None,
    }
}

fn audio_summary(format: &MediaFormatSpec) -> StreamSummary {
    StreamSummary {
        codec: format.acodec.clone().filter(|codec| codec != "none"),
        bitrate_kbps: format.abr.or(format.tbr).map(|value| value.round() as u64),
        width: None,
        height: None,
        fps: None,
        hdr: None,
        language: None,
    }
}

fn best_video<'a>(
    formats: &'a [MediaFormatSpec],
    max_height: Option<u32>,
) -> Option<&'a MediaFormatSpec> {
    formats
        .iter()
        .filter(|format| has_codec(format.vcodec.as_deref()))
        .filter(|format| {
            max_height
                .map(|limit| format.height.unwrap_or(0) <= limit)
                .unwrap_or(true)
        })
        .max_by(|left, right| {
            left.height
                .unwrap_or(0)
                .cmp(&right.height.unwrap_or(0))
                .then_with(|| {
                    left.fps
                        .unwrap_or(0.0)
                        .partial_cmp(&right.fps.unwrap_or(0.0))
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .then_with(|| {
                    left.vbr
                        .unwrap_or(0.0)
                        .partial_cmp(&right.vbr.unwrap_or(0.0))
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
        })
}

fn best_audio(formats: &[MediaFormatSpec]) -> Option<&MediaFormatSpec> {
    formats
        .iter()
        .filter(|format| has_codec(format.acodec.as_deref()))
        .max_by(|left, right| {
            left.abr
                .unwrap_or(0.0)
                .partial_cmp(&right.abr.unwrap_or(0.0))
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| {
                    estimated_bytes(left)
                        .unwrap_or(0)
                        .cmp(&estimated_bytes(right).unwrap_or(0))
                })
        })
}

fn parse_quality_limit(quality: &str) -> Option<u32> {
    if quality.is_empty() || quality == "auto" {
        None
    } else {
        quality.parse::<u32>().ok()
    }
}

fn source_summary(formats: &[MediaFormatSpec]) -> PlanMediaSummary {
    let video = best_video(formats, None);
    let audio = best_audio(formats);
    let container = match (video, audio) {
        (Some(video), _) if has_codec(video.acodec.as_deref()) => Some(video.ext.clone()),
        (Some(video), Some(audio)) if video.ext.eq_ignore_ascii_case(&audio.ext) => {
            Some(video.ext.clone())
        }
        (Some(video), None) => Some(video.ext.clone()),
        (None, Some(audio)) => Some(audio.ext.clone()),
        _ => None,
    }.filter(|ext| !ext.is_empty());
    PlanMediaSummary {
        video: video.map(video_summary),
        audio: audio.map(audio_summary),
        container,
        estimated_bytes: match (
            video.and_then(estimated_bytes),
            audio.and_then(estimated_bytes),
        ) {
            (Some(video_bytes), Some(audio_bytes))
                if video
                    .map(|format| !has_codec(format.acodec.as_deref()))
                    .unwrap_or(false) =>
            {
                Some(video_bytes.saturating_add(audio_bytes))
            }
            (Some(bytes), _) | (_, Some(bytes)) => Some(bytes),
            _ => None,
        },
    }
}

pub fn build_download_plan(
    metadata: &MediaMetadata,
    preset: PresetType,
    quality: &str,
) -> DownloadPlan {
    let formats = metadata.formats.as_deref().unwrap_or(&[]);
    let mut source = source_summary(formats);
    if let Some(audio) = source.audio.as_mut() {
        audio.language = metadata.language.clone();
    }
    let max_height = parse_quality_limit(quality);
    let selected_video = best_video(formats, max_height);
    let selected_audio = best_audio(formats);

    let mut output = PlanMediaSummary::default();
    let processing = match preset {
        PresetType::BestAudio => {
            output.audio = selected_audio
                .map(audio_summary)
                .or_else(|| source.audio.clone());
            if let Some(audio) = output.audio.as_mut() {
                audio.language = metadata.language.clone();
            }
            output.container = selected_audio
                .map(|format| format.ext.clone())
                .filter(|ext| !ext.is_empty());
            output.estimated_bytes = selected_audio.and_then(estimated_bytes);
            PlanProcessing {
                class: ProcessingClass::SourcePreserved,
                video_reencoded: Some(false),
                audio_reencoded: Some(false),
                explanation: Some("The best available source audio stream is extracted without an unnecessary lossy-to-lossy conversion.".to_string()),
            }
        }
        PresetType::Mp3 => {
            output.audio = Some(StreamSummary {
                codec: Some("mp3".to_string()),
                bitrate_kbps: Some(320),
                ..StreamSummary::default()
            });
            output.container = Some("mp3".to_string());
            PlanProcessing {
                class: ProcessingClass::AudioTranscode,
                video_reencoded: Some(false),
                audio_reencoded: Some(true),
                explanation: Some("Audio will be transcoded to MP3 for compatibility. The output bitrate cannot restore detail missing from the source stream.".to_string()),
            }
        }
        PresetType::Flac => {
            output.audio = Some(StreamSummary {
                codec: Some("flac".to_string()),
                ..StreamSummary::default()
            });
            output.container = Some("flac".to_string());
            PlanProcessing {
                class: ProcessingClass::AudioTranscode,
                video_reencoded: Some(false),
                audio_reencoded: Some(true),
                explanation: Some("Audio will be decoded and encoded as FLAC. A lossy source does not become an original lossless recording.".to_string()),
            }
        }
        PresetType::BestVideo => {
            output.video = selected_video
                .map(video_summary)
                .or_else(|| source.video.clone());
            output.audio = selected_audio
                .map(audio_summary)
                .or_else(|| source.audio.clone());
            output.container = Some("mkv".to_string());
            output.estimated_bytes = source.estimated_bytes;
            let separate_streams = selected_video
                .map(|format| !has_codec(format.acodec.as_deref()))
                .unwrap_or(metadata.has_video && metadata.has_audio);
            PlanProcessing {
                class: if separate_streams {
                    ProcessingClass::MergeOnly
                } else {
                    ProcessingClass::RemuxOnly
                },
                video_reencoded: Some(false),
                audio_reencoded: Some(false),
                explanation: Some(if separate_streams {
                    "The selected video and audio streams will be merged without re-encoding."
                        .to_string()
                } else {
                    "The selected streams will be placed in an MKV container without re-encoding."
                        .to_string()
                }),
            }
        }
        PresetType::Mp4Compatible => {
            let compatible_video = formats
                .iter()
                .filter(|format| {
                    has_codec(format.vcodec.as_deref())
                        && max_height
                            .map(|limit| format.height.unwrap_or(0) <= limit)
                            .unwrap_or(true)
                        && (format.ext.eq_ignore_ascii_case("mp4")
                            || format
                                .vcodec
                                .as_deref()
                                .map(|codec| codec.starts_with("avc1") || codec.starts_with("h264"))
                                .unwrap_or(false))
                })
                .max_by_key(|format| format.height.unwrap_or(0));
            let compatible_audio = formats
                .iter()
                .filter(|format| {
                    has_codec(format.acodec.as_deref())
                        && (format.ext.eq_ignore_ascii_case("m4a")
                            || format
                                .acodec
                                .as_deref()
                                .map(|codec| codec.starts_with("aac") || codec.starts_with("mp4a"))
                                .unwrap_or(false))
                })
                .max_by(|left, right| {
                    left.abr
                        .unwrap_or(0.0)
                        .partial_cmp(&right.abr.unwrap_or(0.0))
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
            output.video = compatible_video.map(video_summary);
            output.audio = compatible_audio.map(audio_summary);
            output.container = Some("mp4".to_string());
            output.estimated_bytes = match (
                compatible_video.and_then(estimated_bytes),
                compatible_audio.and_then(estimated_bytes),
            ) {
                (Some(video_bytes), Some(audio_bytes)) => {
                    Some(video_bytes.saturating_add(audio_bytes))
                }
                (Some(bytes), _) | (_, Some(bytes)) => Some(bytes),
                _ => None,
            };
            let plan_known = !metadata.has_video || compatible_video.is_some();
            PlanProcessing {
                class: if plan_known {
                    ProcessingClass::MergeOnly
                } else {
                    ProcessingClass::Unknown
                },
                video_reencoded: if plan_known { Some(false) } else { None },
                audio_reencoded: if plan_known { Some(false) } else { None },
                explanation: Some(if plan_known {
                    "Compatible source streams will be merged into an MP4 container without re-encoding.".to_string()
                } else {
                    "No compatible MP4 video stream was identified; final processing details are unavailable until the backend resolves the download.".to_string()
                }),
            }
        }
    };

    DownloadPlan {
        source,
        output,
        processing,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analyzer::parse_ytdlp_json;

    fn metadata_with_formats() -> MediaMetadata {
        parse_ytdlp_json(r#"{
            "id":"sample","title":"Sample","webpage_url":"https://example.com/watch",
            "formats":[
                {"format_id":"v","ext":"webm","width":3840,"height":2160,"fps":60,"vcodec":"vp9","acodec":"none","vbr":8000,"filesize":1000000,"dynamic_range":"HDR10"},
                {"format_id":"a","ext":"webm","vcodec":"none","acodec":"opus","abr":160,"filesize":100000},
                {"format_id":"v-mp4","ext":"mp4","width":1920,"height":1080,"fps":30,"vcodec":"avc1.640028","acodec":"none","vbr":4000},
                {"format_id":"a-m4a","ext":"m4a","vcodec":"none","acodec":"mp4a.40.2","abr":128}
            ]
        }"#, "https://example.com/watch").unwrap()
    }

    #[test]
    fn classifies_separate_streams_as_merge_only() {
        let plan = build_download_plan(&metadata_with_formats(), PresetType::BestVideo, "auto");
        assert_eq!(plan.processing.class, ProcessingClass::MergeOnly);
        assert_eq!(plan.processing.video_reencoded, Some(false));
    }

    #[test]
    fn classifies_mp3_and_flac_as_audio_transcodes() {
        let metadata = metadata_with_formats();
        for preset in [PresetType::Mp3, PresetType::Flac] {
            let plan = build_download_plan(&metadata, preset, "auto");
            assert_eq!(plan.processing.class, ProcessingClass::AudioTranscode);
            assert_eq!(plan.processing.audio_reencoded, Some(true));
        }
    }

    #[test]
    fn does_not_fabricate_unknown_source_fields() {
        let mut metadata = metadata_with_formats();
        metadata.formats = None;
        let plan = build_download_plan(&metadata, PresetType::BestAudio, "auto");
        assert!(plan.source.audio.is_none());
        assert!(plan.source.video.is_none());
        assert!(plan.source.container.is_none());
    }

    #[test]
    fn mp4_plan_selects_compatible_streams_with_quality_limit() {
        let plan = build_download_plan(&metadata_with_formats(), PresetType::Mp4Compatible, "1080");
        assert_eq!(plan.processing.class, ProcessingClass::MergeOnly);
        assert_eq!(plan.output.video.and_then(|video| video.height), Some(1080));
        assert_eq!(plan.output.container.as_deref(), Some("mp4"));
    }

    #[test]
    fn progressive_source_is_remux_only_for_mkv_output() {
        let metadata = parse_ytdlp_json(r#"{
            "id":"progressive","title":"Progressive","webpage_url":"https://example.com/video",
            "formats":[{"format_id":"p","ext":"mp4","width":1280,"height":720,"fps":30,"vcodec":"h264","acodec":"aac","tbr":1500}]
        }"#, "https://example.com/video").unwrap();
        let plan = build_download_plan(&metadata, PresetType::BestVideo, "auto");
        assert_eq!(plan.processing.class, ProcessingClass::RemuxOnly);
        assert_eq!(plan.processing.video_reencoded, Some(false));
        assert_eq!(plan.processing.audio_reencoded, Some(false));
    }

    #[test]
    fn best_audio_preserves_source_codec() {
        let plan = build_download_plan(&metadata_with_formats(), PresetType::BestAudio, "auto");
        assert_eq!(plan.processing.class, ProcessingClass::SourcePreserved);
        assert_eq!(plan.output.audio.and_then(|audio| audio.codec).as_deref(), Some("opus"));
    }
}
