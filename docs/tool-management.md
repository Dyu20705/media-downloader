# Managed tool support

The app installs tools into its per-user tool directory, and verifies each downloaded archive or executable against a configured SHA-256 digest before installation. A digest mismatch rejects the artifact. Custom executable paths and system `PATH` tools are also supported. The authoritative catalog is `src-tauri/crates/core/src/tool_manager.rs`.

## Current managed platform matrix

| Tool | Pin | Windows x86_64 | Linux x86_64 | macOS x86_64 |
|---|---:|---:|---:|---:|
| yt-dlp | 2026.08.19 | Yes | Yes | Yes |
| FFmpeg / ffprobe | 9.0.2 | Yes | Yes | Yes |
| MediaInfo CLI | 26.05 | Yes | No | No |

MediaInfo is optional. Other operating system architectures do not have managed artifacts in the current catalog; users can configure compatible executables. Release packages target only Windows x86_64, Linux x86_64, and Intel macOS.

## Supply-chain notes

The yt-dlp binaries are fetched from version-addressed assets in the upstream [2026.08.19 release](https://github.com/yt-dlp/yt-dlp/releases/tag/2026.08.19). That upstream page identifies the release as immutable. The catalog pins SHA-256 hashes calculated from the official Linux, Windows, and macOS assets; automated tests ensure checksum enforcement and executable discovery. The Linux x86_64 artifact is a standalone ELF executable, so it does not require a separate Python runtime. FFmpeg and ffprobe use version-addressed 9.0.2 assets, SHA-256 pins, and compatible archive formats on Linux, Windows, and Intel macOS. The Linux and Windows archives come from the date-addressed [BtbN FFmpeg build](https://github.com/BtbN/FFmpeg-Builds/releases/tag/autobuild-2026-09-28-13-06); macOS uses [Evermeet builds](https://evermeet.cx/ffmpeg/). These are GPL builds.

MediaInfo CLI 26.05 uses the official version-specific [Windows x64 ZIP](https://mediaarea.net/download/binary/mediainfo/26.05/MediaInfo_CLI_26.05_Windows_x64.zip). The downloaded archive has locally calculated SHA-256 `f7f80620ce6d14f4995f0de6f98e3ef18ad29496db01899571152ee3311229f9`; its executable is `MediaInfo.exe` at the archive root. Its PE imports are Windows system DLLs (`KERNEL32.dll`, `SHELL32.dll`), so the adjacent optional LIBCURL DLL is not needed for local inspection. Checksum mismatch, corrupt archives, traversal, missing executables, and ambiguous executable selection are regression-tested. Windows version execution requires a Windows host and has not been exercised on Linux.

The official [macOS CLI 26.05 download](https://mediaarea.net/download/binary/mediainfo/26.05/MediaInfo_CLI_26.05_Mac.dmg) is a DMG, not the previously configured tar.bz2 (the 26.05 tar.bz2 URL returns 404). The DMG was downloaded and locally hashed as `507605a7c8f1054a6996d99a4ef5b5a0711cfbf2f8ca2ef5161d6ee701ea8015`, but its install layout and executable invocation have not been verified. The managed installer does not support DMG/package installation. Managed MediaInfo on macOS is therefore removed from v1.0.0; users may install the official CLI themselves and configure its executable path. Linux likewise uses a system or custom MediaInfo. No stale 24.12 managed artifact remains.

FFmpeg build licensing varies with the selected build configuration. Read the artifact vendor's notices and the [FFmpeg legal page](https://ffmpeg.org/legal.html). See [third-party notices](../THIRD_PARTY_NOTICES.md) for the application's current notices.
