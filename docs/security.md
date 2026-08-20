# Security Architecture & Hardening — One-Click Media Downloader

## 1. Threat Model & Security Principles

One-Click Media Downloader adheres to a strict principle of least privilege, defense-in-depth, and zero-trust input handling.

---

## 2. Core Security Controls

### 2.1 No Shell String Interpolation
- **Command Spawning**: All process invocations (`yt-dlp`, `FFmpeg`, `ffprobe`, `MediaInfo`) use vector argument arrays (`std::process::Command` / `tokio::process::Command`) rather than shell string execution (`sh -c` or `cmd /c`).
- **No Wildcard Execution**: Shell operators (`|`, `&`, `;`, `$`, `>`, `<`) in URLs or filenames cannot trigger shell execution.

### 2.2 Strict Input & URL Validation
- **URL Sanitation**: Validated via `validate_media_url()`:
  - Allowed schemes: Strictly `http://` and `https://`. Unsupported schemes (`file://`, `ftp://`, `gopher://`, `javascript:`) are rejected.
  - Reject NUL bytes (`\0`), ASCII control characters (`0x00–0x1F`), and newlines.
  - Length ceiling: Max 2048 characters.
  - Domain structure verification.
- **Path Sanitation**: Validated via `validate_and_ensure_directory()` and `sanitize_file_name()`:
  - Strips OS-reserved characters (`\ / : * ? " < > |`).
  - Rejects NUL bytes and control codes.
  - Enforces length caps (180 chars) to prevent Windows `MAX_PATH` overflow.

### 2.3 Strict Production Content Security Policy (CSP)
Defined in `src-tauri/tauri.conf.json`:
```text
default-src 'self';
script-src 'self';
style-src 'self' 'unsafe-inline' https://fonts.googleapis.com;
font-src 'self' data: https://fonts.gstatic.com;
img-src 'self' data: https: blob:;
connect-src 'self' ipc: http://localhost:3000 https://github.com https://*.githubusercontent.com;
frame-src 'none';
object-src 'none';
```

### 2.4 Minimal Tauri Capabilities
Frontend capabilities are scoped strictly to required operations:
- `core:default`: Standard Tauri IPC invocation.
- `opener:default`: Explicit URL opening in default system browser.
- `dialog:allow-open`: Folder selection dialog.
- Arbitrary filesystem read/write and wildcard shell plugins are **disabled**.

### 2.5 Safe Diagnostics & Log Redaction
The in-memory ring buffer sanitizes all diagnostic log entries:
- Redacts authorization tokens (`Bearer [REDACTED]`, `Authorization: [REDACTED]`).
- Redacts session cookies (`Cookie: [REDACTED]`).
- Redacts API keys (`api_key=[REDACTED]`).
- Redacts signature tokens (`sig=[REDACTED]`, `signature=[REDACTED]`, `token=[REDACTED]`).
- Strict memory boundary: 256-line ring buffer capped at <= 64 KiB retained text.

### 2.6 Safe OS Folder / File Opening
Commands `open_folder` and `open_file` validate that the path:
1. Is non-empty, contains no NUL bytes, and no control codes.
2. Exists on the local filesystem.
3. Invokes OS file managers directly (`explorer /select,<path>` on Windows, `open` on macOS, `xdg-open` on Linux) without passing through `cmd.exe` or shell interpreters.
