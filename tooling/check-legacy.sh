#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
fail=0

die() {
  echo "FAIL: $*"
  fail=1
}

# Forbidden legacy trees / packages on disk
for p in legacy packages/core packages/ui apps/desktop/src/renderer-solid; do
  if [[ -e "$ROOT/$p" ]]; then
    die "forbidden path still present: $p"
  fi
done

# Product desktop must not depend on React
if [[ -f "$ROOT/apps/desktop/package.json" ]]; then
  if node -e "
    const p=require('$ROOT/apps/desktop/package.json');
    const d={...p.dependencies||{},...p.devDependencies||{}};
    if (d.react || d['react-dom']) process.exit(2);
  "; then
    :
  else
    code=$?
    if [[ $code -eq 2 ]]; then
      die "apps/desktop must not depend on react/react-dom"
    else
      die "could not parse apps/desktop/package.json"
    fi
  fi
fi

# Official Solid entry only
if [[ ! -f "$ROOT/apps/desktop/src/renderer/main.tsx" ]]; then
  die "missing apps/desktop/src/renderer/main.tsx"
fi
if ! rg -q 'src/renderer' "$ROOT/apps/desktop/electron.vite.config.ts" 2>/dev/null; then
  die "electron.vite.config.ts must point at src/renderer"
fi
if rg -q 'renderer-solid' "$ROOT/apps/desktop/electron.vite.config.ts" 2>/dev/null; then
  die "electron.vite.config.ts still references renderer-solid"
fi

# No mock workbench in product graph
if rg -n 'createMockWorkbenchAdapter' \
  "$ROOT/apps/desktop/src/renderer" \
  "$ROOT/apps/desktop/src/main" \
  2>/dev/null | rg -q .; then
  die "mock workbench in product graph"
fi

# No live package imports of removed TS core/ui (ignore specs/docs/migration history)
if rg -n '@polterware/core|@polterware/ui|packages/core|packages/ui' \
  "$ROOT/apps" "$ROOT/packages" "$ROOT/backend" \
  --glob '!**/node_modules/**' \
  --glob '!**/out/**' \
  2>/dev/null | rg -q .; then
  die "live code still references packages/core or packages/ui"
  rg -n '@polterware/core|@polterware/ui|packages/core|packages/ui' \
    "$ROOT/apps" "$ROOT/packages" "$ROOT/backend" \
    --glob '!**/node_modules/**' --glob '!**/out/**' 2>/dev/null | head -20 || true
fi

if [[ $fail -ne 0 ]]; then
  exit 1
fi
echo "OK: legacy checks"
