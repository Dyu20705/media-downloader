# Packaging & Release Distribution — One-Click Media Downloader

## 1. Distribution Strategy

One-Click Media Downloader targets Windows 10/11 x64 as a standalone executable (`.exe`) and lightweight installer (`.exe` via NSIS).

### 1.1 WebView2 Strategy
- **Mode**: `downloadBootstrapper` (recommended default).
- **Rationale**: Windows 10 (recent updates) and Windows 11 include Evergreen WebView2 pre-installed. The bootstrapper checks for WebView2 and only downloads the runtime if missing, keeping the installer download size small (~5–10 MB instead of ~160 MB fixed runtime).

---

## 2. Windows Installer Configuration (NSIS)

Configured in `src-tauri/tauri.conf.json`:
- **Installer Type**: NSIS (`.exe`)
- **Installation Mode**: `currentUser` (installs to `%LOCALAPPDATA%\Programs\OneClickMediaDownloader`, requires no admin UAC elevation)
- **Application Identifiers**:
  - Name: `One-Click Media Downloader`
  - Version: `1.0.0`
  - Identifier: `com.oneclick.media.downloader`
  - Publisher: `One-Click Media Downloader Team`
- **Application Data Locations**:
  - Settings: `%LOCALAPPDATA%\one-click-media-downloader\settings.json`
  - Managed Tools: `%LOCALAPPDATA%\OneClickMediaDownloader\tools\`

---

## 3. Production Build Commands

### Step 1: Build Frontend
```bash
npm run build
```
Generates production assets in `dist/`.

### Step 2: Build Tauri Release Bundle
```bash
npm run tauri build
# Or directly via cargo:
cargo tauri build
```
Outputs:
- Standalone Executable: `src-tauri/target/release/one-click-media-downloader.exe`
- NSIS Installer: `src-tauri/target/release/bundle/nsis/One-Click Media Downloader_1.0.0_x64-setup.exe`

---

## 4. Production Code Signing Workflow

To distribute on Windows without SmartScreen security warnings, binaries and installers must be code-signed.

> **Security Rule**: Private signing keys, PFX certificates, and hardware token credentials must **NEVER** be committed to the source repository.

### Signing Command Specification
Using Microsoft `signtool.exe`:

```cmd
:: 1. Sign application executable
signtool.exe sign /v /fd SHA256 /tr http://timestamp.digicert.com /td SHA256 /sha1 <CERTIFICATE_THUMBPRINT> "src-tauri\target\release\one-click-media-downloader.exe"

:: 2. Sign NSIS setup installer
signtool.exe sign /v /fd SHA256 /tr http://timestamp.digicert.com /td SHA256 /sha1 <CERTIFICATE_THUMBPRINT> "src-tauri\target\release\bundle\nsis\One-Click Media Downloader_1.0.0_x64-setup.exe"
```

### CI/CD Signing Integration
In GitHub Actions / Azure DevOps:
1. Store certificate in GitHub Encrypted Secrets (`WINDOWS_CERTIFICATE_BASE64`, `WINDOWS_CERTIFICATE_PASSWORD`).
2. Decode to secure runner temporary directory.
3. Sign using `signtool` before publishing release artifacts.
4. Immediately wipe certificate files from runner disk.
