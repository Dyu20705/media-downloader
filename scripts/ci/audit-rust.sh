#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/../.."

# Core has no exceptions. Keep warning denial and yanked-package checks enabled.
cargo audit --deny warnings --file src-tauri/crates/core/Cargo.lock

# Accepted upstream GTK/glib exceptions for v1.0.0 only. Re-evaluate before
# every future release; see docs/release-remediation.md for residual risk.
cargo audit \
  --deny warnings \
  --ignore RUSTSEC-2024-0370 \
  --ignore RUSTSEC-2024-0429 \
  --file src-tauri/Cargo.lock
