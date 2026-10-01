# Packaging and release distribution

The release workflow is intended to produce a Debian package for Linux x86_64, an NSIS installer for Windows x86_64, and a DMG for Intel macOS. Each release package is built on its native runner. The Linux CI workflow includes package metadata inspection and a headless launch smoke test; Windows and macOS installed-app smoke tests require owner-run machines or VMs.

The stable release path is tag-triggered and must only publish a version whose commit is on `main`. Signing credentials are supplied through the protected `production-release` GitHub Environment; they must never be stored in the repository. See the workflow for the exact required secrets and gates.

No production release has been published by this working branch. Do not treat locally generated bundles as signed production artifacts.
