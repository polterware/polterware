#!/usr/bin/env bash
# Follow-up evidence: action delivery, login-item relaunch, agent spawn from ~/Applications.
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
DATA="$ROOT/spikes/notifier/run"
DIST="$ROOT/spikes/notifier/dist"
REPORT="$ROOT/spikes/notifier/evidence/report.txt"
APP="$ROOT/apps/sparky-notifier/dist/SparkyNotifier.app"
BIN="$APP/Contents/MacOS/SparkyNotifier"
MEM="01ARZ3NDEKTSV4RRFFQ69G5FAV"
UID_NUM="$(id -u)"
AGENT_LABEL="com.polterware.sparky.notifier"
INSTALL="$HOME/Applications/SparkyNotifier.app"
HOST_APP="$DIST/SparkyHost.app"
CORE_PID=""
SERVE_PID=""

note() { printf '%s %s\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)" "$*" | tee -a "$REPORT"; }
kill_pid() {
  local pid="${1:-}"
  [[ -n "$pid" ]] || return 0
  if kill -0 "$pid" 2>/dev/null; then kill -9 "$pid" 2>/dev/null || true; note "killed pid=$pid"; fi
}
cleanup() {
  set +e
  kill_pid "$SERVE_PID"
  kill_pid "$CORE_PID"
  SPARKY_DATA_DIR="$DATA" "$BIN" --unregister-agent >/dev/null 2>&1 || true
  if [[ -x "$HOST_APP/Contents/MacOS/SparkyHost" ]]; then
    "$HOST_APP/Contents/MacOS/SparkyHost" --unregister-login-item >/dev/null 2>&1 || true
    "$HOST_APP/Contents/MacOS/SparkyHost" --unregister-agent >/dev/null 2>&1 || true
  fi
  if [[ -d "$INSTALL" ]]; then
    SPARKY_DATA_DIR="$DATA" "$INSTALL/Contents/MacOS/SparkyNotifier" --unregister-agent >/dev/null 2>&1 || true
    rm -rf "$INSTALL"
  fi
  SPARKY_DATA_DIR="$DATA" "$BIN" --remove-prefix "rem:${MEM}" >/dev/null 2>&1 || true
  note "phase3 cleanup done"
}
trap cleanup EXIT

note "=== phase3 Done reaches stub core, electron closed ==="
rm -f "$DATA/run/core.json"
mkdir -p "$DIST"
if [[ ! -x "$DIST/stubcore" ]]; then
  (cd "$ROOT/spikes/notifier/stubcore" && GOWORK=off go build -o "$DIST/stubcore" .)
fi
"$DIST/stubcore" -data "$DATA" >"$DATA/run/stub.err" 2>&1 &
CORE_PID=$!
for _ in $(seq 1 25); do [[ -f "$DATA/run/core.json" ]] && break; sleep 0.2; done
ENDPOINT="$(python3 -c 'import json; print(json.load(open("'"$DATA"'/run/core.json"))["endpoint"])')"
TOKEN="$(python3 -c 'import json; print(json.load(open("'"$DATA"'/run/core.json"))["token"])')"
pgrep -lf "SparkySpike.app/Contents/MacOS/Electron" >/dev/null && note "WARN electron still running" || note "electron is not running"
SPARKY_DATA_DIR="$DATA" "$BIN" --simulate-action DONE --memory "$MEM" --identifier "rem:${MEM}:manual" | tee -a "$REPORT"
note "events-after-cli $(curl -s -H "Authorization: Bearer ${TOKEN}" "${ENDPOINT}/events")"

SPARKY_DATA_DIR="$DATA" SPARKY_NOTIFIER_DEBUG=1 SPARKY_CORE_BIN="$DIST/stubcore" "$BIN" --serve >"$DATA/run/serve3.out" 2>"$DATA/run/serve3.err" &
SERVE_PID=$!
for _ in $(seq 1 20); do [[ -f "$DATA/run/notifier.json" ]] && break; sleep 0.2; done
DEBUG_URL="$(python3 -c 'import json; print(json.load(open("'"$DATA"'/run/notifier.json"))["debug"])')"
note "debug-url-port ${DEBUG_URL##*:}"
note "debug-post $(curl -s -w ' status=%{http_code}' -H 'Content-Type: application/json' -d '{"action":"DONE","memoryId":"'"$MEM"'","identifier":"rem:'"$MEM"':debug"}' "$DEBUG_URL")"
note "events-after-debug $(curl -s -H "Authorization: Bearer ${TOKEN}" "${ENDPOINT}/events")"

FIRE="$(python3 -c 'from datetime import datetime, timedelta, timezone; print((datetime.now(timezone.utc)+timedelta(seconds=2)).strftime("%Y-%m-%dT%H:%M:%S.000Z"))')"
curl -s -H "Authorization: Bearer ${TOKEN}" -H "Content-Type: application/json" \
  -d "{\"items\":[{\"id\":\"rem:${MEM}:click\",\"memoryId\":\"${MEM}\",\"fireAt\":\"${FIRE}\",\"title\":\"Sparky click\",\"body\":\"spike-click\",\"category\":\"reminder\"}]}" \
  "${ENDPOINT}/plan" >/dev/null
sleep 4
CLICK="$(osascript <<'EOF' 2>&1 || true
tell application "System Events"
  if exists process "Notification Center" then
    tell process "Notification Center"
      set hits to {}
      repeat with w in windows
        set names to {}
        try
          set names to name of buttons of w
        end try
        set end of hits to (names as text)
        repeat with b in {"Done", "Concluído"}
          try
            click button (contents of b) of w
            return "clicked " & (contents of b)
          end try
        end repeat
      end repeat
      return "buttons " & (hits as text)
    end tell
  end if
end tell
return "no notification center"
EOF
)"
note "os-click: $CLICK"
sleep 1
note "events-after-os-click $(curl -s -H "Authorization: Bearer ${TOKEN}" "${ENDPOINT}/events")"
kill_pid "$SERVE_PID"
SERVE_PID=""

