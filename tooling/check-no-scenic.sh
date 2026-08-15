#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
fail=0
for path in apps/web backend/runtimes/server backend/runtimes/node; do
  if [[ -d "$ROOT/$path" ]]; then
    # Fail if directory exists and is empty or only has placeholders without real consumers
    count=$(find "$ROOT/$path" -type f ! -name '.gitkeep' 2>/dev/null | wc -l | tr -d ' ')
    if [[ "$count" -eq 0 ]]; then
      echo "FAIL: scenic empty package: $path"
      fail=1
    fi
  fi
done
# Fail if empty packages were created as stubs without go.mod / package.json consumers
if [[ -d "$ROOT/apps/web" ]] && [[ ! -f "$ROOT/apps/web/package.json" ]]; then
  echo "FAIL: apps/web without package.json (scenic)"
  fail=1
fi
if [[ $fail -ne 0 ]]; then
  exit 1
fi
echo "OK: no scenic empty web/server/node packages"
