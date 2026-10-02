# Packaging and release distribution

The release workflow is intended to produce a Debian package for Linux x86_64, an NSIS installer for Windows x86_64, and a DMG for Intel macOS. Each release package is built on its native runner. The Linux CI workflow includes package metadata inspection and a headless launch smoke test; Windows and macOS installed-app smoke tests require owner-run machines or VMs.

The stable release path is tag-triggered and must only publish a version whose commit is on `main`. Before native packaging, the release workflow runs the frontend boundary/type/lint/test/binding/build checks, locked Rust tests, formatting, Clippy, and npm/Rust dependency audits against the exact tagged SHA. The release quality job denies RustSec warnings, so known audit warnings block packaging until they are resolved or dispositioned by an authorized human. The separate PR checks remain required for branch integration.

An authorized human must review the release PR and explicitly accept the exact candidate commit before merge or tag creation. Signing credentials are supplied through the protected `production-release` GitHub Environment; they must never be stored in the repository. Windows Authenticode and macOS signing/notarization require protected owner credentials and native validation. Linux CI checks package metadata and launches the package; Windows/macOS install and startup checks require owner-controlled machines or VMs. See the workflow for exact required secrets and gates.

No production release has been published by this working branch. Do not treat locally generated bundles as signed production artifacts.
