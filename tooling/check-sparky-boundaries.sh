#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
fail=0

DOMAINS="$ROOT/sparky/backend/domains"
if [[ -d "$DOMAINS" ]]; then
  bad=$(rg -n 'modernc.org/sqlite|github.com/mattn/go-sqlite3|connectrpc.com/connect|github.com/jackc/pgx|electron|"database/sql"' \
    "$DOMAINS" --glob '*.go' 2>/dev/null || true)
  if echo "$bad" | rg -q .; then
    echo "FAIL: sparky domains import concrete infrastructure:"
    echo "$bad"
    fail=1
  fi
fi

if rg -n "from ['\"]electron['\"]|require\\(['\"]electron['\"]\\)" \
  "$ROOT/packages/frontend" "$ROOT/packages/features/sparky-"* \
  --glob '*.{ts,tsx}' 2>/dev/null | rg -v 'node_modules' | rg -q .; then
  echo "FAIL: electron import outside sparky-desktop"
  fail=1
fi

if rg -n "from ['\"]react['\"]|from ['\"]react-dom['\"]" \
  "$ROOT/apps/sparky-desktop" "$ROOT/packages/features/sparky-"* "$ROOT/packages/sparky-contracts" "$ROOT/packages/frontend/map" \
  --glob '*.{ts,tsx}' 2>/dev/null | rg -v 'node_modules' | rg -q .; then
  echo "FAIL: React import in Sparky packages"
  fail=1
fi

sparky_features=("$ROOT/packages/features"/sparky-*)
for feature_dir in "${sparky_features[@]}"; do
  [[ -d "$feature_dir" ]] || continue
  name=$(basename "$feature_dir")
  if [[ "$name" == "sparky-shell" ]]; then
    continue
  fi
  if rg -n "from ['\"]@polterware/feature-sparky-[^'\"]+/(?!feature)" \
    "$feature_dir/src" --glob '*.{ts,tsx}' 2>/dev/null | rg -v 'node_modules' | rg -q .; then
    echo "FAIL: cross-feature deep import in $name"
    fail=1
  fi
done

if [[ $fail -ne 0 ]]; then
  exit 1
fi
echo "OK: sparky boundaries"
