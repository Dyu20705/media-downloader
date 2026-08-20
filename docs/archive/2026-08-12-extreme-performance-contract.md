# Extreme Performance Contract — One-Click Media Downloader

> Status: design amendment to `2026-08-12-one-click-media-downloader-design.md`.
>
> This contract is mandatory for implementation unless a measured benchmark demonstrates that a different choice is better. Optimization must be evidence-driven; no exotic algorithm, cache, thread pool, allocator, or dependency is added only because it is fashionable.

## 1. Objective

The application is a small orchestration layer around `yt-dlp`, FFmpeg/FFprobe, and MediaInfo. It must remain negligible compared with the media tools it launches.

Optimization priorities, in order:

1. correctness and data safety,
2. user-perceived latency,
3. idle CPU / wakeups / energy,
4. bounded memory,
5. avoidable disk I/O and storage amplification,
6. reliability under failure/cancellation,
7. security and least privilege,
8. distribution/startup footprint,
9. peak throughput where it does not materially regress the previous items.

When two goals conflict, prefer the Pareto-efficient design. Do not maximize throughput by default if the gain is small and CPU, memory, I/O, or energy cost rises materially.

## 2. Reference performance methodology

Performance claims must be measured, not inferred from code appearance.

Reference baseline for Windows verification:

- Windows 11 x64
- WebView2 runtime already installed
- SSD/NVMe storage
- at least 4 CPU cores / 8 logical processors
- at least 8 GiB RAM
- AC power for reproducible throughput tests; repeat critical idle/energy proxy tests on battery when available

Record p50 and p95 where repeated timing is practical. Keep benchmark scripts in `scripts/bench/` and results in CI artifacts or release notes, not as generated repository noise.

Hardware-specific numbers are targets, not universal promises. A target may be revised only with recorded evidence and an explanation.

## 3. Startup budget

### Targets

- cold start to first visible usable window: <= 900 ms p50 and <= 1500 ms p95 on the reference baseline
- warm start to usable window: <= 650 ms p95
- no external media tool is spawned during ordinary app startup
- no recursive filesystem scan during startup
- no network request during startup except requests explicitly caused by user content already being restored; MVP restores no remote content

### Design rules

- load the small settings file once
- render the shell immediately
- resolve `yt-dlp`, FFmpeg, FFprobe, and MediaInfo lazily on first operation that needs each tool
- cache successful tool resolutions for the process lifetime
- do not permanently cache a failed lookup, so a tool installed while the app is open can be discovered later
- exact known paths are checked before any bounded directory walk
- development fallback discovery under `youtube-downloader/` must have a bounded maximum depth and stop at the first valid executable according to resolution priority

## 4. Distribution budget

The application must not ship an embedded browser engine. Use the Windows WebView2 runtime through Tauri.

Frontend rules:

- React + TypeScript remains approved
- no router for the one-screen MVP
- no UI component framework
- no global state library unless profiling demonstrates a need
- no icon package; use a tiny local SVG set
- no remote font; use the system UI font stack
- no runtime CSS-in-JS
- no analytics/telemetry SDK

Release Rust profile should favor small code without sacrificing orchestration responsiveness:

```toml
[profile.release]
opt-level = "s"
lto = true
codegen-units = 1
panic = "abort"
strip = true
```

This profile must be benchmarked against the default release profile before final release. Keep it only if startup/runtime do not regress materially.

Third-party tool payload rules:

- never bundle `ffplay.exe`
- never bundle FFmpeg documentation, presets, archives, or duplicate copies
- bundle/download only executables and license notices required by the selected release strategy
- developer-local FFmpeg remains ignored and must never enter Git history
- verify release-downloaded third-party binaries with pinned SHA-256 before execution

## 5. CPU / runtime budget

### Idle

With the window open, no analysis/download active:

- average app CPU target: <= 0.5% over 60 seconds on the reference baseline
- no busy loop
- no polling loop for job state
- no repeating timer faster than 1 Hz while idle
- ideally no app-owned repeating timer at all while idle
- no progress/event work when no child process is active

