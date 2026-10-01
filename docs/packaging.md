# Packaging & Release Distribution — One-Click Media Downloader

## 0. Release status and required evidence

The repository has a tag-based signed release workflow and a Linux packaged launch smoke test. No signed release has been produced; Windows/macOS launch checks and the real credential-backed signing/notarization path remain unverified. Local bundles are development artifacts.

## 1. Distribution Strategy

The release matrix builds a Debian package for Linux x64, a Windows 10/11 x64 NSIS installer, and a macOS DMG on the hosted macOS runner architecture. Cross-platform core tests run on Ubuntu, Windows, and macOS.

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
  - Settings: `%APPDATA%\one-click-media-downloader\settings.json` on Windows (see `settings.rs`); platform config directory on macOS/Linux.
  - Download history: `%LOCALAPPDATA%\openDownloader\downloads.sqlite3` on Windows; the equivalent local application-data directory on macOS/Linux.
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

## Signed release workflow

Push a tag such as v1.0.0 only after the matching version commit is on dev and CHANGELOG.md contains that version. .github/workflows/release.yml builds a Debian package, Windows NSIS installer, and macOS DMG. The workflow rejects mismatched tags and tags outside dev history. Windows signing and verification, macOS Developer ID signing and notarization, and a GPG-signed SHA-256 manifest all fail closed when their secrets are absent. The final publish step requires the protected production-release GitHub environment.

Configure these repository/environment secrets before a production tag:
- Windows: WINDOWS_CERTIFICATE (base64 PFX), WINDOWS_CERTIFICATE_PASSWORD, WINDOWS_CERTIFICATE_THUMBPRINT.
- macOS: APPLE_CERTIFICATE (base64 P12), APPLE_CERTIFICATE_PASSWORD, APPLE_SIGNING_IDENTITY, APPLE_API_ISSUER, APPLE_API_KEY, APPLE_API_KEY_CONTENT, APPLE_TEAM_ID.
- Manifest: RELEASE_GPG_PRIVATE_KEY, RELEASE_GPG_PASSPHRASE.

The workflow does not configure an in-app updater. A future updater requires signed update metadata and a separate tested key-rotation/recovery policy.
