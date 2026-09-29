use ocmd_core::{
    analyzer::parse_ytdlp_json,
    execution::{compile_acquisition_args, ExecutionContext},
    media_graph::MediaGraph,
    planner::AcquisitionPlanner,
    types::*,
};
use serde_json::{json, Value};

fn source(formats: Vec<Value>) -> SourceMediaGraph {
    let metadata = parse_ytdlp_json(
        &json!({
            "id": "review-fixture", "title": "Review fixture", "duration": 10,
            "formats": formats,
        })
        .to_string(),
        "https://example.com/media",
    )
    .unwrap();
    MediaGraph::build_source_graph(&metadata, MediaSourceType::YtDlpExtractor)
}

fn request(profile: OutputProfile, operation: AcquisitionOperation) -> AcquisitionRequest {
    AcquisitionRequest {
        source_scope: SourceScope::SingleMedia,
        operation,
        output_profile: profile,
        track_selection: TrackSelection::default(),
        metadata_patch: None,
        duplicate_policy: DuplicatePolicy::Rename,
        output_directory: "/tmp".into(),
        max_video_height: None,
    }
}

fn compile(plan: &AcquisitionPlan) -> Vec<String> {
    compile_acquisition_args(
        plan,
        &ExecutionContext::new("/tmp", &AppSettings::default()),
    )
    .unwrap()
    .arguments
}

fn value_after<'a>(args: &'a [String], flag: &str) -> Option<&'a str> {
    args.windows(2)
        .find(|pair| pair[0] == flag)
        .map(|pair| pair[1].as_str())
}

fn video_only() -> Value {
    json!({"format_id":"video","ext":"mp4","vcodec":"h264","acodec":"none","width":1920,"height":1080})
}
fn combined() -> Value {
    json!({"format_id":"combined","ext":"mp4","vcodec":"h264","acodec":"aac","width":1280,"height":720,"abr":256,"language":"en"})
}

#[test]
fn combined_audio_is_never_used_as_the_second_half_of_a_merge() {
    let graph = source(vec![video_only(), combined()]);
    assert!(!graph.audio_streams[0].is_audio_only);
    for profile in [
        OutputProfile::BestSource,
        OutputProfile::Small,
        OutputProfile::Universal,
        OutputProfile::Editing,
    ] {
        let plan =
            AcquisitionPlanner::plan(&graph, &request(profile, AcquisitionOperation::EntireMedia))
                .unwrap();
        assert_eq!(
            plan.selected_streams.video_stream_id.as_deref(),
            Some("combined")
        );
        assert_eq!(
            plan.selected_streams.audio_stream_id.as_deref(),
            Some("combined")
        );
        assert_eq!(plan.output.height, Some(720));
        assert!(!plan.transforms.contains(&PlannedTransform::Merge));
        assert_eq!(value_after(&compile(&plan), "-f"), Some("combined"));
    }
}

#[test]
fn separate_video_pairs_only_with_an_explicit_audio_only_format() {
    let graph = source(vec![
        video_only(),
        combined(),
        json!({
            "format_id":"audio","ext":"m4a","vcodec":"none","acodec":"aac","abr":128,"language":"en"
        }),
    ]);
    assert!(
        graph
            .audio_streams
            .iter()
            .find(|a| a.stream_id == "audio")
            .unwrap()
            .is_audio_only
    );
    let plan = AcquisitionPlanner::plan(
        &graph,
        &request(OutputProfile::BestSource, AcquisitionOperation::EntireMedia),
    )
    .unwrap();
    assert_eq!(value_after(&compile(&plan), "-f"), Some("video+audio"));
    assert!(plan.transforms.contains(&PlannedTransform::Merge));
}

