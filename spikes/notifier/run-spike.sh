#!/usr/bin/env bash
# W0-NOTIFIER evidence run. Kills only spike PIDs recorded by this script.
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
DATA="$ROOT/spikes/notifier/run"
DIST="$ROOT/spikes/notifier/dist"
EVIDENCE="$ROOT/spikes/notifier/evidence"
REPORT="$EVIDENCE/report.txt"
APP="$ROOT/apps/sparky-notifier/dist/SparkyNotifier.app"
BIN="$APP/Contents/MacOS/SparkyNotifier"
MEM="01ARZ3NDEKTSV4RRFFQ69G5FAV"
UID_NUM="$(id -u)"
KEEPALIVE_LABEL="com.polterware.sparky.notifier.keepalive"
AGENT_LABEL="com.polterware.sparky.notifier"
HOME_LOG="$HOME/Library/Logs/sparky-notifier-spike.log"
HOST_INSTALL="$HOME/Applications/SparkyHost.app"
CORE_PID=""
ELECTRON_PID=""
SERVE_PID=""

mkdir -p "$EVIDENCE" "$DATA/run" "$DIST"
: >"$REPORT"

note() {
  printf '%s %s\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)" "$*" | tee -a "$REPORT"
}

excerpt() {
  note "--- $* ---"
  if [[ -f "$2" ]]; then
    grep -F "$3" "$2" | tail -n 40 | tee -a "$REPORT" || true
  else
    note "missing $2"
  fi
}

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
  launchctl bootout "gui/${UID_NUM}/${KEEPALIVE_LABEL}" >/dev/null 2>&1 || true
  if [[ -x "$BIN" ]]; then
    SPARKY_DATA_DIR="$DATA" "$BIN" --unregister-agent >/dev/null 2>&1 || true
    SPARKY_DATA_DIR="$DATA" "$BIN" --remove-prefix "spike-limit-" >/dev/null 2>&1 || true
    SPARKY_DATA_DIR="$DATA" "$BIN" --remove-prefix "rem:${MEM}" >/dev/null 2>&1 || true
  fi
  if [[ -x "$DIST/SparkyHost.app/Contents/MacOS/SparkyHost" ]]; then
    "$DIST/SparkyHost.app/Contents/MacOS/SparkyHost" --unregister-agent >/dev/null 2>&1 || true
    "$DIST/SparkyHost.app/Contents/MacOS/SparkyHost" --unregister-login-item >/dev/null 2>&1 || true
  fi
  if [[ -d "$HOST_INSTALL" ]]; then
    "$HOST_INSTALL/Contents/MacOS/SparkyHost" --unregister-agent >/dev/null 2>&1 || true
    "$HOST_INSTALL/Contents/MacOS/SparkyHost" --unregister-login-item >/dev/null 2>&1 || true
    rm -rf "$HOST_INSTALL"
  fi
  rm -f "$HOME_LOG" "$HOME/Library/Logs/electron.pid"
  note "cleanup done"
}
trap cleanup EXIT

run_notifier() {
  SPARKY_DATA_DIR="$DATA" "$BIN" "$@" >"$DATA/run/last.out" 2>"$DATA/run/last.err"
  local code=$?
  note "notifier $* exit=${code}"
  cat "$DATA/run/last.out" | tee -a "$REPORT"
  return 0
}

try_click() {
  local result
  result="$(osascript <<'EOF' 2>&1 || true
tell application "System Events"
  set procNames to {"Sparky", "SparkyNotifier", "UserNotificationCenter", "CoreServicesUIAgent"}
  set buttonNames to {"Allow", "Permitir", "OK", "Done"}
  repeat with procName in procNames
    set procText to contents of procName
    if exists process procText then
      tell process procText
        repeat with buttonName in buttonNames
          set labelText to contents of buttonName
          try
            click button labelText of window 1
            return "clicked " & procText & " " & labelText
          end try
        end repeat
      end tell
    end if
  end repeat
end tell
return "none"
EOF
)"
  note "click: ${result}"
}

