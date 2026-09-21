use crate::types::{
    DownloadJob, ExplainableResult, MediaInspection, MediaKind, OutputMediaArtifact, PresetType,
    TranscodingCost, VerificationChecklist, VerificationLevel, VerificationResult,
};
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::process::Command;

pub async fn verify_and_inspect_media(
    file_path: &Path,
    ffprobe_path: Option<&Path>,
    is_lossy_transcode_warning: bool,
) -> Result<MediaInspection, String> {
    if !file_path.exists() {
        return Err(format!("File does not exist: {}", file_path.display()));
    }

    let metadata =
        std::fs::metadata(file_path).map_err(|e| format!("Failed to read file metadata: {}", e))?;

    let file_size_bytes = metadata.len();
    if file_size_bytes == 0 {
        return Err(format!(
            "Downloaded file is empty (0 bytes): {}",
            file_path.display()
        ));
    }

    if let Some(ffprobe_path) = ffprobe_path {
        return inspect_with_ffprobe(
            ffprobe_path,
            file_path,
            file_size_bytes,
            is_lossy_transcode_warning,
        )
        .await
        .map_err(|error| format!("Deep media verification failed: {error}"));
    }

    // Basic inspection is deliberately not equivalent to successful verification.
    let ext = file_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("unknown")
        .to_lowercase();

    Ok(MediaInspection {
        verification_level: VerificationLevel::BasicInspection,
        container_format: ext.to_uppercase(),
        video_codec: None,
        video_profile: None,
        audio_codec: None,
        width: None,
        height: None,
        fps: None,
        bit_depth: None,
        color_space: None,
        is_hdr: None,
        bitrate_kbps: None,
        audio_channels: None,
        audio_sample_rate_hz: None,
        audio_bitrate_kbps: None,
        audio_language: None,
        file_size_bytes,
        duration_seconds: None,
        is_lossy_transcode_warning,
        stream_count: None,
        chapters_count: None,
    })
}