#[test]
fn audio_extraction_can_use_a_combined_source_but_unknown_video_is_not_mergeable() {
    let graph = source(vec![
        video_only(),
        combined(),
        json!({
            "format_id":"unknown","ext":"m4a","acodec":"aac","abr":512
        }),
    ]);
    assert!(
        !graph
            .audio_streams
            .iter()
            .find(|a| a.stream_id == "unknown")
            .unwrap()
            .is_audio_only
    );
    let plan = AcquisitionPlanner::plan(
        &graph,
        &request(OutputProfile::BestSource, AcquisitionOperation::EntireMedia),
    )
    .unwrap();
    assert_eq!(value_after(&compile(&plan), "-f"), Some("combined"));
    let graph = source(vec![combined()]);
    let plan = AcquisitionPlanner::plan(
        &graph,
        &request(OutputProfile::BestSource, AcquisitionOperation::AudioOnly),
    )
    .unwrap();
    assert_eq!(value_after(&compile(&plan), "-f"), Some("combined"));
    assert!(compile(&plan).iter().any(|a| a == "-x"));
}

#[test]
fn combined_fallback_respects_explicit_audio_language() {
    let mut ja = combined();
    ja["format_id"] = json!("combined-ja");
    ja["language"] = json!("ja");
    ja["height"] = json!(480);
    let graph = source(vec![video_only(), combined(), ja]);
    let mut req = request(OutputProfile::BestSource, AcquisitionOperation::EntireMedia);
    req.track_selection.audio_language = Some("ja".into());
    let plan = AcquisitionPlanner::plan(&graph, &req).unwrap();
    assert_eq!(value_after(&compile(&plan), "-f"), Some("combined-ja"));
}

fn with_subtitles() -> SourceMediaGraph {
    let mut graph = source(vec![combined()]);
    graph.subtitle_streams = [
        ("en", false),
        ("en-US", false),
        ("ja", false),
        ("live_chat", false),
        ("fr", true),
        ("en", true),
    ]
    .into_iter()
    .map(|(language, is_auto)| SubtitleTrack {
        language: language.into(),
        is_auto: Some(is_auto),
        ext: Some("vtt".into()),
        ..Default::default()
    })
    .collect();
    graph
}

#[test]
fn ordinary_video_subtitle_settings_resolve_into_concrete_plan_tracks() {
    let mut graph = with_subtitles();
    graph.chapters.push(MediaChapter {
        title: "Opening".into(),
        start_time: 0.0,
        end_time: 1.0,
    });
    for operation in [
        AcquisitionOperation::EntireMedia,
        AcquisitionOperation::Chapter { chapter_index: 0 },
        AcquisitionOperation::Clip {
            start_ms: 0,
            end_ms: 1000,
        },
    ] {
        for mode in [SubtitleMode::Embed, SubtitleMode::DownloadSeparate] {
            let settings = AppSettings {
                subtitle_mode: mode,
                preferred_subtitle_language: "en.*,ja".into(),
                ..Default::default()
            };
            let req = request(OutputProfile::BestSource, operation.clone());
            let plan =
                AcquisitionPlanner::plan_with_policy(&graph, &req, (&settings).into()).unwrap();
            assert_eq!(
                plan.selected_streams.subtitle_languages,
                ["en", "en-US", "ja"]
            );
            let args = compile(&plan);
            assert_eq!(value_after(&args, "--sub-langs"), Some("en,en-US,ja"));
            assert!(args.iter().any(|a| a
                == if mode == SubtitleMode::Embed {
                    "--embed-subs"
                } else {
                    "--write-subs"
                }));
            assert!(!args.iter().any(|a| a == "en.*,ja"));
        }
    }
}

#[test]
fn subtitle_preference_exclusions_and_auto_track_policy_are_resolved_before_compile() {
    let graph = with_subtitles();
    let resolve = |pattern, auto| {
        ocmd_core::subtitle_selection::resolve_subtitle_preference(
            pattern,
            &graph.subtitle_streams,
            auto,
        )
    };
    assert_eq!(resolve("all,-live_chat,-en.*", false).unwrap(), ["ja"]);
    assert_eq!(
        resolve("all,-live_chat", true).unwrap(),
        ["en", "en-US", "fr", "ja"]
    );
    assert_eq!(resolve("en", false).unwrap(), ["en"]);
    assert_eq!(resolve("", false).unwrap(), ["en"]);
    assert!(resolve("(?=en)en", false).is_err());
    assert!(resolve("en,,ja", false).is_err());
    assert!(resolve(&"e".repeat(2049), false).is_err());
}