### Active orchestration

Rust/React overhead must remain small relative to `yt-dlp`/FFmpeg:

- line parsing is linear in received output size
- no repeated parsing of the same progress record
- no full re-render of the app for every raw child-process line
- post-processing is delegated to FFmpeg rather than reimplemented
- video must not be decoded/re-encoded unless the selected user intent requires transcoding
- remux/copy is preferred over transcode when it satisfies the preset

CPU-heavy future features must run outside the UI event path and must not be introduced in MVP.

## 6. Memory budget and data structures

### Targets

After 60 seconds idle on the reference baseline:

- Rust host process working-set target: <= 35 MiB
- process-tree memory is recorded separately because WebView2 uses multiple processes and some components may be shared by the OS
- memory use during a download must remain O(metadata + bounded diagnostics + child-process buffers), never O(media file size)

### Mandatory structures

Use the simplest cache-friendly standard structure that matches access semantics.

#### Progress state

Progress is latest-value state, not a historical queue.

- use latest-value semantics (`watch`-style) inside the backend
- overwrite stale progress with newer progress
- never enqueue every progress sample indefinitely
- emit to the frontend at a coalesced maximum of 4 progress updates/second
- state transitions, errors, completion, and cancellation bypass coalescing and emit immediately

#### Discrete events

For events that must not be dropped:

- use a bounded MPSC queue with backpressure
- initial capacity: 64 events
- capacity changes require a stress-test reason
- unbounded application queues are prohibited

#### Diagnostics

Technical diagnostics use a fixed-capacity ring:

- `VecDeque`-style ring behavior
- keep at most 256 recent lines
- total retained diagnostic text <= 64 KiB per job
- drop oldest diagnostic lines first
- never retain complete unbounded stderr/stdout history in RAM

#### Resolutions

Format lists are small and iteration-heavy. Use contiguous storage:

1. collect numeric heights into `Vec<u32>`
2. `sort_unstable`
3. `dedup`
4. reverse for descending display

Do not use a tree/set allocation per format unless a benchmark proves it better.

#### Active job

MVP concurrency is one job, so model active state as `Option<ActiveJob>` rather than a general-purpose hash map. Introduce `VecDeque<Job>` only when queue support is implemented.

#### Presets

Use a closed Rust enum and exhaustive `match`. Do not use a dynamic string-to-command dictionary for core presets.

### Metadata parsing

- deserialize directly into typed Serde structs containing only required fields
- ignore unknown fields without storing an all-purpose `serde_json::Value` tree
- metadata stdout has a hard safety cap of 8 MiB for MVP single-item analysis
- exceeding the cap returns a typed `MetadataTooLarge` error rather than allocating without limit

## 7. Async/runtime model

Use Tauri's existing async runtime rather than creating another general-purpose runtime.

Rules:

- do not create a second Tokio runtime
- async tasks handle process I/O, cancellation, state changes, and lightweight parsing
- genuinely blocking operations use Tauri/Tokio blocking facilities instead of blocking async worker threads
- do not create a custom thread pool in MVP
- no Rayon dependency in MVP
- no custom allocator in MVP; allocator changes require benchmark evidence showing a material win in RSS/latency without reliability regressions

The process runner must stream stdout/stderr instead of waiting for the full process output, except bounded metadata JSON where a complete typed decode is required.

## 8. I/O and storage amplification

The app must never proxy media bytes through Rust or JavaScript.

Data path:

```text
network -> yt-dlp -> destination/.part -> FFmpeg merge/remux if required -> final file
```

Mandatory rules:

- Rust never reads/writes the media payload
- React never receives media payload bytes
- no duplicate application-owned temporary media copy
- prefer temp files on the destination volume so finalization can use rename rather than cross-volume copy
- preserve yt-dlp resume behavior using `.part` files
- do not hash downloaded media after completion
- MediaInfo/FFprobe inspection runs once and does not duplicate the media file
- settings are not written during progress events
- settings writes occur only after an actual settings change
- use temp-file + replace semantics for settings persistence to reduce corruption risk
- output naming includes stable media ID to reduce collision risk