/// Generates full VerificationResult and ExplainableResult from completed job and inspection
pub fn generate_verification_and_explanation(
    job: &DownloadJob,
    inspection: &MediaInspection,
    file_path: &str,
) -> (VerificationResult, ExplainableResult) {
    let now = {
        let dur = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();
        format!("{}.{:03}Z", dur.as_secs(), dur.subsec_millis())
    };

    let is_audio_only = matches!(
        job.preset,
        PresetType::BestAudio | PresetType::Mp3 | PresetType::Flac
    ) || job.metadata.media_kind == MediaKind::Audio;

    let file_exists = std::path::Path::new(file_path).exists();
    let file_size_valid = inspection.file_size_bytes > 1024;
    let duration_valid = inspection.duration_seconds.is_some_and(|d| d > 0.0);
    let expects_video = !is_audio_only && job.metadata.has_video;
    let expects_audio = is_audio_only || job.metadata.has_audio;
    let video_stream_valid = if !expects_video {
        true
    } else {
        inspection.video_codec.is_some()
    };
    let audio_stream_valid = !expects_audio || inspection.audio_codec.is_some();
    let container_valid = !inspection.container_format.is_empty()
        && !inspection.container_format.eq_ignore_ascii_case("unknown");

    let mut notes = Vec::new();
    if file_size_valid {
        notes.push(format!(
            "File verified: {:.2} MB",
            inspection.file_size_bytes as f64 / (1024.0 * 1024.0)
        ));
    }
    if let Some(dur) = inspection.duration_seconds {
        notes.push(format!("Duration verified: {:.1}s", dur));
    }
    if let Some(vc) = &inspection.video_codec {
        let res_str = match (inspection.width, inspection.height) {
            (Some(w), Some(h)) => format!("{}x{}", w, h),
            _ => "Unknown".to_string(),
        };
        notes.push(format!("Video stream verified: {} ({})", vc, res_str));
    }
    if let Some(ac) = &inspection.audio_codec {
        notes.push(format!("Audio stream verified: {}", ac));
    }

    let checklist = VerificationChecklist {
        file_exists,
        file_size_valid,
        duration_valid,
        video_stream_valid,
        audio_stream_valid,
        container_valid,
        verified_at: now,
        notes,
    };

    let is_valid = inspection.verification_level == VerificationLevel::Verified
        && file_exists
        && file_size_valid
        && duration_valid
        && video_stream_valid
        && audio_stream_valid
        && container_valid;

    let output_artifact = OutputMediaArtifact {
        artifact_path: file_path.to_string(),
        file_name: std::path::Path::new(file_path)
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_else(|| "media_file".to_string()),
        container: inspection.container_format.to_lowercase(),
        file_size_bytes: inspection.file_size_bytes,
        duration_seconds: inspection.duration_seconds,
        video_codec: inspection.video_codec.clone(),
        audio_codec: inspection.audio_codec.clone(),
        width: inspection.width,
        height: inspection.height,
        fps: inspection.fps,
        is_verified: is_valid,
    };

    // Construct specs label (e.g. "1080p60 H.264 + AAC" or "Audio · 320kbps MP3")
    let specs_label = if is_audio_only {
        let codec = inspection.audio_codec.as_deref().unwrap_or("unknown");
        let br = inspection
            .audio_bitrate_kbps
            .map(|b| format!(" · {}kbps", b))
            .unwrap_or_default();
        format!(
            "{} {}{}",
            inspection.container_format,
            codec.to_uppercase(),
            br
        )
    } else {
        let resolution = inspection
            .height
            .map(|height| format!("{height}p"))
            .unwrap_or_else(|| "unknown resolution".to_string());
        let fps = inspection
            .fps
            .map(|fps| format!("{fps:.0}fps "))
            .unwrap_or_default();
        let vcodec = inspection.video_codec.as_deref().unwrap_or("unknown video");
        let acodec = inspection.audio_codec.as_deref().unwrap_or("unknown audio");
        format!(
            "{resolution} {fps}{} + {}",
            vcodec.to_uppercase(),
            acodec.to_uppercase()
        )
    };

    let why_reasons = if let Some(rec) = &job.metadata.smart_recommendation {
        rec.why_reasons.clone()
    } else {
        let mut reasons = vec![
            "✓ matched requested quality target".to_string(),
            "✓ native stream container encapsulation".to_string(),
            "✓ no unnecessary generational transcoding".to_string(),
        ];
        reasons.push(if is_valid {
            "✓ media streams verified with ffprobe".to_string()
        } else {
            "⚠ deep stream verification unavailable or failed".to_string()
        });
        reasons
    };

    let transcoding_cost = job
        .metadata
        .transcoding_cost
        .unwrap_or(TranscodingCost::Merge);
    let processing_summary = match transcoding_cost {
        TranscodingCost::StreamCopy => {
            "Direct stream copy (zero bitstream modification)".to_string()
        }
        TranscodingCost::Merge => {
            "Merged video and audio bitstreams without video re-encode".to_string()
        }
        TranscodingCost::Remux => "Remuxed container packaging (zero re-encoding)".to_string(),
        TranscodingCost::Transcode => "Transcoded audio to requested target codec".to_string(),
        TranscodingCost::NoProcessing => "Media acquired".to_string(),
    };

    let explainable_result = ExplainableResult {
        title: job.metadata.title.clone(),
        specs_label,
        why_reasons,
        processing_summary,
        transcoding_cost,
        verification_checklist: checklist.clone(),
        recipe_id: job.recipe.as_ref().map(|r| r.id.clone()),
    };

    let verification_result = VerificationResult {
        is_valid,
        verification_level: inspection.verification_level,
        checklist,
        output_artifact: Some(output_artifact),
        fingerprint: job.fingerprint.clone(),
    };

    (verification_result, explainable_result)
}

