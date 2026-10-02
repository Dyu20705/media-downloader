# Third-party notices

The application can download and execute these separately distributed tools. They are not embedded in the desktop package. Their versions and platform artifacts are defined in `src-tauri/crates/core/src/tool_manager.rs`.

The desktop application also uses these direct runtime dependencies:

- **Tauri and Tauri dialog plugin** — Apache-2.0 or MIT. [Tauri](https://github.com/tauri-apps/tauri), [dialog plugin](https://github.com/tauri-apps/plugins-workspace)
- **React and React DOM** — MIT. [React](https://github.com/facebook/react)
- **Lucide React** — ISC. [Lucide](https://github.com/lucide-icons/lucide)

The remaining direct and transitive npm/Rust dependencies are pinned in `package-lock.json` and the Cargo lockfiles. Their package manifests identify their applicable licenses; the project does not relicense those dependencies.

- **yt-dlp** — Unlicense. Some official executable formats bundle other libraries under their respective licenses; consult the upstream [third-party license notices](https://github.com/yt-dlp/yt-dlp/blob/master/THIRD_PARTY_LICENSES.txt) for the exact release artifact. [License](https://github.com/yt-dlp/yt-dlp/blob/master/LICENSE)
- **FFmpeg and ffprobe 9.0.2** — the managed Linux and Windows assets use BtbN's GPL build; the Intel macOS assets use Evermeet's FFmpeg 9.0.2 builds. These builds are distributed under GPL-3.0. [BtbN release](https://github.com/BtbN/FFmpeg-Builds/releases/tag/autobuild-2026-09-28-13-06), [Evermeet](https://evermeet.cx/ffmpeg/), [FFmpeg legal information](https://ffmpeg.org/legal.html)
- **MediaInfo CLI 24.12 (current managed pin)** — BSD-2-Clause. MediaArea's official pages confirm CLI 26.05 is released, but exact official Windows and Intel macOS CLI artifact hashes have not been obtained and independently calculated in this environment. Updating the pin is a release blocker until those archives are verified. [License](https://mediaarea.net/en/MediaInfo/License), [official Windows download](https://mediaarea.net/en/MediaInfo/Download/Windows), [official change log](https://mediaarea.net/MediaInfo/ChangeLog)

Users who redistribute the tools must review the selected artifact's license and comply with its terms. The project does not bundle the upstream tool binaries into application installers.
