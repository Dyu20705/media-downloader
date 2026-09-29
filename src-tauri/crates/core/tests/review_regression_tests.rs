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