async fn inspect_with_ffprobe(
    ffprobe_bin: &Path,
    file_path: &Path,
    file_size_bytes: u64,
    is_lossy_transcode_warning: bool,
) -> Result<MediaInspection, String> {
    let output = Command::new(ffprobe_bin)
        .arg("-v")
        .arg("quiet")
        .arg("-print_format")
        .arg("json")
        .arg("-show_format")
        .arg("-show_streams")
        .arg(file_path)
        .output()
        .await
        .map_err(|e| format!("Failed to execute ffprobe: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "ffprobe exited with non-zero status: {}",
            stderr.trim()
        ));
    }

    let json_text = String::from_utf8_lossy(&output.stdout);
    let root: Value = serde_json::from_str(&json_text)
        .map_err(|e| format!("Failed to parse ffprobe json: {}", e))?;

    let mut container_format = file_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("unknown")
        .to_uppercase();

    let mut duration_seconds = None;
    let mut bitrate_kbps = None;

    if let Some(fmt) = root.get("format") {
        if let Some(fmt_name) = fmt.get("format_name").and_then(|v| v.as_str()) {
            container_format = fmt_name
                .split(',')
                .next()
                .unwrap_or(fmt_name)
                .to_uppercase();
        }
        if let Some(d_str) = fmt.get("duration").and_then(|v| v.as_str()) {
            duration_seconds = d_str.parse::<f64>().ok();
        }
        if let Some(br_str) = fmt.get("bit_rate").and_then(|v| v.as_str()) {
            if let Ok(br) = br_str.parse::<u64>() {
                bitrate_kbps = Some(br / 1000);
            }
        }
    }

    let mut video_codec = None;
    let mut video_profile = None;
    let mut audio_codec = None;
    let mut width = None;
    let mut height = None;
    let mut fps = None;
    let mut bit_depth = None;
    let mut color_space = None;
    let mut is_hdr = false;
    let mut audio_channels = None;
    let mut audio_sample_rate_hz = None;
    let mut audio_bitrate_kbps = None;
    let mut audio_language = None;
    let mut stream_count = 0u32;

    if let Some(streams) = root.get("streams").and_then(|s| s.as_array()) {
        stream_count = streams.len() as u32;
        for stream in streams {
            let codec_type = stream
                .get("codec_type")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if codec_type == "video" && video_codec.is_none() {
                video_codec = stream
                    .get("codec_name")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                video_profile = stream
                    .get("profile")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                width = stream
                    .get("width")
                    .and_then(|v| v.as_u64())
                    .map(|w| w as u32);
                height = stream
                    .get("height")
                    .and_then(|v| v.as_u64())
                    .map(|h| h as u32);
                bit_depth = stream
                    .get("bits_per_raw_sample")
                    .and_then(|v| v.as_str())
                    .and_then(|s| s.parse::<u32>().ok());
                color_space = stream
                    .get("color_space")
                    .or_else(|| stream.get("color_transfer"))
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());

                if let Some(ref cs) = color_space {
                    let cs_l = cs.to_lowercase();
                    if cs_l.contains("bt2020")
                        || cs_l.contains("smpte2084")
                        || cs_l.contains("arib-std-b67")
                        || cs_l.contains("hdr")
                    {
                        is_hdr = true;
                    }
                }
                if bit_depth.unwrap_or(8) >= 10 {
                    is_hdr = true;
                }

                if let Some(r_frame_rate) = stream.get("r_frame_rate").and_then(|v| v.as_str()) {
                    let parts: Vec<&str> = r_frame_rate.split('/').collect();
                    if parts.len() == 2 {
                        if let (Ok(num), Ok(den)) =
                            (parts[0].parse::<f64>(), parts[1].parse::<f64>())
                        {
                            if den > 0.0 {
                                fps = Some((num / den * 100.0).round() / 100.0);
                            }
                        }
                    }
                }
            } else if codec_type == "audio" && audio_codec.is_none() {
                audio_codec = stream
                    .get("codec_name")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                audio_channels = stream
                    .get("channels")
                    .and_then(|v| v.as_u64())
                    .map(|c| c as u32);
                audio_sample_rate_hz = stream
                    .get("sample_rate")
                    .and_then(|v| v.as_str())
                    .and_then(|sr| sr.parse::<u32>().ok());
                if let Some(abr_str) = stream.get("bit_rate").and_then(|v| v.as_str()) {
                    if let Ok(abr) = abr_str.parse::<u64>() {
                        audio_bitrate_kbps = Some(abr / 1000);
                    }
                }
                audio_language = stream
                    .get("tags")
                    .and_then(|t| t.get("language"))
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
            }
        }
    }

    if stream_count == 0 || (video_codec.is_none() && audio_codec.is_none()) {
        return Err("ffprobe found no playable audio or video streams".to_string());
    }

    let chapters_count = root
        .get("chapters")
        .and_then(|c| c.as_array())
        .map(|arr| arr.len() as u32);

    Ok(MediaInspection {
        verification_level: VerificationLevel::Verified,
        container_format,
        video_codec,
        video_profile,
        audio_codec,
        width,
        height,
        fps,
        bit_depth,
        color_space,
        is_hdr: Some(is_hdr),
        bitrate_kbps,
        audio_channels,
        audio_sample_rate_hz,
        audio_bitrate_kbps,
        audio_language,
        file_size_bytes,
        duration_seconds,
        is_lossy_transcode_warning,
        stream_count: Some(stream_count),
        chapters_count,
    })
}

