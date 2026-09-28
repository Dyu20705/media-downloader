# One-Click Media Downloader

> Production-grade desktop media downloader powered by Tauri 2, Rust, React 18, yt-dlp, FFmpeg, and MediaInfo.

---

## 🎬 Demo

https://github.com/user-attachments/assets/f60b6308-8bd5-452e-88e6-eb7018b5772e

---

## Features

- **Simple & Intuitive Workflow**: Paste URL → Choose an operation and output profile → Review the authoritative plan → Download.
- **Automated Tool Management**: Self-manages and cryptographically verifies `yt-dlp`, `FFmpeg`, `FFprobe`, and `MediaInfo` binaries without modifying system PATH.
- **Orthogonal Acquisition Controls**:
  - **Operation** chooses what to acquire: Entire Media, Audio Only, Clip, Chapter, Thumbnail (JPEG), or Subtitles (one WebVTT language per job, optionally automatic captions).
  - **Output Profile** independently chooses how output should behave: Best Source, Universal, Editing, or Small.
  - The backend-generated plan shows exact selected streams, output shape, processing, requirements, and warnings before download.
- **Deep Media Verification**: Real-time post-download inspection ensuring valid video/audio streams, duration, bitrate, and headers.
- **Secure by Design**: Isolated subprocess execution without shell interpolation, path traversal prevention, and strict sanitization.

---

## Tech Stack

- **Frontend**: React 18, TypeScript, Tailwind CSS, Vite, Lucide Icons.
- **Desktop Host & Core Engine**: Tauri 2, Rust.

---

## Getting Started

### Prerequisites
- Node.js 18+ & npm
- Rust 1.75+

### Development
```bash
# Install dependencies
npm install

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
