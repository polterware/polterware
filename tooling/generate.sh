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
# The Desktop's Rust contracts come from the same descriptors (no buf plugin for them).
descriptor="$(mktemp -t polterware-descriptor).json"
dispatch_descriptor="$(mktemp -t polterware-dispatch-descriptor).json"
trap 'rm -f "$descriptor" "$dispatch_descriptor"' EXIT
cargo build --quiet -p gen-contracts
gen="$ROOT/target/debug/gen-contracts"
if [[ -f schemas/buf.yaml ]]; then
  (cd schemas && buf build -o "$descriptor#format=json")
  "$gen" "$descriptor" app/src/contracts "$@"
fi
if [[ -f schemas/vendor/dispatch/buf.yaml ]]; then
  (cd schemas/vendor/dispatch && buf lint && buf build -o "$dispatch_descriptor#format=json")
  "$gen" "$dispatch_descriptor" app/src/dispatch/contracts --client crate::dispatch::Dispatch "$@"
fi
echo "generate done"
