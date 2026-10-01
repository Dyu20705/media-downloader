# One-Click Media Downloader

> Desktop media downloader built with Tauri 2, Rust, React 18, yt-dlp, and FFmpeg. See [current validation and release limitations](CURRENT-STATE.md) before distribution.

---

## 🎬 Demo

https://github.com/user-attachments/assets/f60b6308-8bd5-452e-88e6-eb7018b5772e

---

## Features

- **Simple & Intuitive Workflow**: Paste URL → Choose an operation and output profile → Review the authoritative plan → Download.
- **Tool Management**: Managed installs use checksum validation and OS/architecture-specific artifacts; custom executable paths are supported. See [current platform limitations](CURRENT-STATE.md#tool-management).
- **Orthogonal Acquisition Controls**:
  - **Operation** chooses what to acquire: Entire Media, Audio Only, Clip, Chapter, Thumbnail (JPEG), or Subtitles (one WebVTT language per job, optionally automatic captions).
  - **Output Profile** independently chooses how output should behave: Best Source, Universal, Editing, or Small.
  - The backend-generated plan shows exact selected streams, output shape, processing, requirements, and warnings before download.
- **Plan Verification**: The backend compares inspected codecs, container, dimensions, FPS and duration against the reviewed plan; mismatches are shown in the receipt.
- **Runtime checks**: Subprocess execution without shell interpolation, URL/DNS preflight and bounded diagnostics. URL credentials and path/query/fragment values are removed from diagnostics. Playlist links default to one video; whole-playlist execution is not available yet. See [security boundaries](CURRENT-STATE.md#security-and-diagnostics).

---

## Tech Stack

- **Frontend**: React 18, TypeScript, Tailwind CSS, Vite, Lucide Icons.
- **Desktop Host & Core Engine**: Tauri 2, Rust.

---

## Getting Started

### Prerequisites

- Node.js 24 & npm
- Rust toolchain 1.96.0 (CI-pinned)

### Development

```bash
# Install dependencies
npm ci

# Run in development mode
npm run tauri dev
```

### Production Build

```bash
# Build desktop binary and installer
npm run tauri build
```

---

## Documentation

- [Current implementation, recovery semantics, validation and release status](CURRENT-STATE.md)

- [Current state, validation and limitations](CURRENT-STATE.md)

- [Architecture Overview](docs/architecture.md)
- [Universal Media Resolver](docs/universal-resolver.md)
- [Quality Transparency & Download Plan](docs/quality-transparency.md)
- [Tool Management & Supply Chain](docs/tool-management.md)
- [Security & Hardening](docs/security.md)
- [Performance & Benchmarks](docs/performance.md)
- [Packaging & Windows Distribution](docs/packaging.md)
- [Troubleshooting & Reliability](docs/troubleshooting.md)
- [User Cheatsheet](docs/cheatsheet.md)

---

## License

This project is licensed under the [MIT License](LICENSE). Embedded external tools are distributed under their respective licenses:
- `yt-dlp`: The Unlicense (Public Domain)
- `FFmpeg` / `FFprobe`: GNU General Public License v3.0 / LGPL v2.1+
- `MediaInfo`: BSD 2-Clause License
