#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

if [[ "${1:-}" == "--core-only" ]]; then
  DB_PATH="${POLTERWARE_DEV_DB:-$ROOT/.data/dev-local.db}"
  mkdir -p "$(dirname "$DB_PATH")"
  exec cargo run -p polterware-core --bin polterware -- -db "$DB_PATH" serve
fi

if [[ $# -gt 0 ]]; then
  echo "Usage: ./scripts/dev.sh [--core-only]" >&2
  exit 1
fi

# Build the core where the Desktop looks for it.
cargo build -p polterware-core --bin polterware
mkdir -p dist
cp target/debug/polterware dist/polterware
export POLTERWARE_BIN="$ROOT/dist/polterware"
exec cargo run -p polterware-gpui
