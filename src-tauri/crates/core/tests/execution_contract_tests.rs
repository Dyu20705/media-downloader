use ocmd_core::{
    analyzer::parse_ytdlp_json,
    execution::{compile_acquisition_args, ExecutionContext},
    media_graph::MediaGraph,
    planner::AcquisitionPlanner,
    types::*,
};

fn plan(video: &str, audio: &str, ext: &str) -> AcquisitionPlan {
    let json = serde_json::json!({"id":"fixture","title":"Fixture","duration":1,
        "formats":[{"format_id":"progressive","ext":ext,"width":32,"height":32,
        "vcodec":video,"acodec":audio,"fps":25}]});
    let metadata = parse_ytdlp_json(&json.to_string(), "https://example.com/fixture").unwrap();
    let source = MediaGraph::build_source_graph(&metadata, MediaSourceType::YtDlpExtractor);
    let request = AcquisitionRequest {
        source_scope: SourceScope::SingleMedia,
        operation: AcquisitionOperation::EntireMedia,
        output_profile: OutputProfile::Universal,
        track_selection: TrackSelection::default(),
        metadata_patch: None,
        duplicate_policy: DuplicatePolicy::Rename,
        output_directory: "/tmp".into(),
        max_video_height: None,
    };
    AcquisitionPlanner::plan(&source, &request).unwrap()
}

#[test]
fn progressive_container_change_is_an_explicit_remux() {
    let mp4 = plan("h264", "aac", "mp4");
    assert_eq!(mp4.processing.class, ProcessingClass::SourcePreserved);
    let mkv = plan("h264", "aac", "mkv");
    assert_eq!(mkv.processing.class, ProcessingClass::RemuxOnly);
    let exec = compile_acquisition_args(
        &mkv,
        &ExecutionContext::new("/tmp", &AppSettings::default()),
    )
    .unwrap();
    assert!(exec
        .arguments
        .windows(2)
        .any(|p| p == ["--remux-video", "mp4"]));
    assert!(exec.finalize.is_none());
}