#[test]
fn subtitle_preferences_are_visible_in_plan_identity_and_do_not_override_explicit_tracks() {
    let graph = with_subtitles();
    let req = request(OutputProfile::BestSource, AcquisitionOperation::EntireMedia);
    let mut policy = PostProcessPolicy {
        subtitle_mode: SubtitleMode::Embed,
        preferred_subtitle_language: "en".into(),
        ..Default::default()
    };
    let first = AcquisitionPlanner::plan_with_policy(&graph, &req, policy.clone()).unwrap();
    policy.preferred_subtitle_language = "ja".into();
    let next = AcquisitionPlanner::plan_with_policy(&graph, &req, policy.clone()).unwrap();
    assert_ne!(first.id, next.id);
    assert_eq!(next.selected_streams.subtitle_languages, ["ja"]);
    let mut explicit = req.clone();
    explicit.track_selection.subtitle_languages = vec!["en-US".into()];
    assert_eq!(
        AcquisitionPlanner::plan_with_policy(&graph, &explicit, policy.clone())
            .unwrap()
            .selected_streams
            .subtitle_languages,
        ["en-US"]
    );
    policy.preferred_subtitle_language = "de".into();
    let unavailable = AcquisitionPlanner::plan_with_policy(&graph, &req, policy.clone()).unwrap();
    assert!(unavailable
        .warnings
        .iter()
        .any(|w| w.code == "SUBTITLE_PREFERENCE_UNAVAILABLE"));
    assert!(unavailable.selected_streams.subtitle_languages.is_empty());
    policy.subtitle_mode = SubtitleMode::None;
    assert!(
        !compile(&AcquisitionPlanner::plan_with_policy(&graph, &req, policy).unwrap())
            .iter()
            .any(|a| a == "--sub-langs")
    );
}

#[test]
fn source_audio_profiles_only_produce_executable_extraction_formats() {
    for profile in [OutputProfile::BestSource, OutputProfile::Small] {
        for (codec, ext, container, encoder) in [
            ("mp4a.40.5", "m4a", "m4a", "m4a"),
            ("mp4a.40.29", "aac", "aac", "aac"),
            ("AAC", "mp4", "m4a", "m4a"),
            ("opus", "webm", "opus", "opus"),
            ("vorbis", "webm", "ogg", "vorbis"),
            ("flac", "flac", "flac", "flac"),
            ("mp3", "mp3", "mp3", "mp3"),
        ] {
            let graph = source(vec![json!({
                "format_id":"audio","ext":ext,"vcodec":"none","acodec":codec
            })]);
            let plan = AcquisitionPlanner::plan(
                &graph,
                &request(profile, AcquisitionOperation::AudioOnly),
            )
            .unwrap();
            assert_eq!(plan.output.container, container, "{codec}");
            assert_eq!(plan.output.audio_codec.as_deref(), Some(codec));
            assert_eq!(
                value_after(&compile(&plan), "--audio-format"),
                Some(encoder)
            );
        }
    }
}

#[test]
fn unsupported_preserved_audio_is_rejected_before_a_plan_is_presented() {
    let graph = source(vec![
        json!({"format_id":"audio","ext":"mka","vcodec":"none","acodec":"dts"}),
    ]);
    for profile in [OutputProfile::BestSource, OutputProfile::Small] {
        let error =
            AcquisitionPlanner::plan(&graph, &request(profile, AcquisitionOperation::AudioOnly))
                .unwrap_err();
        assert!(
            error.contains("Source-preserving audio extraction"),
            "{error}"
        );
    }
    for profile in [OutputProfile::Universal, OutputProfile::Editing] {
        let plan =
            AcquisitionPlanner::plan(&graph, &request(profile, AcquisitionOperation::AudioOnly))
                .unwrap();
        assert!(value_after(&compile(&plan), "--audio-format").is_some());
    }
}

