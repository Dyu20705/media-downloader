use crate::types::{
    AcquisitionOperation, AcquisitionPlan, MediaInspection, PlanMismatch, PlanVerification,
    SponsorBlockMode,
};

fn codec(value: &str) -> &str {
    match value.split('.').next().unwrap_or(value) {
        "avc1" | "avc3" | "h264" => "h264",
        "hev1" | "hvc1" | "hevc" | "h265" => "hevc",
        "vp09" | "vp9" => "vp9",
        "av01" | "av1" => "av1",
        "mp4a" | "aac" => "aac",
        other => other,
    }
}

fn container_matches(expected: &str, actual: &str) -> bool {
    let aliases: &[&str] = match expected {
        "mp4" | "m4a" | "mov" => &["mov", "mp4", "m4a"],
        "mkv" | "mka" | "webm" => &["matroska", "webm"],
        "jpg" | "jpeg" => &["image2", "jpeg_pipe", "mjpeg", "jpg", "jpeg"],
        "opus" | "ogg" => &["ogg", "opus"],
        _ => return actual.split(',').any(|a| a == expected),
    };
    actual.split(',').any(|a| aliases.contains(&a))
}

pub fn verify_against_plan(plan: &AcquisitionPlan, actual: &MediaInspection) -> PlanVerification {
    let mut mismatches = Vec::new();
    let mut compare = |field: &str, planned: String, found: Option<String>, matches: bool| {
        if !matches {
            mismatches.push(PlanMismatch {
                field: field.into(),
                planned,
                actual: found,
            });
        }
    };
    compare(
        "Container",
        plan.output.container.clone(),
        Some(actual.container_format.clone()),
        container_matches(
            &plan.output.container.to_lowercase(),
            &actual.container_format.to_lowercase(),
        ),
    );
    for (field, expected, found) in [
        (
            "Video codec",
            plan.output.video_codec.as_deref(),
            actual.video_codec.as_deref(),
        ),
        (
            "Audio codec",
            plan.output.audio_codec.as_deref(),
            actual.audio_codec.as_deref(),
        ),
    ] {
        if let Some(expected) = expected {
            compare(
                field,
                expected.into(),
                found.map(str::to_owned),
                found.is_some_and(|v| codec(&v.to_lowercase()) == codec(&expected.to_lowercase())),
            );
        }
    }
    if plan.output.audio_only {
        compare(
            "Video stream",
            "None".into(),
            actual.video_codec.clone(),
            actual.video_codec.is_none(),
        );
    }
    for (field, expected, found) in [
        ("Width", plan.output.width, actual.width),
        ("Height", plan.output.height, actual.height),
    ] {
        if let Some(expected) = expected {
            compare(
                field,
                expected.to_string(),
                found.map(|v| v.to_string()),
                found == Some(expected),
            );
        }
    }
    if let Some(expected) = plan.output.fps {
        compare(
            "Frame rate",
            expected.to_string(),
            actual.fps.map(|v| v.to_string()),
            actual.fps.is_some_and(|v| (v - expected).abs() <= 0.1),
        );
    }
    let sidecar = matches!(
        plan.operation,
        AcquisitionOperation::ThumbnailOnly | AcquisitionOperation::SubtitlesOnly
    );
    let mut warnings = Vec::new();
    if !sidecar {
        if plan.post_process.sponsor_block_mode == SponsorBlockMode::RemoveSegments {
            warnings.push(
                "Duration comparison skipped: SponsorBlock removes content-dependent segments."
                    .into(),
            );
        } else {
            let expected = plan
                .time_range_ms
                .map(|[start, end]| (end - start) as f64 / 1000.0)
                .or(plan.source.duration_seconds);
            if let Some(expected) = expected {
                // Fast cuts are not frame accurate. The tolerance is visible, not silently unlimited.
                let tolerance = if plan.time_range_ms.is_some() {
                    2.0
                } else {
                    (expected * 0.01).max(1.0)
                };
                compare(
                    "Duration",
                    format!("{expected:.3}s ± {tolerance:.3}s"),
                    actual.duration_seconds.map(|v| format!("{v:.3}s")),
                    actual
                        .duration_seconds
                        .is_some_and(|v| v.is_finite() && (v - expected).abs() <= tolerance),
                );
                if plan.time_range_ms.is_some() {
                    warnings.push("Fast-cut duration tolerance: ±2 seconds.".into());
                }
            }
        }
    }
    PlanVerification {
        conforms: mismatches.is_empty(),
        mismatches,
        warnings,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn codec_and_container_aliases_are_explicit() {
        assert_eq!(codec("avc1.640028"), "h264");
        assert_eq!(codec("mp4a.40.2"), "aac");
        assert!(container_matches("mp4", "mov,mp4,m4a,3gp,3g2,mj2"));
        assert!(!container_matches("mp4", "matroska,webm"));
        assert!(!container_matches("mp4", "notmp4"));
    }
}
