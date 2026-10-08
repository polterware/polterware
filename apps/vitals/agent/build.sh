#!/usr/bin/env bash
# Cross-compile the read-only vitals agent for Linux. No CGO, no extra linker flags.
set -euo pipefail

cd "$(dirname "$0")"
out_dir="../resources/agent"
mkdir -p "${out_dir}"

for arch in amd64 arm64; do
  CGO_ENABLED=0 GOOS=linux GOARCH="${arch}" \
    go build -trimpath -ldflags "-s -w" \
    -o "${out_dir}/vitals-agent-linux-${arch}" .
done
