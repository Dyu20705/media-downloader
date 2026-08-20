# Production-Grade External Tool Management System

One-Click Media Downloader provides a zero-configuration, secure, and deterministic media engine management subsystem. A normal user can launch and run the application without manually downloading or configuring binaries in their Windows environment.

---

## 1. Tool Versions Chosen & Supply-Chain Metadata

Every managed binary is strictly pinned with official distribution URLs, exact cryptographic SHA-256 digests, and corresponding open-source licensing. Uncontrolled "latest" floating releases are strictly prohibited.

| Binary | Pinned Version | Architecture | Official Source URL | SHA-256 Digest | License |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **yt-dlp** | `2025.02.19` | Windows x64 | `https://github.com/yt-dlp/yt-dlp/releases/download/2025.02.19/yt-dlp.exe` | `785f73d2a71d7992984ea7c3ea4e1837895e7c8ecba0aa2286e1aafe9d424b91` | Unlicense |
| **FFmpeg** | `7.1` (Essentials) | Windows x64 | `https://github.com/GyanD/codexffmpeg/releases/download/7.1/ffmpeg-7.1-essentials_build.zip` | `a937a00f2771d9d150ae1ae6e61f22eec74b41fb386eaebdfd5ffcfcfbc99d99` | GPL-3.0 / LGPL-2.1+ |
| **FFprobe** | `7.1` (Essentials) | Windows x64 | `https://github.com/GyanD/codexffmpeg/releases/download/7.1/ffmpeg-7.1-essentials_build.zip` | `a937a00f2771d9d150ae1ae6e61f22eec74b41fb386eaebdfd5ffcfcfbc99d99` | GPL-3.0 / LGPL-2.1+ |
| **MediaInfo** | `24.12` (CLI) | Windows x64 | `https://mediaarea.net/download/binary/mediainfo/24.12/MediaInfo_CLI_24.12_Windows_x64.zip` | `f9570aa61fdb930e46124564c7ee23696f4244db919b33a595a882a9341496a7` | BSD-2-Clause |

---

## 2. Storage Layout & Filesystem Architecture

All managed tools reside entirely inside the sandboxed, application-local directory (`%LOCALAPPDATA%\OneClickMediaDownloader\tools\` on Windows, or `tools/` within the portable bundle).

```text
%LOCALAPPDATA%\OneClickMediaDownloader\tools/
├── manifest.json
├── .staging/
│   └── (ephemeral download temporary files: <tool>-<uuid>.tmp)
├── yt-dlp/
│   └── 2025.02.19/
│       └── yt-dlp.exe
├── ffmpeg/
│   └── 7.1/
│       ├── ffmpeg.exe
│       └── ffprobe.exe
└── mediainfo/
    └── 24.12/
        └── mediainfo.exe
```

### Manifest Schema (`manifest.json`)
The manifest records active installed versions, canonical executable paths, cryptographic hashes, and verification timestamps:

```json
{
  "schemaVersion": 1,
  "tools": {
    "yt-dlp": {
      "name": "yt-dlp",
      "version": "2025.02.19",
      "path": "C:\\Users\\<User>\\AppData\\Local\\OneClickMediaDownloader\\tools\\yt-dlp\\2025.02.19\\yt-dlp.exe",
      "sha256": "785f73d2a71d7992984ea7c3ea4e1837895e7c8ecba0aa2286e1aafe9d424b91",
      "installedAt": "2026-08-20T05:28:00.000Z",
      "verified": true
    }
  },
  "lastUpdated": "2026-08-20T05:28:00.000Z"
}
```

---

## 3. Tool Discovery & Bounded Resolution Order

When resolving tool binaries for execution, `ToolResolver` follows a strict, predictable hierarchy:

1. **Explicit Custom Config Path**: User-specified custom executable path in Settings (`customYtdlpPath`, `customFfmpegPath`, etc.).
2. **Project-Local Development Binaries**: `./bin/`, `./tools/` relative to the application working directory.
3. **Bounded Workspace Search**: Workspace root directory traversal bounded to a maximum depth of 3 levels.
4. **System Environment `PATH`**: Standard system binaries if installed on the host machine.
5. **Application-Local Managed Directory**: `%LOCALAPPDATA%\OneClickMediaDownloader\tools\<tool>\<pinnedVersion>\<exe>`.

If a resolved binary fails basic version or execution tests, it is rejected and marked `INVALID`, prompting a single-click repair.

---

## 4. Security Guarantees & Invariants

1. **No System Registry or Global PATH Modifications**: The application never touches `System32`, registry keys, or environment variables.
2. **Deterministic Cryptographic Verification**: Every downloaded binary or archive is validated against its hardcoded SHA-256 digest before extraction or placement.
3. **Atomic Installation & Staging Cleanup**:
   - Downloads stream to an isolated `.staging/` directory.
   - Archives are extracted to staging folders.
   - Binaries are validated before being moved to versioned target directories using atomic filesystem renames (`fs::rename` / `std::fs::rename`).
   - If an error or hash mismatch occurs, staging artifacts are immediately cleaned up.
   - An active binary is never overwritten in place while in use.
4. **No Uncontrolled Background Update Polling**: Updates are never silently fetched in the background. The engine strictly runs verified, pinned versions.

---

## 5. Integration Tests & Verification Suite

The backend test suite (`src-tauri/crates/core/tests/tool_manager_tests.rs`) verifies the complete lifecycle:

- `test_pinned_catalog_integrity`: Validates that all required tools have pinned versions, valid HTTPS sources, and 64-character hex SHA-256 hashes.
- `test_checksum_verification_success_and_tampered`: Verifies that matching SHA-256 hashes pass and altered or corrupt payloads are rejected.
- `test_atomic_installation_and_manifest_persistence`: Asserts that staging files are cleaned up and `manifest.json` is updated atomically.
- `test_invalid_corrupted_binary_repair_flow`: Tests that damaged or non-executable binaries are safely purged and re-installed.
- `test_resolution_order_priority`: Confirms that explicit custom overrides take precedence over system and managed defaults.

---

## 6. Remaining Scope & Operating System Boundaries

- **Target Architecture**: Pinned x64 binaries are optimized for 64-bit Windows environments. Linux and macOS development environments utilize local development binaries or system packages with identical typed IPC interfaces.
- **Single-Job Execution**: Adheres strictly to the Extreme Performance Contract (single concurrent active job, 4 Hz UI telemetry, zero unbounded memory buffering).
