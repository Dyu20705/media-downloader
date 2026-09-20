use std::sync::Arc;
use ocmd_core::analyzer::parse_ytdlp_json;
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
    assert!(has_arg_pair(&mp4.arguments, "--merge-output-format", "mp4"));

    // 2. Best Video
    let best_vid = compile_download_args(
        PresetType::BestVideo,
        "auto",
        "/downloads",
        "https://example.com/watch?v=123",
        &settings,
    );
    assert!(has_arg_pair(&best_vid.arguments, "-f", "bv+ba/b"));
    assert!(has_arg_pair(
        &best_vid.arguments,
        "--merge-output-format",
        "mkv"
    ));

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
    assert!(has_arg_pair(&best_aud.arguments, "-f", "bestaudio/b"));
    assert!(!best_aud.arguments.contains(&"--audio-format".to_string()));
    assert!(!best_aud.is_lossy_conversion);

    // 4. MP3
    let mp3 = compile_download_args(
        PresetType::Mp3,
        "auto",
        "/downloads",
        "https://example.com/watch?v=123",
        &settings,
    );
    assert!(mp3.is_audio_only);
    assert!(mp3.is_lossy_conversion);
    assert!(has_arg_pair(&mp3.arguments, "--audio-format", "mp3"));
    assert!(has_arg_pair(&mp3.arguments, "--audio-quality", "0"));

    // 5. FLAC
    let flac = compile_download_args(
        PresetType::Flac,
        "auto",
        "/downloads",
        "https://example.com/watch?v=123",
        &settings,
    );
    assert!(flac.is_audio_only);
    assert!(!flac.is_lossy_conversion);
    assert!(has_arg_pair(&flac.arguments, "--audio-format", "flac"));
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

#[test]
fn test_cross_platform_metadata_extraction() {
    // 1. TikTok style payload (creator instead of uploader, thumbnail array)
    let tiktok_json = r#"{
        "id": "718291029102",
        "title": "",
        "description": "Fun dancing video on the beach #summer #fun",
        "creator": "dance_star",
        "thumbnails": [
            { "id": "sb0", "url": "https://p16.tiktokcdn.com/storyboard.jpg", "width": 100 },
            { "id": "thumb_low", "url": "https://p16.tiktokcdn.com/low.jpg", "width": 320, "height": 480 },
            { "id": "thumb_high", "url": "https://p16.tiktokcdn.com/high.jpg", "width": 720, "height": 1280 }
        ]
    }"#;
    let meta_tiktok = parse_ytdlp_json(tiktok_json, "https://www.tiktok.com/@dance_star/video/718291029102").unwrap();
    assert_eq!(meta_tiktok.uploader, Some("dance_star".to_string()));
    assert_eq!(meta_tiktok.title, "Fun dancing video on the beach #summer #fun");
    assert_eq!(meta_tiktok.thumbnail, Some("https://p16.tiktokcdn.com/high.jpg".to_string()));

    // 2. SoundCloud style payload (artist instead of uploader, track as title)
    let sc_json = r#"{
        "id": "sc_12345",
        "track": "Midnight Chill Lofi Beat",
        "artist": "Lofi Producer",
        "thumbnail": "https://i1.sndcdn.com/artworks-000123-t500x500.jpg"
    }"#;
    let meta_sc = parse_ytdlp_json(sc_json, "https://soundcloud.com/lofi/midnight").unwrap();
    assert_eq!(meta_sc.uploader, Some("Lofi Producer".to_string()));
    assert_eq!(meta_sc.title, "Midnight Chill Lofi Beat");

    // 3. Instagram style payload (generic title with description)
    let ig_json = r#"{
        "id": "ig_98765",
        "title": "Instagram post by photog",
        "description": "Sunset golden hour over Mount Fuji.\nCaptured with 35mm lens.",
        "channel": "photog_official",
        "thumbnail": "https://instagram.com/p/photo.jpg"
    }"#;
    let meta_ig = parse_ytdlp_json(ig_json, "https://instagram.com/p/ig_98765").unwrap();
    assert_eq!(meta_ig.uploader, Some("photog_official".to_string()));
    assert_eq!(meta_ig.title, "Sunset golden hour over Mount Fuji.");
}
