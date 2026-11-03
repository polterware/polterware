#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
fail=0
for path in backend/runtimes/server backend/runtimes/node; do
  if [[ -d "$ROOT/$path" ]]; then
    # Fail if directory exists and is empty or only has placeholders without real consumers
    count=$(find "$ROOT/$path" -type f ! -name '.gitkeep' 2>/dev/null | wc -l | tr -d ' ')
    if [[ "$count" -eq 0 ]]; then
      echo "FAIL: scenic empty package: $path"
      fail=1
    fi
  fi
done
if [[ $fail -ne 0 ]]; then
  exit 1
fi
echo "OK: no scenic empty server/node packages"
