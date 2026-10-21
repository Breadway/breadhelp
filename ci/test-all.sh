#!/usr/bin/env bash
# Lint and test everything, inside the CI image (run it through ci/build.sh).
#
# Usage: ci/build.sh bash ci/test-all.sh [extra cargo args, e.g. --locked]
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."

npm --prefix frontend ci
npm --prefix frontend run check
npm --prefix frontend test
# tauri::generate_context! needs the frontend's output dir to exist.
npm --prefix frontend run build
cargo clippy --all-targets "$@" -- -D warnings
cargo clippy --all-targets --manifest-path crates/breadhelp-tour/Cargo.toml "$@" -- -D warnings
cargo test "$@"
cargo test --manifest-path crates/breadhelp-tour/Cargo.toml "$@"