Storage policy:

- source-preserving Best Audio does not convert to FLAC/MP3
- Best Video avoids transcode where possible
- MP3/FLAC perform only the audio conversion explicitly requested
- no cache directory containing duplicate completed downloads
- no persistent thumbnail cache in MVP

## 9. Concurrency policy

### Job-level concurrency

MVP maximum active download jobs: `1`.

This avoids competing disk writes, child-process memory spikes, bandwidth contention, UI complexity, and battery waste.

### Fragment concurrency

`yt-dlp` currently defaults to one concurrent fragment. Keep `-N 1` / tool default as the baseline.

A future automatic policy may select 2 or 4 concurrent fragments only when benchmark data for fragmented streams shows:

- >= 20% median throughput improvement,
- <= 10% additional orchestration + downloader CPU relative to the baseline,
- no meaningful increase in retry/failure rate,
- bounded memory remains inside contract,
- no storage-I/O regression that harms finalization.

Never use an unbounded or CPU-count-sized fragment concurrency formula by default.

Concurrency is a controlled resource, not a score to maximize.

## 10. Progress and UI event algorithm

Raw process progress may arrive much faster than a human can perceive.

Pipeline:

```text
stdout progress record
        |
        v
parse once
        |
        v
latest backend state (replace old)
        |
        +---- immediate terminal/state event
        |
        v
250 ms coalescer while active
        |
        v
Tauri event
        |
        v
React state update
```

Requirements:

- maximum ordinary progress UI rate: 4 Hz
- completion/failure/cancel/status-stage transitions: immediate
- no animation loop is used to synthesize fake progress
- browser `requestAnimationFrame` is not kept active while idle

## 11. UI/UX performance

The MVP is a single-screen utility.

Rules:

- URL input and primary action are immediately usable
- paste event may analyze immediately; typed input uses a short debounce
- use a monotonically increasing analysis generation ID so stale analysis responses cannot overwrite the latest URL
- cancel superseded analysis when feasible; otherwise discard stale result
- thumbnail is lazy/non-blocking and never blocks metadata text or Download controls
- skeletons/spinners must not delay actionable controls
- respect `prefers-reduced-motion`
- no continuous visual animation
- avoid blur/backdrop-filter and other high-cost compositing effects
- prefer transform/opacity only for short necessary transitions
- system font, local SVG icons, plain CSS
- accessibility: keyboard operation, visible focus, semantic controls, status announcements

Target production frontend payload for app-owned JS + CSS: <= 120 KiB gzip. This excludes the system WebView runtime and media tool binaries.

## 12. Reliability

### Download semantics

- keep yt-dlp's resumable `.part` behavior
- keep retries bounded for ordinary downloads; no infinite retry in MVP
- use yt-dlp-supported retry/backoff rather than implementing a competing HTTP retry layer in Rust
- never report success until the child exits successfully and the exact final path exists
- MediaInfo failure does not convert a successful download into failure

### Process lifecycle on Windows

All external work spawned for one download must belong to one owned process tree.

Preferred Windows implementation:

- assign the root downloader process to a Windows Job Object
- configure kill-on-job-close behavior
- cancellation terminates the owned job/process tree so FFmpeg descendants do not remain orphaned
- app shutdown also closes active owned jobs

If Tauri sidecar behavior prevents this exact mechanism, implement an equivalent verified process-tree termination strategy and document the evidence.

### State machine

Illegal state transitions are rejected in the Rust domain model. UI cannot directly mutate backend job state.

### Crash behavior

- completed files are never deleted by recovery logic
- `.part` files remain available for yt-dlp resume
- settings corruption falls back to safe defaults and preserves the corrupt file for diagnostics when practical

## 13. Security

### Principle

The frontend requests typed operations. It never receives a general-purpose shell primitive.

Mandatory controls:

- no `cmd /c`
- no PowerShell command-string interpolation
- no shell-concatenated URL/path commands
- spawn executable + argument vector directly
- only `http://` and `https://` source URLs are accepted in MVP
- reject embedded NUL and invalid path input
- canonicalize/validate output directories at the filesystem boundary
- `Open` and `Open Folder` use only the final path returned by the owned download job
- verify the final path exists before opening it
- do not expose an unrestricted Tauri shell capability to JavaScript
- use minimum Tauri capabilities required by the UI
- no remote JavaScript
- strict CSP appropriate to Tauri IPC plus remote thumbnail images
- no `eval`
- no secrets/cookies/tokens in ordinary logs
- sanitize technical diagnostics before presenting/exporting them
- redact URL query values when they may contain authentication material

Release supply chain:

- lock Rust and frontend dependencies
- pin third-party binary versions
- verify third-party binary checksums
- keep license notices required by redistributed tools
- CI performs dependency/security checks without auto-upgrading production dependencies blindly

The project does not bypass DRM, access controls, paid-media protections, or platform authorization.

## 14. Energy / hardware policy

Energy is optimized using measurable proxies because exact package power telemetry is not universally available.

Primary energy proxies:

- idle CPU
- wakeup/timer frequency
- child-process concurrency
- bytes written to disk
- unnecessary transcoding
- UI animation/compositing activity

Rules:

- event-driven waiting only; no spin wait
- no background polling for updates
- no automatic dependency-update check during startup
- no downloader process until user initiates analysis/download
- analysis process exits immediately after metadata is obtained
- no GPU compute workload in MVP
- no hardware encoder/decoder initialization in MVP
- use stream copy/remux rather than video transcoding whenever the selected preset allows it
- one active download by default
- progress coalescing limits wakeups
- hidden/minimized window must not continue cosmetic updates

A future throughput/turbo mode must be explicit or evidence-backed adaptive behavior; it cannot silently become the default if it materially increases energy use.

## 15. Observability without permanent overhead

Production defaults:

- no telemetry
- no background metrics collector
- concise error ring only

Development/benchmark builds may expose counters behind a compile-time/dev flag:

- startup milestones
- progress records parsed
- frontend progress events emitted
- diagnostic bytes retained
- child process count
- tool-resolution duration
- settings read/write count

Performance instrumentation must compile out or remain dormant with negligible overhead in production.

## 16. Benchmark gates

Before calling MVP optimized/ready, record at minimum:

1. 20 cold/warm startup samples
2. 60-second idle CPU and memory sample
3. frontend compressed asset size
4. tool-resolution timing with cached and uncached paths
5. progress stress test with >= 10,000 synthetic raw progress records proving bounded memory and <= 4 Hz frontend emission
6. 8 MiB metadata boundary test
7. cancellation/process-tree test proving no orphan FFmpeg process
8. path-with-spaces and Unicode path test
9. interrupted download/resume smoke test
10. same media downloaded as Best Video versus a transcoding preset, recording CPU time and output size to verify source-preserving behavior

A performance regression is not accepted only because functional tests pass.

## 17. Optimization decision rule

For every proposed optimization, record:

```text
hypothesis -> benchmark -> result -> decision
```

Reject an optimization when:

- the improvement is below measurement noise,
- it adds complexity without a material metric gain,
- it increases memory/CPU/I/O/energy elsewhere more than the target gain is worth,
- it weakens reliability or security,
- it duplicates behavior already optimized by yt-dlp/FFmpeg/OS/WebView2.

This rule is intentional: the fastest small application is usually the one that avoids doing work, avoids copies, avoids allocations, avoids wakeups, and delegates specialized work to the already optimized media tools.

## 18. Primary implementation references

Use primary/current documentation when implementation details are version-sensitive:

- Tauri external sidecars: https://v2.tauri.app/develop/sidecar/
- Tauri async runtime API: https://docs.rs/tauri/latest/tauri/async_runtime/
- Tokio bounded MPSC/backpressure: https://docs.rs/tokio/latest/tokio/sync/mpsc/
- yt-dlp options/progress/concurrency/retry behavior: https://github.com/yt-dlp/yt-dlp/blob/master/README.md

Exact dependency versions are resolved and locked during implementation; do not hard-code a stale version from this design document.