note "=== phase3 login item relaunch ==="
HOST_BIN="$ROOT/spikes/notifier/host/.build/release/SparkyHost"
if [[ ! -x "$HOST_BIN" ]]; then
  swift build -c release --package-path "$ROOT/spikes/notifier/host" >>"$REPORT" 2>&1
fi
rm -rf "$HOST_APP"
mkdir -p "$HOST_APP/Contents/MacOS" "$HOST_APP/Contents/Library/LoginItems" "$HOST_APP/Contents/Library/LaunchAgents"
cp "$HOST_BIN" "$HOST_APP/Contents/MacOS/SparkyHost"
chmod +x "$HOST_APP/Contents/MacOS/SparkyHost"
cp -R "$APP" "$HOST_APP/Contents/Library/LoginItems/SparkyNotifier.app"
cat >"$HOST_APP/Contents/Info.plist" <<'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleExecutable</key><string>SparkyHost</string>
  <key>CFBundleIdentifier</key><string>com.polterware.sparky.spike.host</string>
  <key>CFBundleName</key><string>SparkyHost</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>CFBundleShortVersionString</key><string>0.1.0</string>
  <key>CFBundleVersion</key><string>1</string>
  <key>LSMinimumSystemVersion</key><string>15.0</string>
  <key>LSUIElement</key><true/>
  <key>NSPrincipalClass</key><string>NSApplication</string>
</dict>
</plist>
EOF
codesign --force --sign - --identifier "com.polterware.sparky.notifier" "$HOST_APP/Contents/Library/LoginItems/SparkyNotifier.app" >>"$REPORT" 2>&1
codesign --force --sign - --identifier "com.polterware.sparky.spike.host" "$HOST_APP" >>"$REPORT" 2>&1
"$HOST_APP/Contents/MacOS/SparkyHost" --register-login-item | tee -a "$REPORT"
sleep 2
LOGIN_PIDS="$(pgrep -f "Library/LoginItems/SparkyNotifier.app/Contents/MacOS/SparkyNotifier" | tr '\n' ' ' || true)"
note "login-item pids=${LOGIN_PIDS:-none}"
FIRST="$(echo "$LOGIN_PIDS" | awk '{print $1}')"
if [[ -n "${FIRST:-}" ]]; then
  kill -9 "$FIRST" || true
  sleep 2
  LOGIN_PIDS2="$(pgrep -f "Library/LoginItems/SparkyNotifier.app/Contents/MacOS/SparkyNotifier" | tr '\n' ' ' || true)"
  note "login-item pids after kill -9=${LOGIN_PIDS2:-none}"
