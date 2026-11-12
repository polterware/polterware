#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
cargo fmt --check
cargo clippy --all-targets
cargo test -p polterware-core
