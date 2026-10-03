#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/../.."

# Core has no exceptions. Keep warning denial and yanked-package checks enabled.
cargo audit --deny warnings --file src-tauri/crates/core/Cargo.lock

host_version="$(
  awk -F' = ' '
    /^\[package\]$/ { in_package = 1; next }
    /^\[/ && in_package { exit }
    in_package && /^version = / {
      gsub(/"/, "", $2)
      print $2
      exit
    }
  ' src-tauri/Cargo.toml
)"

if [ -z "$host_version" ]; then
  echo 'Unable to determine the Tauri host package version for RustSec policy.' >&2
  exit 1
fi

if [ "$host_version" = "1.0.0" ]; then
  # These exceptions were explicitly dispositioned for v1.0.0 only.
  cargo audit     --deny warnings     --ignore RUSTSEC-2024-0370     --ignore RUSTSEC-2024-0429     --file src-tauri/Cargo.lock
else
  # Later releases must not silently inherit the v1.0.0 waiver.
  cargo audit --deny warnings --file src-tauri/Cargo.lock
fi
