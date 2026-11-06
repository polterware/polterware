#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
export PATH="$PATH:$(go env GOPATH 2>/dev/null)/bin:${HOME}/go/bin"
cd "$ROOT"
bash tooling/check-no-scenic.sh
bash tooling/check-domain-imports.sh
bash tooling/generate.sh
(cd backend && go test ./... -count=1)
(cd backend && go vet ./...)
cargo fmt --check
cargo clippy --all-targets
cargo test
echo "verify OK"
