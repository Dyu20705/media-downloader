# One-Click Media Downloader — User Cheatsheet & Guide

This cheatsheet provides a clear, technical reference for presets, formats, quality options, tool management, advanced configuration, and troubleshooting.

---

## 1. Preset Intent Guide

Choose the preset that matches your goal:

| Preset | Container / Format | Audio / Video Codec | Recommended Use Case |
| :--- | :--- | :--- | :--- |
| **MP4 (Compatible)** | `.mp4` | H.264 (AVC) + AAC | **Default choice for videos.** Plays smoothly on virtually all TVs, smartphones, tablets, browsers, and editing software (Premiere, Final Cut, DaVinci). |
| **Best Video** | `.mkv` | VP9 / AV1 / H.265 (Source) + Opus / AAC | **Maximum visual quality.** Retains 4K, 8K, HDR, high frame rates (60fps), and uncompressed metadata in a flexible Matroska container. |
| **Best Audio** | Source-dependent | Source audio where practical | **Source-preserving audio extraction.** Avoids an unnecessary lossy-to-lossy conversion; the codec and extension depend on the selected source. |
| **MP3 (Universal)** | `.mp3` | MP3 (lossy VBR) | **Universal compatibility for audio.** Uses the encoder's highest-quality VBR setting but cannot improve the source stream. |
| **FLAC Output** | `.flac` | FLAC | **Format-specific audio output.** FLAC encoding is lossless at that stage, but transcoding a lossy source cannot restore discarded information. |

---

## 2. Quality Options

| Quality Selector | Target Resolution | Bitrate Range | Typical File Size (10 min) |
| :--- | :--- | :--- | :--- |
| **Best Available (Auto)** | Up to 4K (2160p) / 8K (4320p) | 15–45 Mbps | 1.2–3.5 GB |
| **4K Ultra HD (2160p)** | 3840 × 2160 | 12–25 Mbps | 800 MB – 1.8 GB |
| **1440p QHD** | 2560 × 1440 | 6–12 Mbps | 400–850 MB |
| **1080p Full HD** | 1920 × 1080 | 3–6 Mbps | 200–450 MB |
| **720p HD** | 1280 × 720 | 1.5–3 Mbps | 100–220 MB |
| **480p SD** | 854 × 480 | 0.8–1.5 Mbps | 50–100 MB |

---

## 3. Tool Management & Supply Chain

One-Click Media Downloader manages required external helper tools in an isolated, application-local directory without modifying global Windows `PATH` or registry keys:

- **yt-dlp (`v2025.02.19`)**: Media stream extraction engine.
- **FFmpeg & FFprobe (`v7.1`)**: Audio/video muxing, stream merging, post-processing, and format conversion.
- **MediaInfo (`v24.12`)**: Container verification and stream inspection.

### Tool Resolution Order
1. Custom path override in **Settings** (if configured).
2. Project-local folder (`./bin/` or `./tools/`).
3. System environment `PATH` (if already installed).
4. Application-local managed directory (`%LOCALAPPDATA%\OneClickMediaDownloader\tools\`).

---

## 4. Advanced Settings Reference

- **Embed Metadata (`--embed-metadata`)**: Writes title, artist/uploader, description, and tags directly into the container tags (ID3 / MP4 tags / Vorbis comments).
- **Embed Thumbnail (`--embed-thumbnail`)**: Embeds full-resolution artwork into the file so file managers (Windows Explorer, Finder) display cover art.
- **Embed Chapters (`--embed-chapters`)**: Injects timestamp markers for videos with multiple segments.
- **Trim Filenames**: Automatically limits output filename length (default: 180 characters) to prevent Windows MAX_PATH (260 char) filesystem errors.
- **Extreme Performance Baseline**:
  - Enforces 1 active download job at a time to prevent disk I/O bottlenecks and ISP bandwidth contention.
  - Streaming updates throttled to 4 Hz to guarantee zero UI stutter.
  - O(1) memory consumption with zero unbounded media buffer retention.

---

## 5. Common Errors & Troubleshooting

| Issue / Error | Cause | Resolution |
| :--- | :--- | :--- |
| **"Invalid or unsupported URL"** | The URL is malformed or from an unsupported service. | Ensure the link starts with `https://` and points to a supported video/audio page. |
| **"FFmpeg missing or damaged"** | FFmpeg binary was not found or failed hash check. | Open **Engine Tools** from the header menu and click **Install / Repair**. |
| **"Write permission denied"** | Target output directory is read-only or in a protected system folder. | Open **Settings** and set the download folder to `D:\Videos`, `D:\Music`, or your user `Downloads` folder. |
| **"Connection aborted / Geo-restricted"** | The media is blocked in your region or requires age verification. | Check the webpage in your browser to confirm availability. |
| **"Download cancelled"** | User pressed the cancel button. | Click **Try Again** to restart the download. |

---

## 6. Keyboard Shortcuts

- `Ctrl + V`: Paste URL into the input field.
- `Enter` (in URL field): Trigger media analysis.
- `Tab` / `Shift + Tab`: Navigate cleanly across controls in order: `URL → Analyze → Format → Quality → Folder → Download`.
- `Space` / `Enter`: Activate buttons or select format chips.
- `Escape`: Close any open dialog, modal, or drawer.
