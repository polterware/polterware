#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
export PATH="$PATH:$(go env GOPATH 2>/dev/null)/bin:${HOME}/go/bin"
cd "$ROOT"

if [[ -f sparky/schemas/buf.yaml ]]; then
  (cd sparky/schemas && buf lint)
  if [[ -f sparky/schemas/.buf/baseline.binpb ]]; then
    (cd sparky/schemas && buf breaking --against .buf/baseline.binpb) || {
      echo "buf breaking reported changes — update sparky/schemas/.buf/baseline.binpb intentionally"
      exit 1
    }
  fi
fi

(cd sparky/backend && go test ./... -count=1)
(cd sparky/backend && go vet ./...)

pnpm --filter "@polterware/*sparky*" typecheck
pnpm --filter "@polterware/*sparky*" test

bash tooling/check-sparky-boundaries.sh
node tooling/check-ui-opacity.mjs

node --test tooling/check-ui-opacity.test.mjs
bash tooling/check-no-scenic.sh
bash tooling/check-domain-imports.sh
bash tooling/check-ts-boundaries.sh
bash tooling/check-legacy.sh
node -e "const p=require('./apps/lore-desktop/package.json'); const d={...p.dependencies,...p.devDependencies}; if(d.react||d['react-dom']) process.exit(1)"
test ! -d apps/lore-desktop/src/renderer-solid

echo "verify-sparky OK"
