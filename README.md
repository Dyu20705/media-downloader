# One-Click Media Downloader

> Production-grade desktop media downloader powered by Tauri 2, Rust, React 18, yt-dlp, FFmpeg, and MediaInfo.

---

## 1. What the Product Does

One-Click Media Downloader is a high-performance desktop application designed for fast media extraction and conversion from supported media platforms:
- **Clean Single-View Workflow**: Paste URL → Choose Preset → Pick Resolution & Save Folder → Download.
- **Engine Tools Automation**: Auto-detects, downloads, and cryptographically verifies `yt-dlp`, `FFmpeg`, `FFprobe`, and `MediaInfo` binaries without touching system environment variables.
- **Rich Presets**:
  - **MP4 (Compatible)**: H.264/AAC for universal playback.
  - **Best Video**: Highest available stream quality (up to 4K/8K MKV).
  - **Best Audio**: Pure untouched source audio stream (Opus/AAC).
  - **MP3 (Universal)**: 320 kbps CBR MP3.
  - **FLAC (Lossless)**: Lossless FLAC container packaging.
- **Deep Media Verification**: Real-time inspection via `ffprobe` and `MediaInfo` confirming valid video/audio streams, duration, bitrate, and format headers.

---

## 2. Architecture & Tech Stack

- **Frontend**: React 18, TypeScript, Tailwind CSS, Lucide React, Vite.
- **Desktop Host**: Tauri 2 (Rust).
- **Core Engine (Rust)**:
  - `DownloadStateMachine`: Strict state validation (`IDLE` → `ANALYZING` → `DOWNLOADING` → `POST_PROCESSING` → `VERIFYING` → `COMPLETED`).
  - `ToolResolver & ToolManager`: Atomic binary staging, SHA-256 verification, bounded resolution search.
  - `ProcessRunner`: Job-object process tree ownership on Windows, process group isolation on Unix.
  - `DiagnosticsBuffer`: Safe 256-line ring buffer capped at <= 64 KiB with automated credential/token redaction.

---

## 3. How to Build & Run

### Prerequisites
- Node.js 18+ & npm / bun
- Rust 1.75+ (for native Tauri desktop build)
- Windows 10/11 x64 (for native installer generation)

### Development
```bash
# 1. Install dependencies
npm install

# 2. Run in development mode (with Vite + Dev Server)
npm run dev

# 3. Run Tauri desktop in development
npm run tauri dev
```

### Production Build
```bash
# Build web assets and bundle
npm run build

# Build Windows Release Executable and NSIS Installer
npm run tauri build
```

---

## 4. External Tool Supply Chain

All managed third-party binaries are pinned and verified against cryptographic SHA-256 digests:

| Binary | Version | SHA-256 Digest | License |
| :--- | :--- | :--- | :--- |
| **yt-dlp** | `2025.02.19` | `785f73d2a71d7992984ea7c3ea4e1837895e7c8ecba0aa2286e1aafe9d424b91` | Unlicense |
| **FFmpeg** | `7.1` | `a937a00f2771d9d150ae1ae6e61f22eec74b41fb386eaebdfd5ffcfcfbc99d99` | GPL-3.0 / LGPL-2.1+ |
| **FFprobe** | `7.1` | `a937a00f2771d9d150ae1ae6e61f22eec74b41fb386eaebdfd5ffcfcfbc99d99` | GPL-3.0 / LGPL-2.1+ |
| **MediaInfo**| `24.12` | `f9570aa61fdb930e46124564c7ee23696f4244db919b33a595a882a9341496a7` | BSD-2-Clause |

Detailed specifications are documented in [docs/tool-management.md](docs/tool-management.md).

---

## 5. Security Model & Hardening

- **No Shell Interpolation**: Commands are passed as structured arrays to OS process runners. No `cmd.exe /c` or `sh -c` invocations.
- **Strict Input Validation**: Rejects non-HTTP/HTTPS URLs, NUL bytes, shell meta-characters, and path traversal sequences.
- **Content Security Policy**: Strict production CSP preventing unauthorized scripts, styles, and frames.
- **Capability Isolation**: Minimal Tauri capabilities granted to frontend.
- **Diagnostic Privacy**: Sanitizes Bearer tokens, cookies, auth headers, and sensitive URL parameters from diagnostics.

Detailed security documentation is in [docs/security.md](docs/security.md).

---

## 6. Testing & Quality Assurance

### Run Frontend Linter & Typechecks
```bash
npm run lint
```

### Run Rust Core Unit & Integration Tests
```bash
cargo test --manifest-path src-tauri/Cargo.toml
```

---

## 7. Documentation Index

- [Architecture Overview](docs/architecture.md)
- [Universal Media Resolver](docs/universal-resolver.md)
- [Tool Management & Supply Chain](docs/tool-management.md)
- [Performance Benchmarks & Constraints](docs/performance.md)
- [Security & Hardening](docs/security.md)
- [Packaging & Windows Distribution](docs/packaging.md)
- [Troubleshooting & Reliability](docs/troubleshooting.md)
- [User Cheatsheet](docs/cheatsheet.md)

---

## 8. License

This project is licensed under the MIT License. Embedded external tools are distributed under their respective licenses:
- `yt-dlp`: The Unlicense (Public Domain)
- `FFmpeg` / `FFprobe`: GNU General Public License v3.0 / LGPL v2.1+
- `MediaInfo`: BSD 2-Clause License
