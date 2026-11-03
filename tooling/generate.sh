#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
export PATH="$PATH:$(go env GOPATH 2>/dev/null)/bin:${HOME}/go/bin"
cd "$ROOT"
if [[ -f schemas/buf.yaml ]]; then
  (cd schemas && buf lint && buf generate)
  if [[ -f schemas/.buf/baseline.binpb ]]; then
    (cd schemas && buf breaking --against .buf/baseline.binpb) || {
      echo "buf breaking reported changes — update schemas/.buf/baseline.binpb intentionally"
      exit 1
    }
  fi
fi
if [[ -f schemas/vendor/dispatch/buf.yaml ]]; then
  (cd schemas/vendor/dispatch && buf lint && buf generate)
fi
# protoc-gen-es ends some files with a blank line, which `git diff --check` rejects.
find packages/contracts/src/gen -name '*.ts' -exec perl -0pi -e 's/\n+\z/\n/' {} +
echo "generate done"