fi
"$HOST_APP/Contents/MacOS/SparkyHost" --unregister-login-item | tee -a "$REPORT"
sleep 1
pkill -9 -f "Library/LoginItems/SparkyNotifier.app/Contents/MacOS/SparkyNotifier" 2>/dev/null || true

note "=== phase3 agent from ~/Applications ==="
# Patch a copy so launchd does not prompt and writes into the spike data dir.
rm -rf "$INSTALL"
mkdir -p "$HOME/Applications"
cp -R "$APP" "$INSTALL"
PLIST="$INSTALL/Contents/Library/LaunchAgents/${AGENT_LABEL}.plist"
/usr/libexec/PlistBuddy -c "Delete :EnvironmentVariables" "$PLIST" >/dev/null 2>&1 || true
/usr/libexec/PlistBuddy -c "Add :EnvironmentVariables dict" "$PLIST"
/usr/libexec/PlistBuddy -c "Add :EnvironmentVariables:SPARKY_DATA_DIR string ${DATA}" "$PLIST"
/usr/libexec/PlistBuddy -c "Add :EnvironmentVariables:SPARKY_NOTIFIER_SKIP_AUTH string 1" "$PLIST"
codesign --force --sign - --identifier "com.polterware.sparky.notifier" "$INSTALL" >>"$REPORT" 2>&1
SPARKY_DATA_DIR="$DATA" "$INSTALL/Contents/MacOS/SparkyNotifier" --register-agent | tee -a "$REPORT"
sleep 3
launchctl print "gui/${UID_NUM}/${AGENT_LABEL}" 2>&1 | awk 'NR<=40 {print}' | tee -a "$REPORT" || true
AGENT_PID="$(launchctl print "gui/${UID_NUM}/${AGENT_LABEL}" 2>/dev/null | awk '/pid = / { print $3; exit }')"
note "applications agent pid=${AGENT_PID:-missing}"
if [[ -n "${AGENT_PID:-}" ]]; then
  kill -9 "$AGENT_PID" || true
  sleep 2
  AGENT_PID2="$(launchctl print "gui/${UID_NUM}/${AGENT_LABEL}" 2>/dev/null | awk '/pid = / { print $3; exit }')"
  note "applications agent pid after kill -9=${AGENT_PID2:-missing}"
fi
SPARKY_DATA_DIR="$DATA" "$INSTALL/Contents/MacOS/SparkyNotifier" --unregister-agent | tee -a "$REPORT"
rm -rf "$INSTALL"

note "=== phase3 electron agent after ad-hoc sign ==="
SPIKE_APP="$DIST/SparkySpike.app"
if [[ -d "$SPIKE_APP" ]]; then
  codesign --force --deep --sign - --identifier "com.polterware.sparky.spike" "$SPIKE_APP" >>"$REPORT" 2>&1 || true
  SPARKY_DATA_DIR="$DATA" SPARKY_REGISTER=1 SPARKY_QUIT_AFTER=1 "$SPIKE_APP/Contents/MacOS/Electron" >"$DATA/run/electron-reg2.out" 2>"$DATA/run/electron-reg2.err" || true
  sleep 1
  note "electron ad-hoc register:"
  grep "register " "$DATA/run/run/electron.log" "$HOME/Library/Logs/sparky-notifier-spike.log" 2>/dev/null | tail -n 8 | tee -a "$REPORT" || true
  SPARKY_DATA_DIR="$DATA" SPARKY_UNREGISTER=1 SPARKY_QUIT_AFTER=1 "$SPIKE_APP/Contents/MacOS/Electron" >>"$DATA/run/electron-reg2.out" 2>>"$DATA/run/electron-reg2.err" || true
fi
note "=== phase3 end ==="
