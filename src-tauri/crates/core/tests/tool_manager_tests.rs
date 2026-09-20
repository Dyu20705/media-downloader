use std::fs;
use std::sync::Arc;
use tempfile::tempdir;

use ocmd_core::diagnostics::DiagnosticsBuffer;
use ocmd_core::tool_manager::{get_pinned_tool_spec, ToolManager, PINNED_TOOLS};
use ocmd_core::types::{AppSettings, ToolStatus};

#[test]
fn test_pinned_versions_catalog() {
    assert_eq!(PINNED_TOOLS.len(), 4);

    let ytdlp = get_pinned_tool_spec("yt-dlp").expect("yt-dlp must be pinned");
    assert_eq!(ytdlp.pinned_version, "2025.02.19");
    assert!(ytdlp.is_required);
    assert_eq!(ytdlp.license, "Unlicense");

    let ffmpeg = get_pinned_tool_spec("ffmpeg").expect("ffmpeg must be pinned");
    assert_eq!(ffmpeg.pinned_version, "7.1");
    assert!(ffmpeg.is_required);
    assert!(ffmpeg.license.contains("GPL"));

    let ffprobe = get_pinned_tool_spec("ffprobe").expect("ffprobe must be pinned");
    assert_eq!(ffprobe.pinned_version, "7.1");
    assert!(ffprobe.is_required);

    let mediainfo = get_pinned_tool_spec("mediainfo").expect("mediainfo must be pinned");
    assert_eq!(mediainfo.pinned_version, "24.12");
    assert!(!mediainfo.is_required);
    assert_eq!(mediainfo.license, "BSD-2-Clause");
}

#[test]
fn test_checksum_verification_valid_and_tampered() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test_binary.bin");

    // Write deterministic content
    let content = b"one-click-media-downloader-safe-executable-test";
    fs::write(&file_path, content).unwrap();

    let computed_hash = ToolManager::compute_sha256(&file_path).unwrap();
    assert!(!computed_hash.is_empty());

    // Valid hash passes
    assert!(ToolManager::verify_sha256(&file_path, &computed_hash).unwrap());

    // Case-insensitive hex matching
    assert!(ToolManager::verify_sha256(&file_path, &computed_hash.to_uppercase()).unwrap());

    // Tampered hash or corrupt file fails
    let tampered_hash = "0000000000000000000000000000000000000000000000000000000000000000";
    assert!(!ToolManager::verify_sha256(&file_path, tampered_hash).unwrap());
}

#[tokio::test]
async fn test_installation_atomicity_and_staging_cleanup() {
    let tools_dir = tempdir().unwrap();
    let diag = Arc::new(DiagnosticsBuffer::new());
    let manager = ToolManager::new(Some(tools_dir.path().to_path_buf()), diag);

    // Initial state before managed install must not be managed
    let status_before = manager.check_tool_status("yt-dlp", None).await;
    assert!(
        !status_before.managed,
        "Tool should not be managed before install"
    );

    // Create a mock executable script/binary
    let mock_binary: &[u8] = if cfg!(windows) {
        b"@echo off\r\necho 2025.02.19\r\n"
    } else {
        b"#!/bin/sh\necho 2025.02.19\n"
    };

    let mock_hash = {
        let tmp = tempdir().unwrap();
        let p = tmp.path().join("mock.tmp");
        fs::write(&p, mock_binary).unwrap();
        ToolManager::compute_sha256(&p).unwrap()
    };

    // Install using atomic byte installer
    let install_result = manager
        .install_from_bytes("yt-dlp", mock_binary, &mock_hash, false)
        .await;
    assert!(
        install_result.is_ok(),
        "Installation should succeed: {:?}",
        install_result.err()
    );

    let status_after = install_result.unwrap();
    assert_eq!(status_after.status, ToolStatus::Ready);
    assert!(status_after.managed);
    assert_eq!(status_after.pinned_version, "2025.02.19");

    // Verify staging directory is cleaned up
    let staging_dir = manager.get_staging_dir();
    if staging_dir.exists() {
        let entries: Vec<_> = fs::read_dir(&staging_dir).unwrap().collect();
        assert_eq!(
            entries.len(),
            0,
            "Staging directory must be empty after installation"
        );
    }

    // Verify manifest.json was atomically committed
    let manifest = manager.load_manifest();
    assert!(manifest.tools.contains_key("yt-dlp"));
    let entry = manifest.tools.get("yt-dlp").unwrap();
    assert_eq!(entry.version, "2025.02.19");
    assert!(entry.verified);
    assert_eq!(entry.sha256, mock_hash);
}