prompt_text() {
  local result
  result="$(osascript <<'EOF' 2>&1 || true
tell application "System Events"
  if exists process "Sparky" then
    tell process "Sparky"
      try
        return (value of static texts of windows) as text
      end try
    end tell
  end if
  if exists process "SparkyNotifier" then
    tell process "SparkyNotifier"
      try
        return (value of static texts of windows) as text
      end try
    end tell
  end if
end tell
return "none"
EOF
)"
  note "prompt-text: ${result}"
}

note "host $(sw_vers | tr '\n' ' ') $(uname -m)"
note "=== build ==="
bash "$ROOT/apps/sparky-notifier/build-app.sh" >>"$REPORT" 2>&1 || { note "notifier build failed"; exit 1; }
(cd "$ROOT/spikes/notifier/stubcore" && GOWORK=off go build -o "$DIST/stubcore" .) >>"$REPORT" 2>&1 || { note "stub build failed"; exit 1; }
swift build -c release --package-path "$ROOT/spikes/notifier/host" >>"$REPORT" 2>&1 || { note "host build failed"; exit 1; }

# Point the built agent's plist at the spike data dir, then re-sign. Source plist stays clean.
AGENT_PLIST="$APP/Contents/Library/LaunchAgents/${AGENT_LABEL}.plist"
/usr/libexec/PlistBuddy -c "Delete :EnvironmentVariables" "$AGENT_PLIST" >/dev/null 2>&1 || true
/usr/libexec/PlistBuddy -c "Add :EnvironmentVariables dict" "$AGENT_PLIST"
/usr/libexec/PlistBuddy -c "Add :EnvironmentVariables:SPARKY_DATA_DIR string ${DATA}" "$AGENT_PLIST"
/usr/libexec/PlistBuddy -c "Add :EnvironmentVariables:SPARKY_NOTIFIER_SKIP_AUTH string 1" "$AGENT_PLIST"
codesign --force --sign - --identifier "com.polterware.sparky.notifier" "$APP" >>"$REPORT" 2>&1

note "=== codesign / gatekeeper ==="
codesign -dv --verbose=4 "$APP" >>"$REPORT" 2>&1 || true
spctl --assess --verbose=4 --type execute "$APP" >>"$REPORT" 2>&1 || note "spctl rejected ad-hoc app (expected without Developer ID)"
excerpt "codesign" "$REPORT" "Signature="
excerpt "codesign authority" "$REPORT" "Authority="

note "=== sdk availability ==="
SDK="$(xcrun --show-sdk-path)"
note "sdk=$SDK"
SM_HEADER="$(find "$SDK/System/Library/Frameworks/ServiceManagement.framework" -name 'SMAppService.h' -print -quit 2>/dev/null || true)"
UN_HEADER="$SDK/System/Library/Frameworks/UserNotifications.framework/Headers/UNUserNotificationCenter.h"
note "SMAppService.h=$SM_HEADER"
if [[ -n "$SM_HEADER" ]]; then
  grep -n -E "API_AVAILABLE|SMAppService" "$SM_HEADER" | head -n 30 | tee -a "$REPORT" || true
fi
if [[ -f "$UN_HEADER" ]]; then
  grep -n -E "API_AVAILABLE|UNUserNotificationCenter" "$UN_HEADER" | head -n 20 | tee -a "$REPORT" || true
fi