#[tokio::test]
async fn explicit_transforms_produce_planned_codecs_with_real_ffmpeg() {
    if std::process::Command::new("ffmpeg")
        .arg("-version")
        .output()
        .is_err()
        || std::process::Command::new("ffprobe")
            .arg("-version")
            .output()
            .is_err()
    {
        eprintln!("FFmpeg fixture test skipped: install ffmpeg and ffprobe");
        return;
    }
    for (video, audio, class) in [
        ("h264", "opus", ProcessingClass::AudioTranscode),
        ("vp9", "aac", ProcessingClass::VideoTranscode),
        ("vp9", "opus", ProcessingClass::FullTranscode),
    ] {
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("source.mkv");
        let output = dir.path().join("output.mp4");
        let source = std::process::Command::new("ffmpeg")
            .args([
                "-v",
                "error",
                "-f",
                "lavfi",
                "-i",
                "color=size=32x32:rate=25:duration=1",
                "-f",
                "lavfi",
                "-i",
                "sine=frequency=440:duration=1",
                "-c:v",
                if video == "vp9" {
                    "libvpx-vp9"
                } else {
                    "libx264"
                },
                "-c:a",
                if audio == "opus" { "libopus" } else { "aac" },
                "-shortest",
            ])
            .arg(&input)
            .output()
            .unwrap();
        assert!(
            source.status.success(),
            "{}",
            String::from_utf8_lossy(&source.stderr)
        );
        let plan = plan(video, audio, "mkv");
        assert_eq!(plan.processing.class, class);
        let exec = compile_acquisition_args(
            &plan,
            &ExecutionContext::new("/tmp", &AppSettings::default()),
        )
        .unwrap();
        let transformed = std::process::Command::new("ffmpeg")
            .args(exec.finalize.unwrap().arguments(&input, &output))
            .output()
            .unwrap();
        assert!(
            transformed.status.success(),
            "{}",
            String::from_utf8_lossy(&transformed.stderr)
        );
        let actual = ocmd_core::media_verifier::verify_and_inspect_media(
            &output,
            Some(std::path::Path::new("ffprobe")),
            true,
        )
        .await
        .unwrap();
        assert_eq!(actual.video_codec.as_deref(), Some("h264"));
        assert_eq!(actual.audio_codec.as_deref(), Some("aac"));
        assert_eq!((actual.width, actual.height), (Some(32), Some(32)));
        let conformance = ocmd_core::plan_verifier::verify_against_plan(&plan, &actual);
        assert!(conformance.conforms, "{:?}", conformance.mismatches);
        let mut wrong = actual.clone();
        wrong.audio_codec = Some("opus".into());
        wrong.width = Some(1280);
        wrong.duration_seconds = Some(60.0);
        let mismatch = ocmd_core::plan_verifier::verify_against_plan(&plan, &wrong);
        assert!(!mismatch.conforms);
        assert!(mismatch.mismatches.iter().any(|m| m.field == "Audio codec"));
        assert!(mismatch.mismatches.iter().any(|m| m.field == "Width"));
        assert!(mismatch.mismatches.iter().any(|m| m.field == "Duration"));
        let mut missing = actual.clone();
        missing.audio_codec = None;
        assert!(!ocmd_core::plan_verifier::verify_against_plan(&plan, &missing).conforms);
        let mut subtitle_plan = plan.clone();
        subtitle_plan.post_process.subtitle_mode = SubtitleMode::Embed;
        subtitle_plan.selected_streams.subtitle_languages = vec!["en".into()];
        let verification = ocmd_core::plan_verifier::verify_against_plan(&subtitle_plan, &actual);
        assert!(verification
            .mismatches
            .iter()
            .any(|m| m.field == "Subtitle streams"));

        let cover = dir.path().join("cover.jpg");
        let subtitle = dir.path().join("subtitle.vtt");
        let decorated = dir.path().join("decorated.mkv");
        let decorated_output = dir.path().join("decorated.mp4");
        let run = |args: Vec<String>| {
            let result = std::process::Command::new("ffmpeg")
                .args(args)
                .output()
                .unwrap();
            assert!(
                result.status.success(),
                "{}",
                String::from_utf8_lossy(&result.stderr)
            );
        };
        run(vec![
            "-v".into(),
            "error".into(),
            "-f".into(),
            "lavfi".into(),
            "-i".into(),
            "color=c=red:size=32x32".into(),
            "-frames:v".into(),
            "1".into(),
            "-threads".into(),
            "1".into(),
            cover.to_string_lossy().into_owned(),
        ]);
        std::fs::write(&subtitle, "WEBVTT\n\n00:00.000 --> 00:00.500\nCaption\n").unwrap();
        run(vec![
            "-v".into(),
            "error".into(),
            "-i".into(),
            input.to_string_lossy().into_owned(),
            "-i".into(),
            subtitle.to_string_lossy().into_owned(),
            "-map".into(),
            "0".into(),
            "-map".into(),
            "1".into(),
            "-c".into(),
            "copy".into(),
            "-attach".into(),
            cover.to_string_lossy().into_owned(),
            "-metadata:s:t:0".into(),
            "mimetype=image/jpeg".into(),
            decorated.to_string_lossy().into_owned(),
        ]);
        let transform = compile_acquisition_args(
            &plan,
            &ExecutionContext::new("/tmp", &AppSettings::default()),
        )
        .unwrap()
        .finalize
        .unwrap();
        run(transform.arguments(&decorated, &decorated_output));
        let with_embeds = ocmd_core::media_verifier::verify_and_inspect_media(
            &decorated_output,
            Some(std::path::Path::new("ffprobe")),
            true,
        )
        .await
        .unwrap();
        assert_eq!(with_embeds.subtitle_stream_count, 1);
        assert!(
            ocmd_core::plan_verifier::verify_against_plan(&subtitle_plan, &with_embeds).conforms
        );
        let probe = std::process::Command::new("ffprobe")
            .args(["-v", "error", "-show_streams", "-of", "json"])
            .arg(&decorated_output)
            .output()
            .unwrap();
        let streams: serde_json::Value = serde_json::from_slice(&probe.stdout).unwrap();
        assert!(streams["streams"].as_array().unwrap().iter().any(|s| s
            .pointer("/disposition/attached_pic")
            .and_then(|v| v.as_u64())
            == Some(1)));
    }
}

#[test]
fn unknown_fps_and_explicit_audio_language_are_not_fabricated() {
    let json = r#"{"id":"test","formats":[{"format_id":"en","ext":"m4a","acodec":"aac","vcodec":"none","language":"en"},{"format_id":"v","ext":"mp4","width":1280,"height":720,"vcodec":"h264","acodec":"none"}]}"#;
    let metadata = parse_ytdlp_json(json, "https://example.com/video").unwrap();
    let graph = MediaGraph::build_source_graph(&metadata, MediaSourceType::YtDlpExtractor);
    assert_eq!(graph.video_streams[0].fps, None);
    assert_eq!(graph.audio_streams[0].language.as_deref(), Some("en"));
    let request = AcquisitionRequest {
        source_scope: SourceScope::SingleMedia,
        operation: AcquisitionOperation::AudioOnly,
        output_profile: OutputProfile::BestSource,
        track_selection: TrackSelection {
            audio_language: Some("ja".into()),
            ..Default::default()
        },
        metadata_patch: None,
        duplicate_policy: DuplicatePolicy::Rename,
        output_directory: "/tmp".into(),
        max_video_height: None,
    };
    assert!(AcquisitionPlanner::plan(&graph, &request)
        .unwrap_err()
        .contains("language"));
}
