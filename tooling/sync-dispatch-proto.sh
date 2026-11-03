#!/usr/bin/env bash
# Copies the protos polterware/dispatch owns into this repository: its services into
# schemas/vendor/dispatch (TypeScript for the Desktop's Dispatch client) and
# DispatchHostService into schemas/polterware/local/v1 (served by the backend), then
# regenerates. --check only reports whether the copies differ.
#
#   tooling/sync-dispatch-proto.sh [--check] [<dispatch checkout>]
#
# The checkout defaults to $DISPATCH_DIR, else ../dispatch next to this repository.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
check=0
if [[ "${1:-}" == "--check" ]]; then
  check=1
  shift
fi
src="${1:-${DISPATCH_DIR:-$ROOT/../dispatch}}"
if [[ ! -d "$src/schemas/dispatch/v1" ]]; then
  echo "sync-dispatch-proto: no Dispatch checkout at $src" >&2
  exit 1
fi

host_header='option go_package = "github.com/polterware/polterware/backend/platform/gen/polterware/local/v1;localv1";'
# The host proto with this repository's go_package line after its package line.
host_proto() {
  awk -v line="$host_header" '{ print } /^package / { print line }' "$src/schemas/polterware/local/v1/dispatch_host.proto"
}

stale=0
for file in "$src"/schemas/dispatch/v1/*.proto; do
  target="$ROOT/schemas/vendor/dispatch/dispatch/v1/$(basename "$file")"
  if ! cmp -s "$file" "$target"; then
    stale=1
    [[ $check == 1 ]] || cp "$file" "$target"
  fi
done
host_target="$ROOT/schemas/polterware/local/v1/dispatch_host.proto"
if ! cmp -s <(host_proto) "$host_target"; then
  stale=1
  [[ $check == 1 ]] || host_proto > "$host_target"
fi

if [[ $check == 1 ]]; then
  if [[ $stale == 1 ]]; then
    echo "sync-dispatch-proto: the vendored protos differ from $src; run tooling/sync-dispatch-proto.sh" >&2
    exit 1
  fi
  echo "sync-dispatch-proto: up to date"
  exit 0
fi
bash "$ROOT/tooling/generate.sh"