note "=== authorization ==="
: >"$DATA/run/auth.out"
SPARKY_DATA_DIR="$DATA" "$BIN" --request-auth >"$DATA/run/auth.out" 2>"$DATA/run/auth.err" &
AUTH_PID=$!
AUTH_STATE="timeout"
for _ in $(seq 1 45); do
  if grep -q '"auth"' "$DATA/run/auth.out" 2>/dev/null; then
    AUTH_STATE="$(grep -o '"auth":"[^"]*"' "$DATA/run/auth.out" | head -n 1)"
    break
  fi
  prompt_text
  try_click
  sleep 1
done
if kill -0 "$AUTH_PID" 2>/dev/null; then
  kill "$AUTH_PID" 2>/dev/null || true
  wait "$AUTH_PID" 2>/dev/null || true
fi
note "auth-result ${AUTH_STATE}"
cat "$DATA/run/auth.out" | tee -a "$REPORT" || true
tail -n 20 "$DATA/run/auth.err" | tee -a "$REPORT" || true

AUTHORIZED=0
if grep -q '"auth":"authorized"' "$DATA/run/auth.out" 2>/dev/null || grep -q '"auth":"provisional"' "$DATA/run/auth.out" 2>/dev/null; then
  AUTHORIZED=1
fi

if [[ "$AUTHORIZED" == "1" ]]; then
  note "=== pending limit ==="
  run_notifier --measure-pending 100
  run_notifier --remove-prefix "spike-limit-"
else
  note "skip pending limit; notification permission was not granted"
fi

note "=== agent KeepAlive via SMAppService ==="
run_notifier --register-agent
sleep 2
AGENT_PID="$(launchctl print "gui/${UID_NUM}/${AGENT_LABEL}" 2>/dev/null | awk '/pid = / { print $3; exit }')"
note "agent pid before kill=${AGENT_PID:-missing}"
launchctl print "gui/${UID_NUM}/${AGENT_LABEL}" 2>&1 | head -n 40 | tee -a "$REPORT" || true
if [[ -n "${AGENT_PID:-}" ]]; then
  kill -9 "$AGENT_PID" 2>/dev/null || true
  sleep 2
  AGENT_PID2="$(launchctl print "gui/${UID_NUM}/${AGENT_LABEL}" 2>/dev/null | awk '/pid = / { print $3; exit }')"
  note "agent pid after kill -9=${AGENT_PID2:-missing}"
  if [[ -n "${AGENT_PID2:-}" && "$AGENT_PID2" != "$AGENT_PID" ]]; then
    note "agent restarted after kill -9"
  else
    note "agent did not restart after kill -9"
  fi
else
  note "agent had no pid; status is in the register line above"
fi
run_notifier --unregister-agent
sleep 1

note "=== login item from parent host ==="
HOST_BIN="$ROOT/spikes/notifier/host/.build/release/SparkyHost"
HOST_APP="$DIST/SparkyHost.app"
rm -rf "$HOST_APP"
mkdir -p "$HOST_APP/Contents/MacOS" "$HOST_APP/Contents/Library/LoginItems" "$HOST_APP/Contents/Library/LaunchAgents"
cp "$HOST_BIN" "$HOST_APP/Contents/MacOS/SparkyHost"
chmod +x "$HOST_APP/Contents/MacOS/SparkyHost"
cp -R "$APP" "$HOST_APP/Contents/Library/LoginItems/SparkyNotifier.app"
cat >"$HOST_APP/Contents/Library/LaunchAgents/${AGENT_LABEL}.plist" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key>
  <string>${AGENT_LABEL}</string>
  <key>BundleProgram</key>
  <string>Contents/Library/LoginItems/SparkyNotifier.app/Contents/MacOS/SparkyNotifier</string>
  <key>RunAtLoad</key>
  <true/>
  <key>KeepAlive</key>
  <true/>
  <key>AssociatedBundleIdentifiers</key>
  <array>
    <string>com.polterware.sparky.spike.host</string>
  </array>
  <key>EnvironmentVariables</key>
  <dict>
    <key>SPARKY_DATA_DIR</key>
    <string>${DATA}</string>
    <key>SPARKY_NOTIFIER_SKIP_AUTH</key>
    <string>1</string>
  </dict>
</dict>
</plist>
EOF
cat >"$HOST_APP/Contents/Info.plist" <<'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleExecutable</key>
  <string>SparkyHost</string>
  <key>CFBundleIdentifier</key>
  <string>com.polterware.sparky.spike.host</string>
  <key>CFBundleName</key>
  <string>SparkyHost</string>
  <key>CFBundleDisplayName</key>
  <string>SparkyHost</string>
  <key>CFBundlePackageType</key>
  <string>APPL</string>
  <key>CFBundleShortVersionString</key>
  <string>0.1.0</string>
  <key>CFBundleVersion</key>
  <string>1</string>
  <key>LSMinimumSystemVersion</key>
  <string>15.0</string>
  <key>LSUIElement</key>
  <true/>
  <key>NSPrincipalClass</key>
  <string>NSApplication</string>
</dict>
</plist>
EOF
codesign --force --sign - --identifier "com.polterware.sparky.notifier" "$HOST_APP/Contents/Library/LoginItems/SparkyNotifier.app" >>"$REPORT" 2>&1
codesign --force --sign - --identifier "com.polterware.sparky.spike.host" "$HOST_APP" >>"$REPORT" 2>&1 || note "host codesign failed"
"$HOST_APP/Contents/MacOS/SparkyHost" --register-login-item >"$DATA/run/host-login.out" 2>"$DATA/run/host-login.err" || true
note "host login-item from worktree:"
cat "$DATA/run/host-login.out" | tee -a "$REPORT" || true
"$HOST_APP/Contents/MacOS/SparkyHost" --unregister-login-item >>"$REPORT" 2>&1 || true
"$HOST_APP/Contents/MacOS/SparkyHost" --register-agent >"$DATA/run/host-agent.out" 2>"$DATA/run/host-agent.err" || true
note "host agent from worktree:"
cat "$DATA/run/host-agent.out" | tee -a "$REPORT" || true
"$HOST_APP/Contents/MacOS/SparkyHost" --unregister-agent >>"$REPORT" 2>&1 || true

if ! grep -q '"loginItem":"enabled"' "$DATA/run/host-login.out" 2>/dev/null; then
  note "copying host to ~/Applications and retrying login item"
  rm -rf "$HOST_INSTALL"
  mkdir -p "$HOME/Applications"
  cp -R "$HOST_APP" "$HOST_INSTALL"
  "$HOST_INSTALL/Contents/MacOS/SparkyHost" --register-login-item >"$DATA/run/host-login-apps.out" 2>"$DATA/run/host-login-apps.err" || true
  note "host login-item from ~/Applications:"
  cat "$DATA/run/host-login-apps.out" | tee -a "$REPORT" || true
  "$HOST_INSTALL/Contents/MacOS/SparkyHost" --unregister-login-item >>"$REPORT" 2>&1 || true
  rm -rf "$HOST_INSTALL"
fi

note "loginItem has no KeepAlive. A login item is started at login by SMAppService and is not restarted by launchd after kill -9. The agent plist sets KeepAlive=true, which is why the agent path above is the one that restarts."

note "=== stub core + plan arm ==="
"$DIST/stubcore" -data "$DATA" >"$DATA/run/stub.err" 2>&1 &
CORE_PID=$!
note "stub pid=$CORE_PID"
for _ in $(seq 1 25); do
  [[ -f "$DATA/run/core.json" ]] && break
  sleep 0.2
done
ENDPOINT="$(python3 -c 'import json; print(json.load(open("'"$DATA"'/run/core.json"))["endpoint"])')"
TOKEN="$(python3 -c 'import json; print(json.load(open("'"$DATA"'/run/core.json"))["token"])')"
HEALTH="$(curl -s -o "$DATA/run/health.json" -w '%{http_code}' -H "Authorization: Bearer ${TOKEN}" -H "Host: ${ENDPOINT#http://}" "${ENDPOINT}/health" || true)"
note "health status=$HEALTH"
BAD_HOST="$(curl -s -o /dev/null -w '%{http_code}' -H "Authorization: Bearer ${TOKEN}" -H "Host: evil.example" "${ENDPOINT}/health" || true)"
note "bad-host status=$BAD_HOST"
if [[ "$AUTHORIZED" == "1" ]]; then
  FIRE="$(python3 -c 'from datetime import datetime, timedelta, timezone; print((datetime.now(timezone.utc)+timedelta(seconds=3600)).strftime("%Y-%m-%dT%H:%M:%S.000Z"))')"
  curl -s -o "$DATA/run/plan-set.json" -H "Authorization: Bearer ${TOKEN}" -H "Content-Type: application/json" \
    -d "{\"items\":[{\"id\":\"rem:${MEM}:1\",\"memoryId\":\"${MEM}\",\"fireAt\":\"${FIRE}\",\"title\":\"Plan spike\",\"body\":\"from plan\",\"category\":\"reminder\"}]}" \
    "${ENDPOINT}/plan" >>"$REPORT" || true
  SPARKY_DATA_DIR="$DATA" SPARKY_CORE_BIN="$DIST/stubcore" "$BIN" --arm-plan >"$DATA/run/arm-plan.out" 2>"$DATA/run/arm-plan.err" || true
  note "arm-plan:"
  cat "$DATA/run/arm-plan.out" | tee -a "$REPORT" || true
  run_notifier --dump-pending
  run_notifier --remove-prefix "rem:${MEM}"
fi

note "=== electron identity null + ad-hoc + deep link ==="
DESKTOP="$ROOT/apps/sparky-desktop"
ELECTRON_BIN="$(cd "$DESKTOP" && node -e 'process.stdout.write(require("electron"))')"
ELECTRON_SRC="$(cd "$(dirname "$ELECTRON_BIN")/../.." && pwd)"
SPIKE_APP="$DIST/SparkySpike.app"
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
cp "$HOST_APP/Contents/Library/LaunchAgents/${AGENT_LABEL}.plist" "$SPIKE_APP/Contents/Library/LaunchAgents/${AGENT_LABEL}.plist"
rm -f "$HOME_LOG"

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
  note "electron mode=$mode exited immediately"
  tail -n 20 "$DATA/run/electron-${mode}.err" | tee -a "$REPORT" || true
  return 1
}

ELECTRON_MODE="modified-signature"
if ! try_electron "modified-signature"; then
  codesign --remove-signature "$SPIKE_APP" >>"$REPORT" 2>&1 || true
  ELECTRON_MODE="unsigned"
  if ! try_electron "unsigned"; then
    codesign --force --deep --sign - --identifier "com.polterware.sparky.spike" "$SPIKE_APP" >>"$REPORT" 2>&1 || true
    ELECTRON_MODE="adhoc"
    try_electron "adhoc" || note "electron did not launch ad-hoc either"
  fi
fi
note "electron launch that survived: $ELECTRON_MODE"
codesign -dv "$SPIKE_APP" >>"$REPORT" 2>&1 || note "electron codesign -dv: unsigned"

rm -f "$HOME_LOG" "$DATA/run/electron.log"
SPARKY_DATA_DIR="$DATA" SPARKY_REGISTER=1 SPARKY_QUIT_AFTER=1 "$SPIKE_APP/Contents/MacOS/Electron" >"$DATA/run/electron-reg.out" 2>"$DATA/run/electron-reg.err" &
ELECTRON_PID=$!
sleep 3
kill_pid "$ELECTRON_PID"
ELECTRON_PID=""
note "electron setLoginItemSettings:"
if [[ -f "$DATA/run/electron.log" ]]; then
  grep -E "register |protocol-registered|ready " "$DATA/run/electron.log" | tee -a "$REPORT" || true
else
  grep -E "register |protocol-registered|ready " "$HOME_LOG" | tee -a "$REPORT" || true
fi
SPARKY_DATA_DIR="$DATA" SPARKY_UNREGISTER=1 SPARKY_QUIT_AFTER=1 "$SPIKE_APP/Contents/MacOS/Electron" >>"$DATA/run/electron-reg.out" 2>>"$DATA/run/electron-reg.err" || true
sleep 1

rm -f "$HOME_LOG" "$DATA/run/electron.log"
SPARKY_DATA_DIR="$DATA" "$SPIKE_APP/Contents/MacOS/Electron" >"$DATA/run/electron-warm.out" 2>"$DATA/run/electron-warm.err" &
ELECTRON_PID=$!
sleep 2
open "sparky://memory/${MEM}?warm=1" || note "open warm url failed"
sleep 2
kill_pid "$ELECTRON_PID"
ELECTRON_PID=""
sleep 1
open "sparky://memory/${MEM}" || note "open cold url failed"
sleep 3
COLD_PID=""
if [[ -f "$HOME/Library/Logs/electron.pid" ]]; then
  COLD_PID="$(cat "$HOME/Library/Logs/electron.pid" 2>/dev/null || true)"
fi
if [[ -z "$COLD_PID" && -f "$DATA/run/electron.pid" ]]; then
  COLD_PID="$(cat "$DATA/run/electron.pid" 2>/dev/null || true)"
fi
note "cold electron pid=${COLD_PID:-missing}"
note "electron deep-link log:"
cat "$DATA/run/electron.log" 2>/dev/null | tee -a "$REPORT" || true
grep -E "open-url|ready |protocol-registered" "$HOME_LOG" 2>/dev/null | tee -a "$REPORT" || true
kill_pid "$COLD_PID"
ELECTRON_PID=""

if [[ "$AUTHORIZED" == "1" ]]; then
  note "=== Done action, Electron closed ==="
  FIRE="$(python3 -c 'from datetime import datetime, timedelta, timezone; print((datetime.now(timezone.utc)+timedelta(seconds=3)).strftime("%Y-%m-%dT%H:%M:%S.000Z"))')"
  DONE_ID="rem:${MEM}:done"
  curl -s -o "$DATA/run/plan-done.json" -H "Authorization: Bearer ${TOKEN}" -H "Content-Type: application/json" \
    -d "{\"items\":[{\"id\":\"${DONE_ID}\",\"memoryId\":\"${MEM}\",\"fireAt\":\"${FIRE}\",\"title\":\"Sparky\",\"body\":\"spike-done\",\"category\":\"reminder\"}]}" \
    "${ENDPOINT}/plan" >>"$REPORT" || true
  SPARKY_DATA_DIR="$DATA" SPARKY_CORE_BIN="$DIST/stubcore" SPARKY_NOTIFIER_DEBUG=1 "$BIN" --serve >"$DATA/run/serve.out" 2>"$DATA/run/serve.err" &
  SERVE_PID=$!
  note "serve pid=$SERVE_PID electron closed"
  sleep 6
  run_notifier --dump-delivered
  try_click
  sleep 2
  EVENTS="$(curl -s -H "Authorization: Bearer ${TOKEN}" "${ENDPOINT}/events" || true)"
  note "events-after-click $EVENTS"
  if [[ -f "$DATA/run/notifier.json" ]]; then
    DEBUG_URL="$(python3 -c 'import json; print(json.load(open("'"$DATA"'/run/notifier.json"))["debug"])')"
    DEBUG_CODE="$(curl -s -o "$DATA/run/debug-action.json" -w '%{http_code}' -H 'Content-Type: application/json' \
      -d "{\"action\":\"DONE\",\"memoryId\":\"${MEM}\",\"identifier\":\"${DONE_ID}\"}" "$DEBUG_URL" || true)"
    note "debug-action status=$DEBUG_CODE body=$(cat "$DATA/run/debug-action.json" 2>/dev/null || true)"
  fi
  EVENTS2="$(curl -s -H "Authorization: Bearer ${TOKEN}" "${ENDPOINT}/events" || true)"
  note "events-after-debug $EVENTS2"
  SPARKY_DATA_DIR="$DATA" "$BIN" --simulate-action OPEN --memory "$MEM" --identifier "$DONE_ID" >"$DATA/run/open-action.out" 2>"$DATA/run/open-action.err" || true
  note "simulate OPEN:"
  cat "$DATA/run/open-action.out" | tee -a "$REPORT" || true
  sleep 2
  grep -E "open-url" "$HOME_LOG" "$DATA/run/electron.log" 2>/dev/null | tee -a "$REPORT" || true
  kill_pid "$SERVE_PID"
  SERVE_PID=""
  kill_pid "$CORE_PID"
  CORE_PID=""
  run_notifier --remove-prefix "rem:${MEM}"
fi

if [[ "$AUTHORIZED" == "1" ]]; then
  note "=== survival delivery T+120 with processes killed ==="
  NONCE="deliver-after-kill-$(date +%s)"
  SPARKY_DATA_DIR="$DATA" "$BIN" --arm-in 120 --memory "$MEM" --title "Sparky spike" --body "$NONCE" >"$DATA/run/arm-survival.out" 2>"$DATA/run/arm-survival.err" || true
  note "armed survival:"
  cat "$DATA/run/arm-survival.out" | tee -a "$REPORT" || true
  kill_pid "$CORE_PID"
  CORE_PID=""
  kill_pid "$ELECTRON_PID"
  pkill -9 -f "$DIST/stubcore" 2>/dev/null || true
  pkill -9 -f "$APP/Contents/MacOS/SparkyNotifier" 2>/dev/null || true
  pkill -9 -f "$SPIKE_APP/Contents/MacOS/Electron" 2>/dev/null || true
  note "pkill -9 of stub core, SparkyNotifier and SparkySpike electron (path-scoped, not every Electron)"
  note "sleeping 135s for OS delivery"
  EB_BIN=""
  if [[ -x "$DESKTOP/node_modules/.bin/electron-builder" ]]; then
    EB_BIN="$DESKTOP/node_modules/.bin/electron-builder"
  fi
  if [[ -n "$EB_BIN" ]]; then
    (cd "$ROOT/spikes/notifier/electron" && "$EB_BIN" --mac dir --config electron-builder.yml >"$EVIDENCE/electron-builder.log" 2>&1) &
    EB_PID=$!
    note "electron-builder pid=$EB_PID"
  else
    note "electron-builder bin not found; unsigned copy above is the identity-null evidence"
    EB_PID=""
  fi
  sleep 135
  if [[ -n "${EB_PID:-}" ]]; then
    if kill -0 "$EB_PID" 2>/dev/null; then
      kill "$EB_PID" 2>/dev/null || true
      note "electron-builder still running after the wait; stopped it"
    else
      wait "$EB_PID" 2>/dev/null || true
      note "electron-builder finished; tail:"
      tail -n 30 "$EVIDENCE/electron-builder.log" | tee -a "$REPORT" || true
    fi
  fi
  SPARKY_DATA_DIR="$DATA" "$BIN" --dump-delivered >"$DATA/run/delivered.out" 2>"$DATA/run/delivered.err" || true
  note "delivered after kill:"
  cat "$DATA/run/delivered.out" | tee -a "$REPORT" || true
  if grep -q "$NONCE" "$DATA/run/delivered.out"; then
    note "SURVIVAL PASS nonce delivered with notifier, core and electron dead"
  else
    note "SURVIVAL FAIL nonce not in delivered list"
  fi
  SPARKY_DATA_DIR="$DATA" "$BIN" --remove-prefix "rem:${MEM}" >/dev/null 2>&1 || true
fi

note "=== end ==="
