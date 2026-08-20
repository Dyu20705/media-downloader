# Application Architecture — One-Click Media Downloader

## 1. System Overview

One-Click Media Downloader is built with a dual-runtime desktop architecture combining **Tauri 2 (Rust core)** and a **Vite + React 18 frontend** with typed Inter-Process Communication (IPC).

```
┌─────────────────────────────────────────────────────────────┐
│                    React 18 + TypeScript UI                 │
│  (Single-View Workspace, Format Presets, 4Hz Live Telemetry)│
└──────────────────────────────┬──────────────────────────────┘
                               │ Typed Tauri IPC / SSE Events
┌──────────────────────────────▼──────────────────────────────┐
│                   Tauri 2 / Rust Core Core                  │
│ ┌───────────────────────┐         ┌───────────────────────┐ │
│ │  DownloadStateMachine │         │    ToolResolver &     │ │
│ │ (Downloading, Verify) │         │     ToolManager       │ │
│ └───────────┬───────────┘         └───────────┬───────────┘ │
│             │                                 │             │
│ ┌───────────▼───────────┐         ┌───────────▼───────────┐ │
│ │ ProcessTree Runner    │         │ DiagnosticsBuffer     │ │
│ │ (Job Object / SIGKILL)│         │ (256-line, <=64KiB)   │ │
│ └───────────┬───────────┘         └───────────────────────┘ │
└─────────────┼───────────────────────────────────────────────┘
              │ Typed process invocation
┌─────────────▼───────────────────────────────────────────────┐
│                 Managed Media Binaries                      │
│      yt-dlp (Extractor) │ FFmpeg / FFprobe │ MediaInfo      │
└─────────────────────────────────────────────────────────────┘
```

---

## 2. Core Modules & Responsibilities

### 2.1 Tauri Backend (`src-tauri/crates/core`)
- **`url_validator.rs`**: Strict HTTP/HTTPS URL validation, null-byte rejection, length limits (2048 chars), malformed URL rejection.
- **`path_validator.rs`**: Path normalization, directory creation, reserved Windows character sanitization (`\ / : * ? " < > |`), path length truncation to prevent `MAX_PATH` overflow.
- **`tool_manager.rs`**: Pinned binary specification catalog (`yt-dlp`, `FFmpeg`, `FFprobe`, `MediaInfo`) with SHA-256 verification, staging directory extraction, atomic installation, and atomic `manifest.json` persistence.
- **`process_runner.rs`**: Async process spawning with stdout/stderr line streaming. On Windows, uses process tree termination (`taskkill /F /T /PID`) and process group isolation on Unix to ensure no orphaned child processes survive app exit or job cancellation.
- **`state_machine.rs`**: Strict validated transitions:
  `IDLE → ANALYZING → READY → DOWNLOADING → POST_PROCESSING → VERIFYING → COMPLETED | FAILED | CANCELLED`.
- **`download_manager.rs`**: Single-download concurrency lock, 4 Hz UI telemetry throttling, process tree lifecycle management, post-download verification.
- **`media_verifier.rs`**: Verification using `ffprobe` / `MediaInfo` checking container headers, duration, stream count, and codec specs.
- **`settings.rs`**: Atomic settings persistence with `.tmp` staging, `.bak` backup copy, corruption recovery, and schema verification.
- **`diagnostics.rs`**: Bounded 256-entry ring buffer with <= 64 KiB memory ceiling, sanitizing auth tokens, cookies, passwords, and sensitive URLs.

### 2.2 Frontend (`src/`)
- **`App.tsx`**: Single-view master workspace container, zero-polling SSE / Tauri event subscriptions, lazy-loading secondary dialogs (Settings, Diagnostics, Tools, History, MediaInfo).
- **`components/UrlInputBar.tsx`**: URL input with clear button, analyze action, and keyboard shortcuts.
- **`components/MediaSummaryCard.tsx`**: High-contrast summary display showing title, duration, author, and source link.
- **`components/FormatSelector.tsx`**: One-click format preset cards (`mp4-compatible`, `best-video`, `best-audio`, `mp3-universal`, `flac-lossless`).
- **`components/QualityAndDirectory.tsx`**: Target resolution selector and destination folder picker.
- **`components/DownloadProgressState.tsx`**: Responsive progress bar, download speed, ETA, and cancellation triggers.

---

## 3. Strict Concurrency & Memory Guarantees

1. **Max Concurrency = 1**: The application strictly enforces a single concurrent active download job to prevent disk thrashing and bandwidth contention.
2. **Zero Polling**: Replaces timer polling with push-based Tauri IPC events or SSE streams.
3. **Bounded Telemetry**: Progress events are throttled to a maximum frequency of 4 Hz (250 ms), maintaining 60 FPS UI responsiveness.
4. **No Media Bytes in Memory**: Stream media writes directly to disk; never buffers raw video/audio chunks in React or Rust memory buffers.
