#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
bash tooling/check-no-scenic.sh
bash tooling/check-domain-imports.sh
bash tooling/generate.sh --check
cargo fmt --check
cargo clippy --all-targets
cargo test
echo "verify OK"
