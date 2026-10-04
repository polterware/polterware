#!/usr/bin/env bash
# Real-core, real-notification gate. Uses a TEMP database, TEMP Electron user-data-dir,
# and a throwaway bundle identity. Never touches ~/Library/Application Support/Sparky.
set -euo pipefail
NATIVE="$(cd "$(dirname "$0")/.." && pwd)"
DESKTOP="$(cd "$NATIVE/.." && pwd)"
ROOT="$(cd "$DESKTOP/../.." && pwd)"
[[ "$(uname -s)" == Darwin ]] || { echo 'FAIL: native E2E requires macOS' >&2; exit 1; }

WORKDIR="$(mktemp -d /tmp/sparky-w1-native-notify-c.XXXXXX)"
DB="$WORKDIR/db/sparky.db"
USERDATA="$WORKDIR/userdata"
BIN="$WORKDIR/bin/sparky"
APP="$WORKDIR/SparkyE2E.app"
RESULT="$WORKDIR/inspect.json"
ELECTRON_PID=""
CORE_PID=""
cleanup() {
  local code=$?
  set +e
  if [[ -n "${ELECTRON_PID}" ]] && kill -0 "$ELECTRON_PID" 2>/dev/null; then
    local exe
    exe="$(ps -p "$ELECTRON_PID" -o comm= 2>/dev/null || true)"
    if [[ "$exe" == *Electron* || "$exe" == *SparkyE2E* ]]; then kill "$ELECTRON_PID" 2>/dev/null; sleep 0.4; kill -9 "$ELECTRON_PID" 2>/dev/null; fi
  fi
  if [[ -x "$APP/Contents/MacOS/Electron" && -f "$NATIVE/build/notify.node" ]]; then
    SPARKY_NOTIFY_E2E=1 "$APP/Contents/MacOS/Electron" --user-data-dir="$WORKDIR/inspect-userdata" "$NATIVE/scripts/e2e-inspect.cjs" cleanup "$NATIVE/build/notify.node" "$RESULT" >/dev/null 2>&1 || true
  fi
  if [[ -n "${CORE_PID}" ]] && kill -0 "$CORE_PID" 2>/dev/null; then
    local core_exe
    core_exe="$(ps -p "$CORE_PID" -o args= 2>/dev/null || true)"
    if [[ "$core_exe" == "$BIN"* ]]; then kill "$CORE_PID" 2>/dev/null; sleep 0.3; kill -9 "$CORE_PID" 2>/dev/null; fi
  fi
  rm -rf "$WORKDIR"
  exit "$code"
}
trap cleanup EXIT

mkdir -p "$WORKDIR/db" "$WORKDIR/bin" "$USERDATA" "$NATIVE/build"
bash "$NATIVE/scripts/build.sh"
(cd "$ROOT" && go build -o "$BIN" ./sparky/backend/runtimes/cli)
BANNER="$("$BIN")"
if [[ "$BANNER" == *'(stub)'* ]]; then
  echo "BLOCKED: real core unavailable: $BANNER" >&2
  exit 2
fi
pnpm --dir "$DESKTOP" exec electron-vite build

ELECTRON_PKG="$(node -p "require('node:path').dirname(require.resolve('electron/package.json',{paths:[process.argv[1]]}))" "$DESKTOP")"
if [[ ! -d "$ELECTRON_PKG/dist/Electron.app" ]]; then
  node "$ELECTRON_PKG/install.js"
fi
ditto "$ELECTRON_PKG/dist/Electron.app" "$APP"
PLIST="$APP/Contents/Info.plist"
/usr/libexec/PlistBuddy -c 'Set :CFBundleIdentifier com.polterware.sparky.e2e-w1' "$PLIST"
/usr/libexec/PlistBuddy -c 'Set :CFBundleName Sparky E2E' "$PLIST"
/usr/libexec/PlistBuddy -c 'Set :CFBundleDisplayName Sparky E2E' "$PLIST"
CODE="$APP/Contents/Resources/app"
mkdir -p "$CODE/native/build"
cp "$DESKTOP/package.json" "$CODE/package.json"
ditto "$DESKTOP/out" "$CODE/out"
cp "$NATIVE/build/notify.node" "$CODE/native/build/notify.node"
codesign --force --deep --sign - --identifier com.polterware.sparky.e2e-w1 "$APP"
codesign --verify --deep --strict "$APP"

export SPARKY_CORE_BIN="$BIN"
export SPARKY_CORE_DB="$DB"
export SPARKY_NOTIFY_E2E=1
"$APP/Contents/MacOS/Electron" "$CODE" --user-data-dir="$USERDATA" &
ELECTRON_PID=$!

node "$NATIVE/scripts/e2e-driver.mjs" \
  --workdir "$WORKDIR" \
  --db "$DB" \
  --bin "$BIN" \
  --app "$APP" \
  --code "$CODE" \
  --userdata "$USERDATA" \
  --electron-pid "$ELECTRON_PID" \
  --notify "$CODE/native/build/notify.node" \
  --inspect "$NATIVE/scripts/e2e-inspect.cjs" \
  --result "$RESULT"
CORE_PID="$(node -e 'try{console.log(JSON.parse(require("node:fs").readFileSync(process.argv[1],"utf8")).pid||"")}catch{console.log("")}' "$WORKDIR/db/run/core.json")"
echo "PASS: real-core native notification e2e ($WORKDIR)"
