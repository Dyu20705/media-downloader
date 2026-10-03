# Packaging and release distribution

## Current production scope

The current production distribution target is **Linux x86_64 only**, packaged as a Debian `.deb`. Windows x86_64 and Intel macOS remain future production targets.

Pull-request CI deliberately continues to build unsigned Windows NSIS and Intel macOS DMG packages with Tauri's `--no-sign` mode. Those jobs detect portability and bundling regressions without requiring production signing credentials. They are not release artifacts and must not be presented as signed or production-supported downloads.

The Windows and Apple signing helper scripts remain in the repository for future enablement, but the production release workflow does not invoke them and does not require any Windows or Apple secrets.

## Linux release authenticity

Linux release authenticity uses three independent pieces of evidence:

1. the exact `.deb` is inspected and launched under Xvfb before publication;
2. GitHub build provenance is attested for the staged Debian artifact;
3. a dedicated release GPG key signs both the Debian artifact and `SHA256SUMS`.

The release also publishes the corresponding public GPG key. A consumer can therefore verify either the package directly or the signed checksum manifest.

## Required `production-release` environment secrets

Do not commit signing material. Configure only these secrets in GitHub **Settings → Environments → production-release → Environment secrets**:

| Secret | Required format / meaning |
| --- | --- |
| `RELEASE_GPG_PRIVATE_KEY` | ASCII-armored dedicated release private key containing exactly one primary secret key. |
| `RELEASE_GPG_PASSPHRASE` | Passphrase for the dedicated release GPG private key. |

A manual `workflow_dispatch` from `main` performs a fast, non-publishing signing preflight. It verifies that the current main SHA is still current, validates version-source consistency, imports the protected GPG key into an ephemeral keyring, and performs a real sign/verify probe. It does **not** rebuild the application.

## Production tag flow

A stable production tag must satisfy all of the following:

- the tagged commit is contained in `main`;
- the tag is exactly `v<version>`;
- `package.json`, `src-tauri/tauri.conf.json`, `src-tauri/Cargo.toml`, and `src-tauri/crates/core/Cargo.toml` contain the same SemVer;
- `CHANGELOG.md` contains exactly the release version section expected by the release-note extractor;
- the version-scoped RustSec policy is valid for that version.

The tag-triggered workflow then performs the full frontend, Rust, dependency-audit, package, smoke-test, provenance, checksum, and GPG-signing gates against that exact SHA. The Debian artifact is built once in that release run and the same staged bytes are signed, attested, preserved as a workflow artifact, and uploaded to the GitHub Release.

The protected `production-release` environment is the final credential boundary. If environment reviewers are configured, approving the tag-triggered release job is the human publication gate.

## Version and historical releases

Existing public tags/releases must not be deleted or reused to make the new pipeline fit historical state. The repository currently contains historical `v1.0.0` and `v1.0.1` releases while the current development metadata still reports `1.0.0`. Therefore the next production version must be chosen explicitly and synchronized in a dedicated release/version change before creating a new tag.

The security disposition in [release-remediation.md](release-remediation.md) scopes its two RustSec exceptions to `v1.0.0`. A later version must not silently inherit those exceptions; the audit policy intentionally requires re-evaluation.

No Windows or macOS production package is published by the current release workflow.