#[test]
fn subtitle_only_plan_declares_conversion_and_ffmpeg_preflight() {
    let mut graph = with_subtitles();
    graph.subtitle_streams[0].ext = Some("srt".into());
    let mut req = request(
        OutputProfile::BestSource,
        AcquisitionOperation::SubtitlesOnly,
    );
    req.track_selection.subtitle_languages = vec!["en".into()];
    for plan in [
        AcquisitionPlanner::plan(&graph, &req).unwrap(),
        AcquisitionPlanner::plan_with_policy(&graph, &req, PostProcessPolicy::default()).unwrap(),
    ] {
        assert_eq!(plan.output.container, "vtt");
        assert!(plan.processing.requires_ffmpeg);
        assert!(plan.requirements.iter().any(|r| r.code == "FFMPEG"));
        assert!(plan
            .transforms
            .contains(&PlannedTransform::ConvertSubtitles {
                format: "vtt".into()
            }));
        let args = compile(&plan);
        assert_eq!(value_after(&args, "--sub-format"), Some("vtt/best"));
        assert_eq!(value_after(&args, "--convert-subs"), Some("vtt"));
    }
}

#[tokio::test]
async fn srt_only_source_is_converted_to_the_promised_vtt_artifact_offline() {
    for tool in ["yt-dlp", "ffmpeg"] {
        let available = std::process::Command::new(tool)
            .arg(if tool == "yt-dlp" {
                "--version"
            } else {
                "-version"
            })
            .output()
            .is_ok_and(|result| result.status.success());
        if !available {
            assert!(std::env::var_os("CI").is_none(), "CI requires {tool}");
            eprintln!("Offline subtitle test skipped: install {tool}");
            return;
        }
    }
    let dir = tempfile::tempdir().unwrap();
    let srt = dir.path().join("source.srt");
    std::fs::write(
        &srt,
        "1\n00:00:00,000 --> 00:00:01,000\nRegression caption\n",
    )
    .unwrap();
    let info = json!({
        "id":"srt-only", "title":"SRT only", "extractor":"generic", "extractor_key":"Generic",
        "webpage_url":"https://example.com/srt-only", "url":"https://example.com/not-downloaded.mp4", "ext":"mp4",
        "subtitles":{"en":[{"ext":"srt","url":url::Url::from_file_path(&srt).unwrap().as_str()}]}
    });
    let metadata = parse_ytdlp_json(&info.to_string(), "https://example.com/srt-only").unwrap();
    let graph = MediaGraph::build_source_graph(&metadata, MediaSourceType::YtDlpExtractor);
    let mut req = request(
        OutputProfile::BestSource,
        AcquisitionOperation::SubtitlesOnly,
    );
    req.track_selection.subtitle_languages = vec!["en".into()];
    let plan =
        AcquisitionPlanner::plan_with_policy(&graph, &req, PostProcessPolicy::default()).unwrap();
    let mut args = compile_acquisition_args(
        &plan,
        &ExecutionContext::new(dir.path().to_str().unwrap(), &AppSettings::default()),
    )
    .unwrap()
    .arguments;
    assert_eq!(args.pop().as_deref(), Some(plan.source.url.as_str()));
    let info_path = dir.path().join("source.info.json");
    std::fs::write(&info_path, info.to_string()).unwrap();
    // File URLs are enabled only in this offline fixture, never in production arguments.
    args.extend([
        "--enable-file-urls".into(),
        "--load-info-json".into(),
        info_path.to_string_lossy().into_owned(),
    ]);
    let result = tokio::time::timeout(
        std::time::Duration::from_secs(30),
        tokio::process::Command::new("yt-dlp")
            .kill_on_drop(true)
            .args(args)
            .output(),
    )
    .await
    .unwrap()
    .unwrap();
    assert!(
        result.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
    let artifact = dir.path().join("artifact.en.vtt");
    assert!(
        artifact.exists(),
        "{}",
        String::from_utf8_lossy(&result.stdout)
    );
    let actual = ocmd_core::media_verifier::verify_subtitle(&artifact)
        .await
        .unwrap();
    assert_eq!(actual.container_format, "vtt");
    assert!(ocmd_core::plan_verifier::verify_against_plan(&plan, &actual).conforms);
    assert!(std::fs::read_to_string(artifact)
        .unwrap()
        .contains("Regression caption"));
}
