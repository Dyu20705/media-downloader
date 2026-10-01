# Application Architecture — One-Click Media Downloader

## 1. System Overview

One-Click Media Downloader is built with a dual-runtime desktop architecture combining **Tauri 2 (Rust core)** and a **Vite + React 18 frontend** with typed Inter-Process Communication (IPC).

```
┌─────────────────────────────────────────────────────────────┐
│                    React 18 + TypeScript UI                 │
│ (Operation × Output Profile, Plan UI, 4Hz Live Telemetry)   │
└──────────────────────────────┬──────────────────────────────┘
                               │ Typed Tauri IPC
┌──────────────────────────────▼──────────────────────────────┐
│                 Tauri 2 / Rust Host + Core                 │
│ ┌───────────────────────┐         ┌───────────────────────┐ │
│ │  JobStore (SQLite)    │         │    ToolResolver &     │ │
│ │ (Downloading, Verify) │         │     ToolManager       │ │
│ └───────────┬───────────┘         └───────────┬───────────┘ │
│             │                                 │             │
│ ┌───────────▼───────────┐         ┌───────────▼───────────┐ │
│ │ ProcessTree Runner    │         │ DiagnosticsBuffer     │ │
│ │ (taskkill / SIGKILL) │         │ (256-line, <=64KiB)   │ │
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
- **`download_manager.rs`**: Single-download admission lock, process lifecycle, explicit retry/replan, and post-download verification.
- **`persistence.rs`**: Versioned SQLite schema, transactional job/attempt snapshots, privacy-filtered URL persistence, startup interrupted-state recovery.
- **`planner.rs`**: Authoritative `SourceMediaGraph + AcquisitionRequest → AcquisitionPlan` resolution, including exact stream selection, planned output, processing class, estimates, warnings, and tool requirements.
- **`media_verifier.rs`**: Verification using `ffprobe` / `MediaInfo` checking container headers, duration, stream count, and codec specs.
- **`settings.rs`**: Atomic settings persistence with `.tmp` staging, `.bak` backup copy, corruption recovery, and schema verification.
- **`diagnostics.rs`**: Bounded 256-entry ring buffer with <= 64 KiB memory ceiling, sanitizing auth tokens, cookies, passwords, and sensitive URLs.

### 2.2 Frontend (`src/`)
- **`App.tsx`**: Single-view master workspace container and secondary-dialog coordinator (Settings, Diagnostics, Tools, History, MediaInfo).
- **`components/UrlInputBar.tsx`**: URL input with clear button, analyze action, and keyboard shortcuts.
- **`components/MediaSummaryCard.tsx`**: High-contrast summary display showing title, duration, author, and source link.
- **`components/AcquisitionControls.tsx`**: Orthogonal operation and output-profile controls, with unavailable operations explicitly disabled until an executor exists.
- **`components/DownloadPlanCard.tsx`**: Backend-authoritative source selection, planned output, processing, estimates, requirements, and warnings.
- **`components/QualityAndDirectory.tsx`**: Target resolution selector and destination folder picker.
- **`components/DownloadProgressState.tsx`**: Responsive progress bar, download speed, ETA, and cancellation triggers.

---

## 3. Concurrency and retained-data bounds

1. One active download job is admitted at a time; additional requests use a durable FIFO queue.
2. React polls the active job every 400 ms while work is active; progress state updates are coalesced to at most 4 Hz.
3. The diagnostics buffer retains at most 256 entries and 64 KiB of accounted text. Process output uses a bounded queue and drops oversized or excess diagnostic lines.
4. Media transfer and encoding are delegated to child tools. This is not a bound on those tools' internal memory use.

## Durable state

The Tauri host opens a SQLite `JobStore` in the per-user local application-data directory. If the database cannot open or interrupted-job recovery fails, diagnostics report the error and the app falls back to session-only in-memory history; it does not remove the existing database. On successful startup, nonterminal persisted jobs become `INTERRUPTED`; no download is resumed automatically. Retries are explicit, use a new job ID, and recompute the plan under current settings. URL credentials, fragments, and unapproved query values are removed before serialization.
