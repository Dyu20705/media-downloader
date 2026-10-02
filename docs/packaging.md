# Packaging and release distribution

The release workflow produces a Debian package for Linux x86_64, an NSIS installer for Windows x86_64, and a DMG for Intel macOS. Each production package is built on its native runner. PR CI builds unsigned Windows NSIS and Intel macOS DMG packages with Tauri's `--no-sign` mode so platform bundling failures are discovered before the protected release workflow; Linux CI additionally inspects and launches the Debian package under Xvfb.

The stable release path is tag-triggered and must only publish a version whose commit is on `main`. A manual `workflow_dispatch` from `main` is the signed rehearsal: it must target the exact current `main` SHA, validate all production credentials before the expensive quality/package stages, run the full frontend/Rust/security suite, build and verify all three native packages, attest provenance, and produce a GPG-signed combined checksum manifest without publishing a GitHub Release. If `main` moves during that rehearsal, the workflow fails and must be rerun from the new `main` HEAD.

## Required `production-release` environment secrets

Do not commit any signing material. Configure these in GitHub **Settings → Environments → production-release → Environment secrets**:

| Secret | Required format / meaning |
| --- | --- |
| `WINDOWS_CERTIFICATE` | Base64 of an exportable PFX/PKCS#12 code-signing certificate **including the private key**. |
| `WINDOWS_CERTIFICATE_PASSWORD` | Password used when the PFX was exported. |
| `WINDOWS_CERTIFICATE_THUMBPRINT` | 40-hex SHA-1 thumbprint of that code-signing certificate. |
| `APPLE_CERTIFICATE` | Single-line base64 of the exported **Developer ID Application** `.p12`, including its private key. |
| `APPLE_CERTIFICATE_PASSWORD` | Password used when the Apple `.p12` was exported. |
| `APPLE_SIGNING_IDENTITY` | Exact Developer ID identity, e.g. `Developer ID Application: Name (TEAMID)`, as shown by `security find-identity -v -p codesigning`. |
| `APPLE_API_ISSUER` | App Store Connect API key issuer UUID. |
| `APPLE_API_KEY` | App Store Connect API Key ID. |
| `APPLE_API_KEY_CONTENT` | Raw contents of `AuthKey_<KEY_ID>.p8` including the `BEGIN PRIVATE KEY` / `END PRIVATE KEY` lines; **not base64**. |
| `RELEASE_GPG_PRIVATE_KEY` | ASCII-armored dedicated release private key; exactly one primary secret key. |
| `RELEASE_GPG_PASSPHRASE` | Passphrase for the release GPG private key. |

The credential preflight runs all three lanes with `fail-fast: false`: Windows validates the PFX, thumbprint, private key, certificate validity and Windows SDK signing tool; macOS imports the Developer ID certificate into an ephemeral keychain and uses `notarytool history` to authenticate the App Store Connect API key without submitting software; GPG imports the dedicated release key and performs a real sign/verify probe. This is intended to surface all credential problems in one workflow run rather than one at a time.

The Windows release path signs through `scripts/release/sign-windows.ps1`, verifies both the built application executable and final NSIS installer with Authenticode, and requires the configured signer thumbprint. The macOS release path verifies the app signature, Gatekeeper assessment, and stapled notarization tickets for both the app bundle and DMG. The final manifest exports only the configured release GPG key, signs `SHA256SUMS`, verifies that signature, and re-checks all package hashes.

An authorized human must review and accept the exact release candidate before tag creation. After a successful signed rehearsal, owner-controlled Windows/macOS install and startup smoke tests remain required. Only then create `v1.0.0` on the **same verified SHA**; the tag-triggered workflow reruns the same protected gates and publishes the GitHub Release.

No production release has been published by this working branch. Do not treat local bundles or failed/rehearsal artifacts as production releases.
