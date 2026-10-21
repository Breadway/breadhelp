#!/usr/bin/env bash
# Builds everything breadhelp ships, inside the CI image (run it through
# ci/build.sh): the Svelte frontend, then the help window (root workspace)
# and the tour overlay (its own workspace, see the root Cargo.toml).
#
# Usage: ci/build.sh bash ci/build-all.sh [extra cargo args, e.g. --locked]
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."

npm --prefix frontend ci
npm --prefix frontend run build
cargo build --release "$@"
cargo build --release --manifest-path crates/breadhelp-tour/Cargo.toml "$@"
