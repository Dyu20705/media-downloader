# One-Click Media Downloader — Design

## Goal

Build a Windows-first, local-only desktop application that makes media downloading as close as possible to:

1. paste or select a URL,
2. choose where to save,
3. choose a human-friendly output preset such as MP4, Best Video, MP3, Best Audio, or FLAC,
4. click Download once,
5. open the resulting file or its containing folder.

The application is an orchestration layer around `yt-dlp`, FFmpeg/FFprobe, and MediaInfo. It must not reimplement their media-processing logic.

## Product principles

- Default path is simple: paste URL → preset → Download.
- Remember the chosen output directory and last-used preset after first use.
- Hide codec IDs, yt-dlp format IDs, and command-line flags from normal users.
- Prefer source-preserving behavior over unnecessary transcoding.
- Do not imply that converting lossy audio to FLAC restores quality.
- Keep all processing local. No account, backend service, cloud sync, or database is required for MVP.
- Downloaded media, temporary files, and third-party executable binaries are never tracked by Git.

## Approved stack

### Desktop shell

- Tauri 2
- Rust backend
- React + TypeScript frontend

### Media tools

- `yt-dlp`: metadata extraction, format selection, download, playlist handling
- `ffmpeg`: mux/remux/transcode when requested
- `ffprobe`: low-level verification/fallback probing
- `mediainfo`: post-download human-readable technical inspection

## Existing local binary constraint

The repository intentionally excludes the existing local FFmpeg directory because its executables are too large for normal Git hosting. The current `.gitignore` behavior must remain intact.

Development must support binaries that already exist locally without requiring them to be committed.

### Tool resolution order for development

For each external tool, resolve in this order:

1. explicit application/tool configuration when provided,
2. project-local development tool locations,
3. recursive discovery beneath the existing `youtube-downloader/` development directory for known executable names,
4. system `PATH`,
5. packaged sidecar location in production builds.

Known executable names:

- `yt-dlp.exe`
- `ffmpeg.exe`
- `ffprobe.exe`
- `mediainfo.exe`

`ffplay.exe` is not required.

Tool resolution must return a typed result and a user-facing repair message when a required executable is unavailable.

## User experience

### Main screen

The primary screen contains:

- URL input with Paste support
- Analyze action (automatic after paste is acceptable after debounce)
- media preview: thumbnail, title, uploader/source, duration
- output folder selector
- preset selector
- quality selector
- Download button
- progress/status area
- completion actions: Open and Open Folder

### Presets

Normal UI exposes intent-based presets rather than codec combinations.

#### MP4 — Compatible

Use yt-dlp's MP4-oriented selection/remux behavior for broad playback compatibility. This is the default video preset.

#### Best Video

Prefer the best available video/audio and a container that preserves the selected source streams without unnecessary transcoding, typically MKV.

#### Best Audio

Download the best source audio stream without transcoding where possible. This is the default audio-quality choice.

#### MP3

Extract audio and convert to MP3 for compatibility.

#### FLAC — Lossless output

Extract audio and convert to FLAC only because the user requests the FLAC container/codec. The UI must state that conversion from a lossy source does not restore lost information.

### Quality

MVP exposes:

- Auto / Best (default)
- resolutions discovered from metadata, e.g. 2160p, 1440p, 1080p, 720p, 480p

Exact yt-dlp format IDs belong only in a future Advanced view.

### Save location

On first use, ask for or allow selection of an output directory. Persist it in app-local settings. Do not require a database.

Minimum settings model:

```json
{
  "downloadDirectory": "D:\\Media",
  "lastPreset": "mp4-compatible",
  "quality": "best",
  "openFolderAfterDownload": false
}
```

## Architecture

```text
React UI
   │
   │ Tauri commands + events
   ▼
Rust application core
   │
   ├── tool resolver
   ├── yt-dlp metadata analyzer
   ├── preset/quality command builder
   ├── download job manager
   ├── structured progress parser
   ├── final-path resolver
   ├── settings store
   └── media inspector
          │
          ├── ffprobe
          └── mediainfo
```

The React layer must not construct shell command strings. The Rust layer owns all process arguments and validates inputs.

## Process safety

- Spawn executables directly with argument arrays.
- Do not execute user URLs through `cmd /c`, PowerShell string interpolation, or shell-concatenated command strings.
- Treat URL, output path, title, and metadata as data, never executable shell syntax.
- Capture stdout/stderr separately.
- Preserve raw technical output for diagnostics while mapping known failures to concise UI errors.

## Metadata analysis

Analyze before download using yt-dlp machine-readable output, not normal console prose.

The typed frontend-facing model must include at least:

```text
MediaMetadata
- title
- uploader
- durationSeconds
- thumbnailUrl
- webpageUrl
- mediaKind
- playlistCount?
- availableResolutions[]
```

No media payload should be downloaded during analysis.

## Download request model

```text
DownloadRequest
- url
- outputDirectory
- preset
- quality
```

`preset` is a closed enum owned by the Rust core.

## Job model and state machine

Each job has a stable ID and follows:

```text
IDLE
  → ANALYZING
  → READY
  → DOWNLOADING
  → POST_PROCESSING
  → VERIFYING
  → COMPLETED
```

Any active state may transition to `FAILED`. Downloading/post-processing may transition through `CANCELLING` to `CANCELLED`.

Minimum job data:

