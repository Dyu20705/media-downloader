# Universal Media Resolver Architecture

The **Universal Media Resolver** transforms One-Click Media Downloader from a site-specific tool into an adaptive, universal media intake engine. The user simply provides any URL, and the resolver analyzes, classifies, and selects the most efficient processing pipeline automatically without requiring deep format knowledge.

---

## 1. Core Principles

1. **Simplicity First**: The user flow is strictly:
   ```
   Paste URL → Analyze → Detect source & media → Recommend optimal strategy → Download → Verify → Done
   ```
2. **Cheapest Processing Priority**:
   ```
   Priority: 1. Stream copy > 2. Remux > 3. Merge > 4. Transcode
   ```
   Never re-encode video or audio when container repackaging or stream multiplexing satisfies the user's intent.
3. **No Unsolicited Speculative Scanning**: Resolution operations are lightweight, bounded by strict timeouts (15 seconds max), and execute zero heavy FFmpeg passes during metadata inspection.
4. **Security & Ethical Integrity**:
   - Strictly reject non-HTTP/HTTPS schemes (e.g. `file://`, `gopher://`).
   - Strictly reject private IP spoofing (e.g. `127.0.0.1`, `10.0.0.0/8`, `192.168.0.0/16`).
   - Do NOT attempt to bypass DRM, login paywalls, CAPTCHAs, or token authorization systems. Clearly categorize these as `DRM_PROTECTED` or `REQUIRES_AUTHENTICATION`.

---

## 2. Media Source Classification (`MediaSourceType`)

| Source Type | Identification | Strategy | Cost | Capabilities |
| :--- | :--- | :--- | :--- | :--- |
| **`DIRECT_FILE`** | File extension matches direct container (.mp4, .webm, .mkv, .mov, .mp3, .flac, .wav, .m4a) | `DIRECT_COPY` | `STREAM_COPY` | Untouched bitstream copy, no re-encoding. |
| **`HLS`** | `.m3u8` manifest URLs | `HLS_DOWNLOAD` | `STREAM_COPY` | Dynamic / live segment demuxing to MP4 container. |
| **`DASH`** | `.mpd` manifest URLs | `DASH_DOWNLOAD` | `REMUX` | Representation chunk multiplexing to MP4/MKV. |
| **`YT_DLP_EXTRACTOR`** | Recognized service (YouTube, Vimeo, SoundCloud, Bandcamp, Twitch, TikTok, etc.) | `YT_DLP_MERGE` / `YT_DLP_DOWNLOAD` | `MERGE` / `STREAM_COPY` | Rich metadata, HDR, multi-track audio, subtitles, chapters. |
| **`YT_DLP_GENERIC`** | Arbitrary webpage containing embedded HTML5 video/audio streams | `YT_DLP_DOWNLOAD` | `STREAM_COPY` | Stream extraction without transcoding. |
| **`UNSUPPORTED`** | No detectable or valid media streams found on page | `DIRECT_COPY` | `NO_PROCESSING` | Clean error categorization with helpful guidance. |
| **`INACCESSIBLE`** | DRM-protected or authentication-walled source | `DIRECT_COPY` | `NO_PROCESSING` | Protected stream guidance. |

---

## 3. Pipeline Strategies & Cost Model

### Strategies (`DownloadStrategy`)
* `DIRECT_COPY`: Direct HTTP stream download directly to destination.
* `YT_DLP_DOWNLOAD`: Standard progressive stream download via yt-dlp.
* `YT_DLP_MERGE`: Separate video and audio stream multiplexing into target container.
* `FFMPEG_REMUX`: Changing container format (e.g., TS to MP4) without modifying bitstream.
* `FFMPEG_TRANSCODE`: Re-encoding required (e.g. converting AAC/Opus to MP3 or FLAC).
* `HLS_DOWNLOAD`: Fetching HLS playlist segments into unified media container.
* `DASH_DOWNLOAD`: Fetching DASH audio/video adaptation sets.

### Transcoding Cost Model (`TranscodingCost`)
* `NO_PROCESSING`: Inactive / error state.
* `STREAM_COPY`: Zero quality loss, near-instant disk throughput.
* `REMUX`: Container rewrite only (zero video/audio re-encoding).
* `MERGE`: Native track multiplexing (e.g. 4K VP9 + Opus -> MKV).
* `TRANSCODE`: CPU/GPU heavy re-encoding (only when explicitly requested by format preset).

---

## 4. Structured Error Hierarchy (`ResolverErrorCategory`)

When media cannot be downloaded, the resolver provides a clear category and user-friendly explanation:
* `INVALID_URL`: URL syntax invalid or prohibited scheme.
* `UNSUPPORTED_PROTOCOL`: Non-HTTP/HTTPS protocols.
* `NO_MEDIA_FOUND`: Page did not contain any playable video or audio streams.
* `REQUIRES_AUTHENTICATION`: Account login, membership, or age-gate required.
* `DRM_PROTECTED`: Widevine, FairPlay, or encrypted media presentation.
* `NETWORK_UNREACHABLE`: Hostname cannot be resolved or connection dropped.
* `TIMEOUT`: Extraction timed out (exceeded 15s limit).
* `EXTRACTOR_FAILED`: Parser encountered unexpected page structure.
