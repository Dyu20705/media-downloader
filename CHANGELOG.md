# Changelog

## 1.0.0

- Added reviewed single-media acquisition plans, durable history, FIFO queueing, explicit retry, and interrupted-job recovery.
- Added private URL persistence, IPC error sanitization, cross-platform core checks, and Linux package startup validation.
- Bounded subprocess diagnostics, added recoverable session-only persistence fallback, and aligned tool events with shared application diagnostics.
- Reduced Tauri plugin permissions, removed remote font dependencies, and added separate TypeScript, ESLint, and release verification gates.
- Updated managed yt-dlp to upstream 2026.08.19 artifacts with pinned SHA-256 hashes.
- Prevented release builds from discovering executables in the process working directory or repository tree; added regression coverage for yt-dlp and shared media-tool resolution.
- Validated and pinned destinations for in-process HTTP redirects, and documented the remaining yt-dlp network-boundary limitation.
- Required release packaging to pass quality and dependency-security gates on the exact tagged commit; hardened changelog extraction and Windows Authenticode verification.
