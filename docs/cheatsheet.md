# One-Click Media Downloader — User Cheatsheet & Guide

This cheatsheet provides a clear, technical reference for acquisition operations, output profiles, quality options, tool management, advanced configuration, and troubleshooting.

---

## 1. Operation and Output Profile Guide

First choose what to acquire, then independently choose how the output should behave.

| Dimension | Choice | Meaning |
| :--- | :--- | :--- |
| **Operation** | Entire Media | Acquire the complete selected video and audio streams. |
| **Operation** | Audio Only | Acquire audio without the video stream. |
| **Output Profile** | Best Source | Preserve the strongest suitable source streams and avoid unnecessary transcoding. |
| **Output Profile** | Universal | Prefer broad playback compatibility; Audio Only produces MP3. |
| **Output Profile** | Editing | Produce an editing-friendly output; Audio Only produces FLAC. |
| **Output Profile** | Small | Select the smallest suitable source streams. |

Clip and Chapter perform fast time-range cuts; boundaries can align to nearby keyframes. Thumbnail exports JPEG. Subtitles exports one selected WebVTT language per job with optional automatic captions. Image and subtitle exports use separate per-job folders and ignore media output profiles.

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

- **yt-dlp (`v2026.08.19`)**: Media stream extraction engine.
- **FFmpeg & FFprobe (`v9.0.2`)**: Audio/video muxing, stream merging, post-processing, and format conversion.
- **MediaInfo (`v24.12`)**: Container verification and stream inspection. The required production refresh to 26.05 is still pending verified artifact hashes.

### Tool Resolution Order
1. Custom path override in **Settings** (if configured).
2. Application-local managed directory (verified against the pinned archive checksum).
3. System environment `PATH` (if already installed).

Debug builds may also search project-relative `./`, `./bin/`, and `./tools/` locations. Release builds never search the process working directory or repository tree.

---

## 4. Advanced Settings Reference

- **Embed Metadata (`--embed-metadata`)**: Writes title, artist/uploader, description, and tags directly into the container tags (ID3 / MP4 tags / Vorbis comments).
- **Embed Thumbnail (`--embed-thumbnail`)**: Embeds full-resolution artwork into the file so file managers (Windows Explorer, Finder) display cover art.
- **Embed Chapters (`--embed-chapters`)**: Injects timestamp markers for videos with multiple segments.
- **Trim Filenames**: Automatically limits output filename length (default: 180 characters) to prevent Windows MAX_PATH (260 char) filesystem errors.
- **Resource handling**: One download is active at a time; progress updates and diagnostic retention are bounded. See [performance characteristics](performance.md) for the exact limits and their scope.

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
- `Tab` / `Shift + Tab`: Navigate cleanly across controls in order: `URL → Analyze → Operation → Profile → Quality → Folder → Download`.
- `Space` / `Enter`: Activate buttons or select operation/profile choices.
- `Escape`: Close any open dialog, modal, or drawer.
