#!/usr/bin/env bash
set -euo pipefail
SPIKE="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(cd "$SPIKE/../.." && pwd)"
cd "$SPIKE"
mkdir -p dist/headers run/napi/run run/swift/run
ELECTRON_VERSION="$(node -p "require('$ROOT/apps/sparky-desktop/node_modules/electron/package.json').version")"
[[ "$ELECTRON_VERSION" == 44.4.5 ]] || { echo "Expected Electron 44.4.5, got $ELECTRON_VERSION"; exit 1; }
ELECTRON_BIN="$(node -p "require('$ROOT/apps/sparky-desktop/node_modules/electron')")"
ELECTRON_APP="$(cd "$(dirname "$ELECTRON_BIN")/../.." && pwd)"
if [[ ! -f dist/headers/include/node/node_api.h ]]; then
  curl --fail --location "https://electronjs.org/headers/v${ELECTRON_VERSION}/node-v${ELECTRON_VERSION}-headers.tar.gz" -o dist/headers.tar.gz
  tar -xzf dist/headers.tar.gz --strip-components=1 -C dist/headers
fi
xcrun clang++ -std=c++17 -fobjc-arc -fblocks -mmacosx-version-min=15.0 -DNAPI_VERSION=8 \
  -I dist/headers/include/node -I node_modules/node-addon-api -bundle -undefined dynamic_lookup \
  -framework AppKit -framework UserNotifications native/notify.mm -o dist/notify.node
xcrun swiftc -swift-version 5 -O -emit-library -target arm64-apple-macosx15.0 -module-name Notify \
  native/Notify.swift -o dist/libNotify.dylib
(cd stubcore && GOWORK=off go build -o ../dist/stubcore .)
for FORM in napi swift; do
  APP="$SPIKE/dist/$FORM/Sparky.app"
  rm -rf "$APP"
  mkdir -p "$(dirname "$APP")"
  cp -R "$ELECTRON_APP" "$APP"
  PLIST="$APP/Contents/Info.plist"
  /usr/libexec/PlistBuddy -c 'Set :CFBundleIdentifier com.polterware.sparky.spike2' "$PLIST"
  /usr/libexec/PlistBuddy -c 'Set :CFBundleName Sparky' "$PLIST"
  /usr/libexec/PlistBuddy -c 'Set :CFBundleDisplayName Sparky' "$PLIST"
  /usr/libexec/PlistBuddy -c 'Set :LSMinimumSystemVersion 15.0' "$PLIST"
  # Accessory app: no Dock icon/bounce during hidden relaunch. Foreground behavior is a W1 concern.
  /usr/libexec/PlistBuddy -c 'Add :LSUIElement bool true' "$PLIST"
  APP_CODE="$APP/Contents/Resources/app"
  mkdir -p "$APP_CODE/native"
  cp electron/*.cjs electron/index.html "$APP_CODE/"
  printf '{"name":"sparky-native-notify-spike","version":"0.0.1","main":"main.cjs"}\n' > "$APP_CODE/package.json"
  node -e 'const fs=require("fs"); fs.writeFileSync(process.argv[1], JSON.stringify({form:process.argv[2],data:process.argv[3]}))' \
    "$APP_CODE/spike-config.json" "$FORM" "$SPIKE/run/$FORM"
  if [[ "$FORM" == napi ]]; then
    cp dist/notify.node "$APP_CODE/native/"
  else
    cp dist/libNotify.dylib "$APP_CODE/native/"
    mkdir -p "$APP_CODE/node_modules"
    cp -RL node_modules/koffi "$APP_CODE/node_modules/koffi"
    # Ship only the host platform's FFI prebuild, not Linux/Windows binaries or build sources.
    node -e 'const fs=require("fs"),p=require("path"),root=process.argv[1]; for(const x of fs.readdirSync(root)) if(x!=="darwin_arm64") fs.rmSync(p.join(root,x),{recursive:true,force:true})' "$APP_CODE/node_modules/koffi/build/koffi"
  fi
  cp dist/stubcore "$APP/Contents/Resources/"
  codesign --force --deep --sign - --identifier com.polterware.sparky.spike2 "$APP"
  codesign --verify --deep --strict "$APP"
  echo "Built $FORM: $APP"
done
