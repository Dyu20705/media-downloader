# Managed tool support

The app installs tools into its per-user tool directory, and verifies each downloaded archive or executable against a configured SHA-256 digest before installation. A digest mismatch rejects the artifact. Custom executable paths and system `PATH` tools are also supported. The authoritative catalog is `src-tauri/crates/core/src/tool_manager.rs`.

## Current managed platform matrix

| Tool | Pin | Windows x86_64 | Linux x86_64 | macOS x86_64 |
|---|---:|---:|---:|---:|
| yt-dlp | 2026.08.19 | Yes | Yes | Yes |
| FFmpeg / ffprobe | 9.0.2 | Yes | Yes | Yes |
| MediaInfo CLI | 24.12 | Yes | No | Yes |

MediaInfo is optional. Other operating system architectures do not have managed artifacts in the current catalog; users can configure compatible executables. Release packages target only Windows x86_64, Linux x86_64, and Intel macOS.

## Supply-chain notes

The yt-dlp binaries are fetched from version-addressed assets in the upstream [2026.08.19 release](https://github.com/yt-dlp/yt-dlp/releases/tag/2026.08.19). That upstream page identifies the release as immutable. The catalog pins SHA-256 hashes calculated from the official Linux, Windows, and macOS assets; automated tests ensure checksum enforcement and executable discovery. The Linux x86_64 artifact is a standalone ELF executable, so it does not require a separate Python runtime. FFmpeg and ffprobe use version-addressed 9.0.2 assets, SHA-256 pins, and compatible archive formats on Linux, Windows, and Intel macOS. The Linux and Windows archives come from the date-addressed [BtbN FFmpeg build](https://github.com/BtbN/FFmpeg-Builds/releases/tag/autobuild-2026-09-28-13-06); macOS uses [Evermeet builds](https://evermeet.cx/ffmpeg/). These are GPL builds.

MediaArea's official [Windows downloads page](https://mediaarea.net/en/MediaInfo/Download/Windows) and [26.05 change log](https://mediaarea.net/MediaInfo/ChangeLog) confirm CLI 26.05 exists. The current code remains on 24.12 because this execution environment could not resolve `mediaarea.net`, so the exact Windows and macOS CLI archives could not both be downloaded and independently hashed here. No checksum is inferred from a third-party package listing. This is an open production release blocker: obtain the exact official artifacts, calculate SHA-256 locally, confirm archive members and version output, then pin and regression-test them.

FFmpeg build licensing varies with the selected build configuration. Read the artifact vendor's notices and the [FFmpeg legal page](https://ffmpeg.org/legal.html). See [third-party notices](../THIRD_PARTY_NOTICES.md) for the application's current notices.
