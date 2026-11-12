#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
DOMAINS="$ROOT/core/src/domains"
# Domain modules may depend on other domains' public APIs, but not concrete infrastructure.
bad=$(rg -n '\b(rusqlite|axum|portable_pty)::|crate::providers::|crate::runtimes::' "$DOMAINS" --glob '*.rs' || true)
if [[ -n "$bad" ]]; then
  echo "FAIL: domains import concrete infrastructure:"
  echo "$bad"
  exit 1
fi
echo "OK: domain import boundaries (Rust visibility enforces private internals)"
