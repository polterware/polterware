#!/usr/bin/env bash
# Spike checks for W0-MCP. Run from spikes/mcp or via this path.
set -euo pipefail
cd "$(dirname "$0")"
export GOWORK=off
gofmt_out="$(gofmt -l .)"
if [[ -n "$gofmt_out" ]]; then
  echo "gofmt needed:" >&2
  echo "$gofmt_out" >&2
  exit 1
fi
go vet ./...
go test ./...
echo "unit tests ok"