#[tokio::test]
async fn test_corrupted_binary_rejection_and_staging_safety() {
    let tools_dir = tempdir().unwrap();
    let diag = Arc::new(DiagnosticsBuffer::new());
    let manager = ToolManager::new(Some(tools_dir.path().to_path_buf()), diag);

    let mock_binary = b"corrupted payload with wrong checksum";
    let expected_hash = "1111222233334444555566667777888899990000aaaabbbbccccddddeeeeffff";

    // Attempt install with mismatching hash
    let result = manager
        .install_from_bytes("yt-dlp", mock_binary, expected_hash, false)
        .await;
    assert!(
        result.is_err(),
        "Installation with wrong checksum must fail"
    );

    // Verify corrupt file NEVER became active
    let manifest = manager.load_manifest();
    assert!(
        !manifest.tools.contains_key("yt-dlp"),
        "Corrupt install must not be in manifest"
    );

    let status = manager.check_tool_status("yt-dlp", None).await;
    assert!(
        !status.managed,
        "Corrupt binary must not be marked as managed"
    );

    // Verify staging temp files are cleaned up
    let staging_dir = manager.get_staging_dir();
    if staging_dir.exists() {
        let entries: Vec<_> = fs::read_dir(&staging_dir).unwrap().collect();
        assert_eq!(entries.len(), 0, "Staging must be cleaned up after error");
    }
}

#[tokio::test]
async fn test_resolution_order_priority() {
    let temp_tools = tempdir().unwrap();
    let diag = Arc::new(DiagnosticsBuffer::new());
    let manager = ToolManager::new(Some(temp_tools.path().to_path_buf()), diag);

    // 1. Test explicit configured path in AppSettings
    let explicit_dir = tempdir().unwrap();
    let custom_exe_name = if cfg!(windows) {
        "custom_yt.exe"
    } else {
        "custom_yt"
    };
    let custom_exe_path = explicit_dir.path().join(custom_exe_name);

    if cfg!(windows) {
        fs::write(&custom_exe_path, b"@echo off\r\necho 2025.02.19\r\n").unwrap();
    } else {
        fs::write(&custom_exe_path, b"#!/bin/sh\necho 2025.02.19\n").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&custom_exe_path).unwrap().permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&custom_exe_path, perms).unwrap();
        }
    }

    let settings = AppSettings {
        custom_ytdlp_path: Some(custom_exe_path.to_string_lossy().to_string()),
        ..AppSettings::default()
    };

    let resolved = manager.resolve_tool("yt-dlp", Some(&settings)).await;
    assert!(resolved.is_some());
    let tool = resolved.unwrap();
    assert_eq!(tool.path, custom_exe_path);
    assert!(!tool.is_managed, "Explicit path is not managed");
}

#[tokio::test]
async fn test_reinstall_and_repair_flow() {
    let tools_dir = tempdir().unwrap();
    let diag = Arc::new(DiagnosticsBuffer::new());
    let manager = ToolManager::new(Some(tools_dir.path().to_path_buf()), diag);

    let mock_binary: &[u8] = if cfg!(windows) {
        b"@echo off\r\necho 2025.02.19\r\n"
    } else {
        b"#!/bin/sh\necho 2025.02.19\n"
    };

    let mock_hash = {
        let tmp = tempdir().unwrap();
        let p = tmp.path().join("mock.tmp");
        fs::write(&p, mock_binary).unwrap();
        ToolManager::compute_sha256(&p).unwrap()
    };

    // Install initial version
    manager
        .install_from_bytes("yt-dlp", mock_binary, &mock_hash, false)
        .await
        .unwrap();
    let status_1 = manager.check_tool_status("yt-dlp", None).await;
    assert_eq!(status_1.status, ToolStatus::Ready);

    // Corrupt active binary
    let target_dir = manager.get_version_dir("yt-dlp", "2025.02.19");
    let bin_name = if cfg!(windows) {
        "yt-dlp.exe"
    } else {
        "yt-dlp"
    };
    let active_bin = target_dir.join(bin_name);
    fs::write(&active_bin, b"invalid corrupted data").unwrap();
    manager.clear_cache();

    // Check status detects invalid binary
    let status_corrupted = manager.check_tool_status("yt-dlp", None).await;
    assert_eq!(status_corrupted.status, ToolStatus::Invalid);
    assert!(status_corrupted.error_message.is_some());

    // Repair tool
    let repair_res = manager
        .install_from_bytes("yt-dlp", mock_binary, &mock_hash, false)
        .await;
    assert!(repair_res.is_ok());
    let status_repaired = manager.check_tool_status("yt-dlp", None).await;
    assert_eq!(status_repaired.status, ToolStatus::Ready);
}
