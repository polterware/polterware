#!/usr/bin/env bash
# Builds an ad-hoc Polterware.app for macOS arm64 with its ZIP, DMG, and checksums, as
# Dispatch's tooling/release/bundle.sh does. The bundle holds the GPUI app, the `polterware`
# core in Resources, the app icon, and Sparkle. Signing is ad-hoc: no certificate,
# entitlements, hardened runtime, or notarization. SOURCE_DATE_EPOCH (the last commit's time
# by default) is the build number.
#   bash tooling/release/bundle.sh [--existing-core]
# The version is app/Cargo.toml's (the workspace's), unless RELEASE_VERSION is set.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
[[ $# -le 1 && ( $# -eq 0 || "$1" == --existing-core ) ]] || { echo 'Usage: bundle.sh [--existing-core]' >&2; exit 1; }
[[ "$(uname -s)" == Darwin && "$(uname -m)" == arm64 ]] || { echo 'Desktop releases require a macOS arm64 host.' >&2; exit 1; }
export SOURCE_DATE_EPOCH="${SOURCE_DATE_EPOCH:-$(git -C "$ROOT" log -1 --format=%ct)}"
[[ "$SOURCE_DATE_EPOCH" =~ ^[0-9]+$ ]] || { echo 'Invalid SOURCE_DATE_EPOCH' >&2; exit 1; }
VERSION="${RELEASE_VERSION:-$(sed -n '/^\[workspace.package\]/,/^\[/s/^version = "\(.*\)"/\1/p' "$ROOT/Cargo.toml")}"
[[ "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z.-]+)?$ ]] || { echo "Invalid release version: $VERSION" >&2; exit 1; }
# shellcheck source=../updater/sparkle.env
source "$ROOT/tooling/updater/sparkle.env"
PRODUCT=Polterware
OUTPUT="$ROOT/target/release-bundle/$VERSION"
APP="$OUTPUT/$PRODUCT.app"
[[ ! -e "$OUTPUT" ]] || { echo "Release output already exists: $OUTPUT" >&2; exit 1; }
if [[ "${1:-}" != --existing-core ]]; then
  mkdir -p "$ROOT/dist"
  (cd "$ROOT/backend" && CGO_ENABLED=0 GOOS=darwin GOARCH=arm64 go build -mod=readonly -trimpath -buildvcs=false -ldflags='-s -w -buildid=' -o "$ROOT/dist/polterware" ./runtimes/cli)
  touch -t "$(date -r "$SOURCE_DATE_EPOCH" +%Y%m%d%H%M.%S)" "$ROOT/dist/polterware"
fi
[[ -f "$ROOT/dist/polterware" ]] || { echo 'Core binary is missing: dist/polterware' >&2; exit 1; }
[[ "$(lipo -archs "$ROOT/dist/polterware")" == arm64 ]] || { echo 'Core must be arm64.' >&2; exit 1; }
ARCHIVE="$ROOT/tooling/updater/vendor/Sparkle-$SPARKLE_VERSION.tar.xz"
[[ -f "$ARCHIVE" ]] || bash "$ROOT/tooling/updater/fetch.sh"
printf '%s  %s\n' "$SPARKLE_SHA256" "$ARCHIVE" | shasum -a 256 -c -
STAGING="$(mktemp -d "${TMPDIR:-/tmp}/polterware-sparkle.XXXXXXXX")"
trap 'rm -rf "$STAGING"' EXIT
tar -xJf "$ARCHIVE" -C "$STAGING"
(cd "$ROOT" && cargo build --release --locked -p polterware-gpui)
mkdir -p "$APP/Contents/"{MacOS,Resources,Frameworks}
cp "$ROOT/target/release/polterware-gpui" "$APP/Contents/MacOS/$PRODUCT"
cp -p "$ROOT/dist/polterware" "$APP/Contents/Resources/polterware"
cat > "$APP/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleIdentifier</key><string>com.polterware.polterware</string>
  <key>CFBundleName</key><string>$PRODUCT</string>
  <key>CFBundleDisplayName</key><string>$PRODUCT</string>
  <key>CFBundleExecutable</key><string>$PRODUCT</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>CFBundleShortVersionString</key><string>$VERSION</string>
  <key>CFBundleVersion</key><string>$SOURCE_DATE_EPOCH</string>
  <key>CFBundleIconName</key><string>polterware</string>
  <key>CFBundleIconFile</key><string>polterware</string>
  <key>LSMinimumSystemVersion</key><string>15.0</string>
  <key>LSApplicationCategoryType</key><string>public.app-category.developer-tools</string>
  <key>NSHighResolutionCapable</key><true/>
  <key>SUFeedURL</key><string>$SPARKLE_FEED_URL</string>
  <key>SUPublicEDKey</key><string>$SPARKLE_PUBLIC_KEY</string>
  <key>SUEnableAutomaticChecks</key><true/>
  <key>SUScheduledCheckInterval</key><integer>86400</integer>
  <key>SUAutomaticallyUpdate</key><false/>
  <key>SUShowReleaseNotes</key><false/>
</dict>
</plist>
PLIST
plutil -lint "$APP/Contents/Info.plist"
xcrun actool "$ROOT/app/resources/polterware.icon" --compile "$APP/Contents/Resources" --platform macosx --minimum-deployment-target 15.0 --app-icon polterware --output-partial-info-plist "$OUTPUT/icon-info.plist" >/dev/null
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
ARTIFACT="$PRODUCT-$VERSION-arm64"
(cd "$OUTPUT" && ditto -c -k --sequesterRsrc --keepParent "$PRODUCT.app" "$ARTIFACT.zip")
hdiutil create -volname "$PRODUCT" -srcfolder "$APP" -ov -format UDZO "$OUTPUT/$ARTIFACT.dmg" >/dev/null
(cd "$OUTPUT" && shasum -a 256 "$ARTIFACT.dmg" "$ARTIFACT.zip" > SHA256SUMS)
echo "Built $APP"