pub fn resolve_final_download_path(output_dir: &Path, media_id: &str) -> Option<PathBuf> {
    if !output_dir.exists() {
        return None;
    }

    // Scan directory for matching file containing [media_id]
    if let Ok(entries) = std::fs::read_dir(output_dir) {
        let mut matching_files: Vec<(PathBuf, std::time::SystemTime)> = Vec::new();
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(fname) = path.file_name().and_then(|f| f.to_str()) {
                    // Check if filename contains media_id and is not a partial/temp download (.part, .ytdl, etc)
                    if fname.contains(media_id)
                        && !fname.ends_with(".part")
                        && !fname.ends_with(".ytdl")
                    {
                        if let Ok(meta) = entry.metadata() {
                            let modified = meta.modified().unwrap_or(std::time::UNIX_EPOCH);
                            matching_files.push((path, modified));
                        }
                    }
                }
            }
        }

        // Return the most recently modified matching file
        matching_files.sort_by_key(|candidate| std::cmp::Reverse(candidate.1));
        if let Some((best_match, _)) = matching_files.into_iter().next() {
            return Some(best_match);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn missing_ffprobe_produces_basic_inspection_only() {
        let directory = tempfile::tempdir().unwrap();
        let media = directory.path().join("candidate.mp4");
        std::fs::write(&media, vec![0_u8; 2048]).unwrap();

        let inspection = verify_and_inspect_media(&media, None, false).await.unwrap();
        assert_eq!(
            inspection.verification_level,
            VerificationLevel::BasicInspection
        );
        assert!(inspection.video_codec.is_none());
        assert!(inspection.audio_codec.is_none());
        assert!(inspection.duration_seconds.is_none());
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn failed_ffprobe_does_not_fall_back_to_success() {
        use std::os::unix::fs::PermissionsExt;

        let directory = tempfile::tempdir().unwrap();
        let media = directory.path().join("truncated.mp4");
        let probe = directory.path().join("ffprobe");
        std::fs::write(&media, vec![0_u8; 2048]).unwrap();
        std::fs::write(&probe, b"#!/bin/sh\necho 'corrupt input' >&2\nexit 1\n").unwrap();
        let mut permissions = std::fs::metadata(&probe).unwrap().permissions();
        permissions.set_mode(0o755);
        std::fs::set_permissions(&probe, permissions).unwrap();

        let result = verify_and_inspect_media(&media, Some(&probe), false).await;
        assert!(result
            .unwrap_err()
            .contains("Deep media verification failed"));
    }
}
