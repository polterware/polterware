#!/usr/bin/env bash
# Continues W0-NOTIFIER evidence after notification permission is granted.
# Does not rebuild the helper, so the TCC grant for the current signature stays valid
# until the launchd signing experiment at the end.
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
DATA="$ROOT/spikes/notifier/run"
DIST="$ROOT/spikes/notifier/dist"
EVIDENCE="$ROOT/spikes/notifier/evidence"
REPORT="$EVIDENCE/report.txt"
APP="$ROOT/apps/sparky-notifier/dist/SparkyNotifier.app"
BIN="$APP/Contents/MacOS/SparkyNotifier"
MEM="01ARZ3NDEKTSV4RRFFQ69G5FAV"
HOME_LOG="$HOME/Library/Logs/sparky-notifier-spike.log"
UID_NUM="$(id -u)"
AGENT_LABEL="com.polterware.sparky.notifier"
CORE_PID=""
SERVE_PID=""
ELECTRON_PID=""

note() { printf '%s %s\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)" "$*" | tee -a "$REPORT"; }
kill_pid() {
  local pid="${1:-}"
  [[ -n "$pid" ]] || return 0
  if kill -0 "$pid" 2>/dev/null; then
    kill -9 "$pid" 2>/dev/null || true
    note "killed pid=$pid"
  fi
}
cleanup() {
  set +e
  kill_pid "$SERVE_PID"
  kill_pid "$CORE_PID"
  kill_pid "$ELECTRON_PID"
  SPARKY_DATA_DIR="$DATA" "$BIN" --remove-prefix "spike-limit-" >/dev/null 2>&1 || true
  SPARKY_DATA_DIR="$DATA" "$BIN" --remove-prefix "rem:${MEM}" >/dev/null 2>&1 || true
  SPARKY_DATA_DIR="$DATA" "$BIN" --unregister-agent >/dev/null 2>&1 || true
  note "phase2 cleanup done"
}
trap cleanup EXIT

note "=== phase2 pending limit ==="
SPARKY_DATA_DIR="$DATA" "$BIN" --auth-status | tee -a "$REPORT"
SPARKY_DATA_DIR="$DATA" "$BIN" --measure-pending 100 | tee -a "$REPORT"
SPARKY_DATA_DIR="$DATA" "$BIN" --remove-prefix "spike-limit-" | tee -a "$REPORT"

note "=== phase2 plan arm ==="
mkdir -p "$DIST" "$DATA/run"
if [[ ! -x "$DIST/stubcore" ]]; then
  (cd "$ROOT/spikes/notifier/stubcore" && GOWORK=off go build -o "$DIST/stubcore" .)
fi
rm -f "$DATA/run/core.json"
"$DIST/stubcore" -data "$DATA" >"$DATA/run/stub.err" 2>&1 &
CORE_PID=$!
for _ in $(seq 1 25); do
  [[ -f "$DATA/run/core.json" ]] && break
  sleep 0.2
done
ENDPOINT="$(python3 -c 'import json; print(json.load(open("'"$DATA"'/run/core.json"))["endpoint"])')"
TOKEN="$(python3 -c 'import json; print(json.load(open("'"$DATA"'/run/core.json"))["token"])')"
note "core endpoint-port ${ENDPOINT##*:} pid=$CORE_PID"
FIRE="$(python3 -c 'from datetime import datetime, timedelta, timezone; print((datetime.now(timezone.utc)+timedelta(seconds=3600)).strftime("%Y-%m-%dT%H:%M:%S.000Z"))')"
curl -s -H "Authorization: Bearer ${TOKEN}" -H "Content-Type: application/json" \
  -d "{\"items\":[{\"id\":\"rem:${MEM}:1\",\"memoryId\":\"${MEM}\",\"fireAt\":\"${FIRE}\",\"title\":\"Plan spike\",\"body\":\"from plan\",\"category\":\"reminder\"}]}" \
  "${ENDPOINT}/plan" | tee -a "$REPORT"
echo | tee -a "$REPORT"
SPARKY_DATA_DIR="$DATA" SPARKY_CORE_BIN="$DIST/stubcore" "$BIN" --arm-plan | tee -a "$REPORT"
SPARKY_DATA_DIR="$DATA" "$BIN" --dump-pending | tee -a "$REPORT"
SPARKY_DATA_DIR="$DATA" "$BIN" --remove-prefix "rem:${MEM}" | tee -a "$REPORT"

note "=== phase2 electron ==="
ELECTRON_BIN="$(cd "$ROOT/apps/sparky-desktop" && node -e 'process.stdout.write(require("electron"))')"
ELECTRON_SRC="$(cd "$(dirname "$ELECTRON_BIN")/../.." && pwd)"
SPIKE_APP="$DIST/SparkySpike.app"
note "electron src=$ELECTRON_SRC"
rm -rf "$SPIKE_APP"
cp -R "$ELECTRON_SRC" "$SPIKE_APP"
PLIST="$SPIKE_APP/Contents/Info.plist"
/usr/libexec/PlistBuddy -c "Set :CFBundleIdentifier com.polterware.sparky.spike" "$PLIST"
/usr/libexec/PlistBuddy -c "Set :CFBundleName SparkySpike" "$PLIST"
/usr/libexec/PlistBuddy -c "Set :CFBundleDisplayName SparkySpike" "$PLIST"
/usr/libexec/PlistBuddy -c "Delete :CFBundleURLTypes" "$PLIST" >/dev/null 2>&1 || true
/usr/libexec/PlistBuddy -c "Add :CFBundleURLTypes array" "$PLIST"
/usr/libexec/PlistBuddy -c "Add :CFBundleURLTypes:0 dict" "$PLIST"
/usr/libexec/PlistBuddy -c "Add :CFBundleURLTypes:0:CFBundleURLName string sparky" "$PLIST"
/usr/libexec/PlistBuddy -c "Add :CFBundleURLTypes:0:CFBundleURLSchemes array" "$PLIST"
/usr/libexec/PlistBuddy -c "Add :CFBundleURLTypes:0:CFBundleURLSchemes:0 string sparky" "$PLIST"
mkdir -p "$SPIKE_APP/Contents/Resources/app" "$SPIKE_APP/Contents/Library/LoginItems" "$SPIKE_APP/Contents/Library/LaunchAgents"
cp "$ROOT/spikes/notifier/electron/main.js" "$SPIKE_APP/Contents/Resources/app/main.js"
cp "$ROOT/spikes/notifier/electron/package.json" "$SPIKE_APP/Contents/Resources/app/package.json"
cp -R "$APP" "$SPIKE_APP/Contents/Library/LoginItems/SparkyNotifier.app"
cat >"$SPIKE_APP/Contents/Library/LaunchAgents/${AGENT_LABEL}.plist" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key><string>${AGENT_LABEL}</string>
  <key>BundleProgram</key><string>Contents/Library/LoginItems/SparkyNotifier.app/Contents/MacOS/SparkyNotifier</string>
  <key>RunAtLoad</key><true/>
  <key>KeepAlive</key><true/>
</dict>
</plist>
EOF

try_electron() {
  local mode="$1"
  note "electron launch mode=$mode"
  SPARKY_DATA_DIR="$DATA" "$SPIKE_APP/Contents/MacOS/Electron" >"$DATA/run/electron-${mode}.out" 2>"$DATA/run/electron-${mode}.err" &
  local pid=$!
  sleep 2
  if kill -0 "$pid" 2>/dev/null; then
    note "electron mode=$mode running pid=$pid"
    kill "$pid" 2>/dev/null || true
    wait "$pid" 2>/dev/null || true
    return 0
  fi
  note "electron mode=$mode exited"
  tail -n 12 "$DATA/run/electron-${mode}.err" | tee -a "$REPORT" || true
  return 1
}

ELECTRON_MODE="modified-signature"
if ! try_electron modified-signature; then
  codesign --remove-signature "$SPIKE_APP" >>"$REPORT" 2>&1 || true
  ELECTRON_MODE="unsigned"
  if ! try_electron unsigned; then
    codesign --force --deep --sign - --identifier "com.polterware.sparky.spike" "$SPIKE_APP" >>"$REPORT" 2>&1 || true
    ELECTRON_MODE="adhoc"
    try_electron adhoc || note "electron failed ad-hoc too"
  fi
fi
note "electron mode that ran: $ELECTRON_MODE"
codesign -dv "$SPIKE_APP" >>"$REPORT" 2>&1 || note "spike electron has no signature"

rm -f "$HOME_LOG" "$DATA/run/electron.log"
SPARKY_DATA_DIR="$DATA" SPARKY_REGISTER=1 SPARKY_QUIT_AFTER=1 "$SPIKE_APP/Contents/MacOS/Electron" >"$DATA/run/electron-reg.out" 2>"$DATA/run/electron-reg.err" &
ELECTRON_PID=$!
sleep 3
kill_pid "$ELECTRON_PID"
ELECTRON_PID=""
note "electron register log:"
grep -E "register |unregister |protocol-registered|ready " "$DATA/run/electron.log" "$HOME_LOG" 2>/dev/null | tee -a "$REPORT" || true
SPARKY_DATA_DIR="$DATA" SPARKY_UNREGISTER=1 SPARKY_QUIT_AFTER=1 "$SPIKE_APP/Contents/MacOS/Electron" >>"$DATA/run/electron-reg.out" 2>>"$DATA/run/electron-reg.err" || true
sleep 1

rm -f "$HOME_LOG" "$DATA/run/electron.log"
SPARKY_DATA_DIR="$DATA" "$SPIKE_APP/Contents/MacOS/Electron" >"$DATA/run/electron-warm.out" 2>"$DATA/run/electron-warm.err" &
ELECTRON_PID=$!
for _ in $(seq 1 15); do
  grep -q "protocol-registered" "$DATA/run/electron.log" 2>/dev/null && break
  sleep 0.3
done
open "sparky://memory/${MEM}?warm=1" || note "open warm failed"
sleep 2
kill_pid "$ELECTRON_PID"
ELECTRON_PID=""
sleep 1
open "sparky://memory/${MEM}" || note "open cold failed"
sleep 3
note "deep-link log:"
grep -E "open-url|ready |protocol-registered" "$DATA/run/electron.log" "$HOME_LOG" 2>/dev/null | tee -a "$REPORT" || true
COLD_PID="$(cat "$HOME/Library/Logs/electron.pid" 2>/dev/null || true)"
[[ -z "$COLD_PID" ]] && COLD_PID="$(cat "$DATA/run/electron.pid" 2>/dev/null || true)"
kill_pid "$COLD_PID"

note "=== phase2 Done action, electron closed ==="
FIRE="$(python3 -c 'from datetime import datetime, timedelta, timezone; print((datetime.now(timezone.utc)+timedelta(seconds=4)).strftime("%Y-%m-%dT%H:%M:%S.000Z"))')"
DONE_ID="rem:${MEM}:done"
curl -s -H "Authorization: Bearer ${TOKEN}" -H "Content-Type: application/json" \
  -d "{\"items\":[{\"id\":\"${DONE_ID}\",\"memoryId\":\"${MEM}\",\"fireAt\":\"${FIRE}\",\"title\":\"Sparky\",\"body\":\"spike-done\",\"category\":\"reminder\"}]}" \
  "${ENDPOINT}/plan" | tee -a "$REPORT"
echo | tee -a "$REPORT"
SPARKY_DATA_DIR="$DATA" SPARKY_CORE_BIN="$DIST/stubcore" SPARKY_NOTIFIER_DEBUG=1 "$BIN" --serve >"$DATA/run/serve.out" 2>"$DATA/run/serve.err" &
SERVE_PID=$!
note "serve pid=$SERVE_PID (electron is not running)"
sleep 7
SPARKY_DATA_DIR="$DATA" "$BIN" --dump-delivered | tee -a "$REPORT"
osascript <<'EOF' >>"$REPORT" 2>&1 || true
tell application "System Events"
  if exists process "Notification Center" then
    tell process "Notification Center"
      repeat with w in windows
        try
          click button "Done" of w
          return "clicked Done"
        end try
        try
          click button "Concluído" of w
          return "clicked Concluido"
        end try
      end repeat
    end tell
  end if
end tell
return "no Done button"
EOF
sleep 1
note "events $(curl -s -H "Authorization: Bearer ${TOKEN}" "${ENDPOINT}/events")"
if [[ -f "$DATA/run/notifier.json" ]]; then
  DEBUG_URL="$(python3 -c 'import json; print(json.load(open("'"$DATA"'/run/notifier.json"))["debug"])')"
  note "debug $(curl -s -w ' status=%{http_code}' -H 'Content-Type: application/json' -d "{\"action\":\"DONE\",\"memoryId\":\"${MEM}\",\"identifier\":\"${DONE_ID}\"}" "$DEBUG_URL")"
fi
note "events-after-debug $(curl -s -H "Authorization: Bearer ${TOKEN}" "${ENDPOINT}/events")"
# Open action through the same handler the notification uses, while electron is stopped, then again warm.
kill_pid "$ELECTRON_PID"
SPARKY_DATA_DIR="$DATA" "$BIN" --simulate-action OPEN --memory "$MEM" --identifier "$DONE_ID" | tee -a "$REPORT"
sleep 2
note "open-after-simulate:"
grep -E "open-url|ready " "$HOME_LOG" "$DATA/run/electron.log" 2>/dev/null | tee -a "$REPORT" || true
kill_pid "$(cat "$HOME/Library/Logs/electron.pid" 2>/dev/null || true)"
kill_pid "$SERVE_PID"
SERVE_PID=""
SPARKY_DATA_DIR="$DATA" "$BIN" --remove-prefix "rem:${MEM}" | tee -a "$REPORT"

note "=== phase2 survival T+120 ==="
NONCE="deliver-after-kill-$(date +%s)"
SPARKY_DATA_DIR="$DATA" "$BIN" --arm-in 120 --memory "$MEM" --title "Sparky spike" --body "$NONCE" | tee -a "$REPORT"
kill_pid "$CORE_PID"
CORE_PID=""
pkill -9 -f "$DIST/stubcore" 2>/dev/null || true
pkill -9 -f "$APP/Contents/MacOS/SparkyNotifier" 2>/dev/null || true
pkill -9 -f "$SPIKE_APP/Contents/MacOS/Electron" 2>/dev/null || true
note "killed notifier, stub core and spike electron; sleeping 135s"
sleep 135
SPARKY_DATA_DIR="$DATA" "$BIN" --dump-delivered >"$DATA/run/delivered.out" 2>"$DATA/run/delivered.err" || true
note "delivered:"
cat "$DATA/run/delivered.out" | tee -a "$REPORT"
if grep -q "$NONCE" "$DATA/run/delivered.out"; then
  note "SURVIVAL PASS"
else
  note "SURVIVAL FAIL"
fi
SPARKY_DATA_DIR="$DATA" "$BIN" --remove-prefix "rem:${MEM}" | tee -a "$REPORT"

note "=== phase2 launchd with Apple Development identity ==="
DEV_ID="$(security find-identity -v -p codesigning | awk -F'\"' '/Apple Development/{print $2; exit}')"
note "dev identity=${DEV_ID:-none}"
if [[ -n "${DEV_ID:-}" ]]; then
  codesign --force --sign "$DEV_ID" --identifier "com.polterware.sparky.notifier" "$APP" >>"$REPORT" 2>&1
  codesign -dv "$APP" 2>&1 | tee -a "$REPORT" || true
  SPARKY_DATA_DIR="$DATA" "$BIN" --register-agent | tee -a "$REPORT"
  sleep 3
  launchctl print "gui/${UID_NUM}/${AGENT_LABEL}" 2>&1 | awk 'NR<=70 {print}' | tee -a "$REPORT" || true
  AGENT_PID="$(launchctl print "gui/${UID_NUM}/${AGENT_LABEL}" 2>/dev/null | awk '/pid = / { print $3; exit }')"
  note "dev-signed agent pid=${AGENT_PID:-missing}"
  if [[ -n "${AGENT_PID:-}" ]]; then
    kill -9 "$AGENT_PID" || true
    sleep 2
    AGENT_PID2="$(launchctl print "gui/${UID_NUM}/${AGENT_LABEL}" 2>/dev/null | awk '/pid = / { print $3; exit }')"
    note "dev-signed agent pid after kill -9=${AGENT_PID2:-missing}"
  fi
  SPARKY_DATA_DIR="$DATA" "$BIN" --unregister-agent | tee -a "$REPORT"
fi
note "=== phase2 end ==="
