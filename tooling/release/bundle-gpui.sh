#!/usr/bin/env bash
# Builds an ad-hoc Polterware GPUI app, ZIP and DMG, following Dispatch's
# tooling/release/bundle.sh and Electron's tooling/release/dist-mac.sh.
# Usage: bash tooling/release/bundle-gpui.sh [--existing-core]
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
[[ $# -le 1 && ( $# -eq 0 || "$1" == --existing-core ) ]] || { echo 'Usage: bundle-gpui.sh [--existing-core]' >&2; exit 1; }
[[ "$(uname -s)" == Darwin && "$(uname -m)" == arm64 ]] || { echo 'Desktop releases require a macOS arm64 host.' >&2; exit 1; }
unset CSC_LINK CSC_KEY_PASSWORD CSC_NAME CSC_KEYCHAIN CSC_INSTALLER_LINK CSC_INSTALLER_KEY_PASSWORD
export CSC_IDENTITY_AUTO_DISCOVERY=false
export SOURCE_DATE_EPOCH="${SOURCE_DATE_EPOCH:-$(git -C "$ROOT" log -1 --format=%ct)}"
VERSION="${RELEASE_VERSION:-$(node -p 'require(process.argv[1]).version' "$ROOT/apps/polterware-desktop/package.json")}"
PRODUCT="$(node "$ROOT/tooling/release/apps.cjs" polterware product)"
OUTPUT="$ROOT/desktop-gpui/target/release-bundle/$VERSION"
APP="$OUTPUT/$PRODUCT.app"
[[ ! -e "$OUTPUT" ]] || { echo "Release output already exists: $OUTPUT" >&2; exit 1; }
[[ "$SOURCE_DATE_EPOCH" =~ ^[0-9]+$ ]] || { echo 'Invalid SOURCE_DATE_EPOCH' >&2; exit 1; }
node "$ROOT/tooling/release/gpui-plist.cjs" "$VERSION" "$SOURCE_DATE_EPOCH" >/dev/null
if [[ "${1:-}" != --existing-core ]]; then
  mkdir -p "$ROOT/dist"
  (cd "$ROOT/backend" && CGO_ENABLED=0 GOOS=darwin GOARCH=arm64 go build -mod=readonly -trimpath -buildvcs=false -ldflags='-s -w -buildid=' -o "$ROOT/dist/polterware" ./runtimes/cli)
  touch -t "$(date -r "$SOURCE_DATE_EPOCH" +%Y%m%d%H%M.%S)" "$ROOT/dist/polterware"
fi
[[ -f "$ROOT/dist/polterware" ]] || { echo 'Core binary is missing: dist/polterware' >&2; exit 1; }
[[ "$(lipo -archs "$ROOT/dist/polterware")" == arm64 ]] || { echo 'Core must be arm64.' >&2; exit 1; }
SPARKLE_VERSION="$(node -p 'require(process.argv[1]).version' "$ROOT/tooling/updater/config.cjs")"
SPARKLE_DIGEST="$(node -p 'require(process.argv[1]).archiveSha256' "$ROOT/tooling/updater/config.cjs")"
ARCHIVE="$ROOT/tooling/updater/vendor/Sparkle-$SPARKLE_VERSION.tar.xz"
[[ -f "$ARCHIVE" ]] || { echo "Sparkle archive missing: $ARCHIVE" >&2; exit 1; }
printf '%s  %s\n' "$SPARKLE_DIGEST" "$ARCHIVE" | shasum -a 256 -c -
STAGING="$(mktemp -d "${TMPDIR:-/tmp}/polterware-sparkle.XXXXXXXX")"
trap 'rm -rf "$STAGING"' EXIT
tar -xJf "$ARCHIVE" -C "$STAGING"
(cd "$ROOT/desktop-gpui" && cargo build --release --locked)
mkdir -p "$APP/Contents/"{MacOS,Resources,Frameworks}
cp "$ROOT/desktop-gpui/target/release/polterware-gpui" "$APP/Contents/MacOS/$PRODUCT"
cp -p "$ROOT/dist/polterware" "$APP/Contents/Resources/polterware"
node "$ROOT/tooling/release/gpui-plist.cjs" "$VERSION" "$SOURCE_DATE_EPOCH" > "$APP/Contents/Info.plist"
plutil -lint "$APP/Contents/Info.plist"
xcrun actool "$ROOT/apps/polterware-desktop/resources/polterware.icon" --compile "$APP/Contents/Resources" --platform macosx --minimum-deployment-target 15.0 --app-icon polterware --output-partial-info-plist "$OUTPUT/icon-info.plist" >/dev/null
ditto "$STAGING/Sparkle.framework" "$APP/Contents/Frameworks/Sparkle.framework"
cp "$STAGING/LICENSE" "$APP/Contents/Resources/Sparkle-LICENSE"
FRAMEWORK="$APP/Contents/Frameworks/Sparkle.framework"
for path in Versions/B/XPCServices/Downloader.xpc Versions/B/XPCServices/Installer.xpc Versions/B/Autoupdate Versions/B/Updater.app ''; do
  codesign --force --deep --sign - --timestamp=none "$FRAMEWORK/$path"
done
# Signing rewrites the core; preserve its mtime for the core build handshake.
CORE="$APP/Contents/Resources/polterware"
CORE_MTIME="$(stat -f %m "$CORE")"
codesign --force --sign - --timestamp=none "$CORE"
codesign --force --sign - --timestamp=none "$APP"
touch -t "$(date -r "$CORE_MTIME" +%Y%m%d%H%M.%S)" "$CORE"
codesign --verify --deep --strict "$APP"
ARTIFACT="${PRODUCT// /-}-$VERSION-arm64"
(cd "$OUTPUT" && ditto -c -k --sequesterRsrc --keepParent "$PRODUCT.app" "$ARTIFACT.zip")
hdiutil create -volname "$PRODUCT" -srcfolder "$APP" -ov -format UDZO "$OUTPUT/$ARTIFACT.dmg" >/dev/null
(cd "$OUTPUT" && shasum -a 256 "$ARTIFACT.dmg" "$ARTIFACT.zip" > SHA256SUMS)
echo "Built $APP"
