#!/usr/bin/env bash
# Real-core gate. Notification execution is blocked until requests 1/2 are integrated;
# a synthetic core or the W0 acceptance transcript must never satisfy this gate.
set -euo pipefail
NATIVE="$(cd "$(dirname "$0")/.." && pwd)"
ROOT="$(cd "$NATIVE/../../.." && pwd)"
[[ "$(uname -s)" == Darwin ]] || { echo 'FAIL: native E2E requires macOS' >&2; exit 1; }
mkdir -p "$NATIVE/build"
(cd "$ROOT" && go build -o "$NATIVE/build/e2e-sparky" ./sparky/backend/runtimes/cli)
# status is read-only and does not start a server or modify a default personal database.
CORE_STATUS="$("$NATIVE/build/e2e-sparky" status)"
if [[ "$CORE_STATUS" == *'(stub)'* ]]; then
  echo "BLOCKED: real core unavailable: $CORE_STATUS" >&2
  echo 'See plans/sparky-next/requests/w1-native-notify-2.md (W1-CORE / W1-ELECTRON)' >&2
  exit 2
fi
# Stop this sub-task rather than reaching outside Owns to add shell test automation.
# W1-ELECTRON must provide an isolated installed bundle/supervisor test fixture and
# W0-CONTRACTS must provide Repeat before real one-shot/repeat/action acceptance.
echo 'BLOCKED: production Main integration and real-core notification fixture pending requests 1/2' >&2
exit 2
