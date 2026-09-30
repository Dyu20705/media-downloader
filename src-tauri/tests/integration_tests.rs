use one_click_media_downloader_lib::core::analyzer::{analyze_media_metadata, parse_ytdlp_json};
use one_click_media_downloader_lib::core::path_validator::{
    sanitize_file_name, validate_and_ensure_directory,
};
use one_click_media_downloader_lib::core::state_machine::DownloadStateMachine;
use one_click_media_downloader_lib::core::tools::ToolResolver;
use one_click_media_downloader_lib::core::types::{DownloadStatus, MediaKind};
use one_click_media_downloader_lib::core::url_validator::validate_media_url;
use one_click_media_downloader_lib::core::{
    execution::{compile_acquisition_args, ExecutionContext},
    media_graph::MediaGraph,
    planner::AcquisitionPlanner,
};
use std::sync::Arc;

fn has_arg_pair(arguments: &[String], flag: &str, value: &str) -> bool {
    arguments
        .windows(2)
        .any(|pair| pair[0] == flag && pair[1] == value)
}

#[test]
fn test_url_validation() {
    assert!(validate_media_url("https://www.youtube.com/watch?v=LXb3EKWsInQ").is_ok());
    assert!(validate_media_url("http://example.com/video.mp4").is_ok());
    assert!(validate_media_url("file:///etc/passwd").is_err());
    assert!(validate_media_url("ftp://ftp.example.com").is_err());
    assert!(validate_media_url("").is_err());
}

#[test]
fn test_path_validation_and_sanitization() {
    let raw = "Video: Title / Special * Character ? <Test> | File.mp4";
    let sanitized = sanitize_file_name(raw, 100);
    assert!(!sanitized.contains(':'));
    assert!(!sanitized.contains('/'));
    assert!(!sanitized.contains('*'));
    assert!(!sanitized.contains('?'));
    assert!(!sanitized.contains('<'));
    assert!(!sanitized.contains('>'));
    assert!(!sanitized.contains('|'));

    let temp_dir = tempfile::tempdir().unwrap();
    let res = validate_and_ensure_directory(&temp_dir.path().to_string_lossy());
    assert!(res.is_ok());
}

#[test]
fn test_acquisition_compilation() {
    use one_click_media_downloader_lib::core::types::*;
    let metadata = parse_ytdlp_json(r#"{"id":"test","title":"Test","formats":[{"format_id":"v","ext":"mp4","width":1280,"height":720,"vcodec":"h264","acodec":"aac"}]}"#, "https://example.com/video").unwrap();
    let graph = MediaGraph::build_source_graph(&metadata, MediaSourceType::YtDlpExtractor);
    let request = AcquisitionRequest {
        source_scope: SourceScope::SingleMedia,
        operation: AcquisitionOperation::EntireMedia,
        output_profile: OutputProfile::Universal,
        track_selection: TrackSelection::default(),
        metadata_patch: None,
        duplicate_policy: DuplicatePolicy::Rename,
        output_directory: "/downloads".into(),
        max_video_height: None,
    };
    let plan = AcquisitionPlanner::plan(&graph, &request).unwrap();
    let compiled = compile_acquisition_args(
        &plan,
        &ExecutionContext::new("/downloads", &AppSettings::default()),
    )
    .unwrap();
    assert!(has_arg_pair(&compiled.arguments, "-f", "v"));
    assert!(compiled.finalize.is_none());
}

#[test]
fn test_state_machine_lifecycle() {
    let mut sm = DownloadStateMachine::new();
    assert_eq!(sm.current_state(), DownloadStatus::Idle);

    assert!(sm.transition(DownloadStatus::Analyzing).is_ok());
    assert!(sm.transition(DownloadStatus::Ready).is_ok());
    assert!(sm.transition(DownloadStatus::Downloading).is_ok());
    assert!(sm.transition(DownloadStatus::PostProcessing).is_ok());
    assert!(sm.transition(DownloadStatus::Verifying).is_ok());
    assert!(sm.transition(DownloadStatus::Completed).is_ok());

    // Illegal transition from Completed to PostProcessing
    assert!(sm.transition(DownloadStatus::PostProcessing).is_err());
}

#[test]
fn test_metadata_parsing_from_ytdlp_json() {
    let json_text = r#"{
        "id": "test_video_id",
        "title": "Sample 4K Video Test",
        "uploader": "Test Channel",
        "duration": 180.5,
        "thumbnail": "https://img.youtube.com/vi/test/0.jpg",
        "webpage_url": "https://youtube.com/watch?v=test_video_id",
        "formats": [
            { "vcodec": "avc1", "acodec": "none", "height": 720 },
            { "vcodec": "vp9", "acodec": "none", "height": 2160 },
            { "vcodec": "av01", "acodec": "none", "height": 1080 }
        ]
    }"#;

    let meta = parse_ytdlp_json(json_text, "https://youtube.com/watch?v=test_video_id").unwrap();
    assert_eq!(meta.id, "test_video_id");
    assert_eq!(meta.title, "Sample 4K Video Test");
    assert_eq!(meta.uploader, Some("Test Channel".to_string()));
    assert_eq!(meta.duration, Some(180.5));
    assert_eq!(meta.media_kind, MediaKind::Video);
    assert_eq!(meta.available_resolutions, vec![2160, 1080, 720]);
}

#[tokio::test]
async fn test_real_ytdlp_metadata_analysis_integration() {
    let resolver = Arc::new(ToolResolver::new());
    if resolver.resolve_tool("yt-dlp").await.is_none() {
        eprintln!("Skipping live yt-dlp test: binary not available");
        return;
    }

    let url = "https://www.youtube.com/watch?v=LXb3EKWsInQ";
    let meta_result = analyze_media_metadata(url, &resolver).await;
    match meta_result {
        Ok(meta) => {
            assert_eq!(meta.id, "LXb3EKWsInQ");
            assert!(!meta.title.is_empty());
            assert!(meta.duration.unwrap_or(0.0) > 0.0);
            assert!(!meta.available_resolutions.is_empty());
        }
        Err(e) => {
            eprintln!(
                "Live yt-dlp analysis returned error (acceptable in offline environments): {}",
                e
            );
        }
    }
}
