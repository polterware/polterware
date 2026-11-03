#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
fail=0
# Shared frontend must not import electron
if rg -n "from ['\"]electron['\"]|require\\(['\"]electron['\"]\\)" \
  "$ROOT/packages/frontend" "$ROOT/packages/features" \
  --glob '*.{ts,tsx}' 2>/dev/null | rg -v 'node_modules' | rg -q .; then
  echo "FAIL: electron import outside desktop"
  fail=1
fi
# Features must not deep-import other features internals (simple heuristic)
if rg -n "from ['\"]@polterware/features-[^'\"]+/src/" \
  "$ROOT/packages/features" --glob '*.{ts,tsx}' 2>/dev/null | rg -q .; then
  echo "FAIL: feature internal deep imports"
  fail=1
fi
if [[ $fail -ne 0 ]]; then
  exit 1
fi
echo "OK: ts boundaries (basic)"