```text
DownloadJob
- id
- sourceUrl
- metadata
- preset
- quality
- outputDirectory
- progress
- speed
- eta
- status
- finalPath?
- error?
```

MVP concurrency is 1 job at a time. Queue/playlist work is post-MVP.

## Structured progress

Use yt-dlp structured progress output such as `--progress-template` rather than scraping its ordinary human-oriented progress lines.

Rust converts process output to typed events and emits Tauri events such as:

```text
download-progress
postprocess-progress
download-state
download-completed
download-failed
```

React consumes only typed events.

## Final output path

Never infer the final filename from the title or pre-processing extension. yt-dlp/FFmpeg may change the extension during post-processing.

Capture the post-move final path from yt-dlp and return it as `DownloadResult.finalPath`.

`Open` and `Open Folder` must use this exact path.

## MediaInfo and verification

MediaInfo runs after a successful download and is not part of the critical download path.

On success, expose a compact summary such as:

- container
- video codec
- width/height
- frame rate when available
- HDR when available
- audio codec
- channels
- sample rate
- duration

If MediaInfo is unavailable or fails, the download remains successful and the UI reports inspection as unavailable. FFprobe may provide a lower-level fallback.

## Error handling

Map known failures to stable application error kinds, including:

- invalid/unsupported URL
- metadata unavailable
- authentication/cookie required
- network failure
- output-directory permission failure
- yt-dlp unavailable
- FFmpeg unavailable
- MediaInfo unavailable
- cancelled
- unknown external-tool failure

Normal UI shows a concise message plus a `Show technical details` affordance containing sanitized process diagnostics.

## Repository layout

Target layout:

```text
media-downloader/
├── src/
│   ├── components/
│   ├── stores/
│   └── App.tsx
├── src-tauri/
│   ├── src/
│   │   ├── commands/
│   │   ├── downloader/
│   │   ├── media/
│   │   └── platform/
│   └── binaries/          # generated/ignored
├── scripts/
├── tests/fixtures/
├── docs/
├── third_party.lock.json
├── package.json
├── README.md
└── .gitignore
```

Keep modules responsibility-focused rather than creating one large process-management file.

## Third-party binary policy

### Source control

Never commit:

- FFmpeg executables or archives
- FFprobe executable
- yt-dlp executable
- MediaInfo executable
- downloaded media
- generated temporary media

### Development

Reuse locally installed/existing binaries through the tool resolver. The user's existing ignored FFmpeg folder must continue to work.

### Release

Future packaged releases use a deterministic bootstrap/bundle process:

1. version-lock third-party tools,
2. download them during bootstrap/release build,
3. verify SHA-256,
4. extract only required executables,
5. bundle them as Tauri sidecars,
6. keep generated binary directories ignored by Git.

## MVP scope

MVP is complete when all of the following work on Windows:

1. Tauri application starts.
2. Rust ↔ React command invocation works.
3. Existing local yt-dlp/FFmpeg/FFprobe/MediaInfo can be discovered without Git-tracking them.
4. A supported URL can be analyzed into typed metadata.
5. User can select and persist an output directory.
6. User can select MP4, Best Video, Best Audio, MP3, or FLAC.
7. User can select Auto/Best or a discovered resolution.
8. One click starts a download.
9. UI receives structured progress and state updates.
10. Cancellation stops the owned child process cleanly.
11. Final path is returned after post-processing.
12. Open and Open Folder use the exact final path.
13. MediaInfo summary appears when available without making inspection failure fail the download.
14. Core deterministic tests run without network access or real media services.

## Explicitly post-MVP

- playlist selection UI
- multiple concurrent jobs
- cookies/browser authentication UI
- livestream mode
- subtitle controls
- custom yt-dlp argument textbox
- built-in media player/editor
- cloud sync
- account system
- database-backed download history
- automatic independent updates of FFmpeg/MediaInfo

## Testing strategy

Core logic must be testable without network access and without launching real external tools unless running a manual smoke test.

Use process-runner abstractions and fixtures for:

- metadata JSON
- progress output
- post-processing output
- unsupported URL errors
- network errors
- final path output
- MediaInfo JSON

Required deterministic tests include:

- preset → expected argument vector
- requested resolution → expected format policy
- metadata JSON → typed model
- progress line → typed progress
- tool stderr → stable error kind
- final-path output → exact path
- output path containing spaces
- Unicode titles
- tool discovery with existing project-local FFmpeg directory
- missing-tool repair message
- cancel → owned child termination path

Real-network smoke tests are manual/release tests only.

## Delivery sections and remote-update constraint

Implementation is executed in sections. A section may contain multiple local commits, but must perform at most one `git push`/remote branch-ref update at the end of that section.

Planned sections:

1. Foundation: Tauri/React/Rust scaffold, config, tool resolver, settings skeleton.
2. Download core: metadata, presets, quality policy, structured process runner, progress, final path, cancellation.
3. One-click UI: paste/analyze, preview, folder/preset/quality selection, progress and errors.
4. Post-download: MediaInfo/FFprobe inspection, Open/Open Folder, completion UX.
5. MVP hardening: deterministic tests, docs, dependency bootstrap/bundle policy, Windows build verification.

No section may add the existing large local FFmpeg folder to Git history.

## Non-goals

This project does not attempt to bypass DRM, access-control systems, paid-media protections, or platform authorization controls. It downloads only content that the underlying tools can lawfully and technically access under the user's permissions and applicable platform terms.
