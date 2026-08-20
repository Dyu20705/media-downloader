use std::sync::Arc;
use ocmd_core::analyzer::{analyze_media_metadata, parse_ytdlp_json};
use ocmd_core::diagnostics::DiagnosticsBuffer;
use ocmd_core::path_validator::{
    sanitize_file_name, validate_and_ensure_directory,
};
use ocmd_core::presets::compile_download_args;
use ocmd_core::state_machine::DownloadStateMachine;
use ocmd_core::tools::ToolResolver;
use ocmd_core::types::{
    AppSettings, DownloadStatus, MediaKind, PresetType,
};
use ocmd_core::url_validator::validate_media_url;

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
fn test_all_preset_compilation() {
    let settings = AppSettings::default();

    // 1. MP4 Compatible
    let mp4 = compile_download_args(
        PresetType::Mp4Compatible,
        "1080",
        "/downloads",
        "https://example.com/watch?v=123",
        &settings,
    );
    assert_eq!(mp4.output_extension, "mp4");
    assert!(mp4.arguments.contains(&"--merge-output-format".to_string()));
    assert!(mp4.arguments.contains(&"mp4".to_string()));

    // 2. Best Video
    let best_vid = compile_download_args(
        PresetType::BestVideo,
        "auto",
        "/downloads",
        "https://example.com/watch?v=123",
        &settings,
    );
    assert_eq!(best_vid.output_extension, "mkv");
    assert!(best_vid.arguments.contains(&"mkv".to_string()));

    // 3. Best Audio
    let best_aud = compile_download_args(
        PresetType::BestAudio,
        "auto",
        "/downloads",
        "https://example.com/watch?v=123",
        &settings,
    );
    assert!(best_aud.is_audio_only);
    assert!(best_aud.arguments.contains(&"-x".to_string()));

    // 4. MP3
    let mp3 = compile_download_args(
        PresetType::Mp3,
        "auto",
        "/downloads",
        "https://example.com/watch?v=123",
        &settings,
    );
    assert!(mp3.is_audio_only);
    assert!(mp3.arguments.contains(&"mp3".to_string()));

    // 5. FLAC
    let flac = compile_download_args(
        PresetType::Flac,
        "auto",
        "/downloads",
        "https://example.com/watch?v=123",
        &settings,
    );
    assert!(flac.is_audio_only);
    assert!(flac.is_lossy_conversion);
    assert!(flac.arguments.contains(&"flac".to_string()));
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
    assert!(meta.available_resolutions.contains(&2160));
    assert!(meta.available_resolutions.contains(&1080));
    assert!(meta.available_resolutions.contains(&720));
}

#[tokio::test]
async fn test_tool_resolver_and_diagnostics() {
    let resolver = ToolResolver::new();
    let diag = Arc::new(DiagnosticsBuffer::new());

    diag.log("info", "test_source", "Diagnostic initialization test");
    let logs = diag.get_logs();
    assert_eq!(logs.len(), 1);
    assert_eq!(logs[0].message, "Diagnostic initialization test");

    let status = resolver.check_health().await;
    assert_eq!(status.len(), 4);
    assert!(status.iter().any(|t| t.name == "yt-dlp"));
    assert!(status.iter().any(|t| t.name == "ffmpeg"));
    assert!(status.iter().any(|t| t.name == "ffprobe"));
    assert!(status.iter().any(|t| t.name == "mediainfo"));
}
