#!/usr/bin/env bash
set -euo pipefail
DESKTOP="$(cd "$(dirname "$0")/.." && pwd)"
ROOT="$(cd "$DESKTOP/../.." && pwd)"
[[ "$(uname -s)" == Darwin && "$(uname -m)" == arm64 ]] || { echo 'Sparky releases require a macOS arm64 host.' >&2; exit 1; }
# Never discover or load signing certificates, even if CI/user environment supplies them.
unset CSC_LINK CSC_KEY_PASSWORD CSC_NAME CSC_KEYCHAIN CSC_INSTALLER_LINK CSC_INSTALLER_KEY_PASSWORD
export CSC_IDENTITY_AUTO_DISCOVERY=false
export NODE_ENV=production
export SPARKY_NATIVE_ARCH=arm64
export SOURCE_DATE_EPOCH="${SOURCE_DATE_EPOCH:-$(git -C "$ROOT" log -1 --format=%ct)}"
VERSION="${SPARKY_RELEASE_VERSION:-$(node -p "require(process.argv[1]).version" "$DESKTOP/package.json")}"
node -e 'if (!/^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?$/.test(process.argv[1])) throw Error("Invalid release version")' "$VERSION"
[[ "$(node -p "require(process.argv[1]).version" "$DESKTOP/node_modules/electron/package.json")" == 44.4.5 ]] || { echo 'Install the frozen lockfile with Electron 44.4.5.' >&2; exit 1; }
OUTPUT="$DESKTOP/release/$VERSION"
[[ ! -e "$OUTPUT" ]] || { echo "Release output already exists: $OUTPUT (move it before rebuilding)." >&2; exit 1; }
mkdir -p "$ROOT/dist"
(cd "$ROOT/sparky/backend" && CGO_ENABLED=0 GOOS=darwin GOARCH=arm64 go build -mod=readonly -trimpath -buildvcs=false -ldflags='-s -w -buildid=' -o "$ROOT/dist/sparky" ./runtimes/cli)
# The supervisor uses the binary's mtime in its build handshake. Use the commit time.
node -e 'require("node:fs").utimesSync(process.argv[1], Number(process.env.SOURCE_DATE_EPOCH), Number(process.env.SOURCE_DATE_EPOCH))' "$ROOT/dist/sparky"
bash "$DESKTOP/native/scripts/build.sh"
pnpm --dir "$DESKTOP" exec electron-vite build --mode production
pnpm --dir "$DESKTOP" exec electron-builder --mac dmg zip --arm64 --publish never \
  "--config.extraMetadata.version=$VERSION" "--config.directories.output=$OUTPUT"
APP="$OUTPUT/mac-arm64/Sparky.app"
node "$DESKTOP/scripts/verify-bundle.cjs" "$APP" > "$OUTPUT/bundle-verification.txt"
cat "$OUTPUT/bundle-verification.txt"
# Rejection is expected: ad-hoc is not Developer ID signing/notarization.
set +e
spctl --assess --type execute --verbose=4 "$APP" > "$OUTPUT/gatekeeper.txt" 2>&1
ASSESSMENT=$?
set -e
printf '\nspctl exit: %s (ad-hoc rejection expected)\n' "$ASSESSMENT" >> "$OUTPUT/gatekeeper.txt"
cat "$OUTPUT/gatekeeper.txt"
(cd "$OUTPUT" && shasum -a 256 "Sparky-$VERSION-arm64.dmg" "Sparky-$VERSION-arm64.zip" > SHA256SUMS)
node - "$OUTPUT" "$VERSION" "$ROOT" <<'JS'
const fs = require('node:fs'), path = require('node:path'), crypto = require('node:crypto');
const { execFileSync } = require('node:child_process');
const directory = process.argv[2], version = process.argv[3], root = process.argv[4];
const hash = file => crypto.createHash('sha256').update(fs.readFileSync(file)).digest('hex');
const command = (name, args) => execFileSync(name, args, { encoding: 'utf8' }).trim();
const resources = path.join(directory, 'mac-arm64/Sparky.app/Contents/Resources');
const report = { version, arch: 'arm64', electron: '44.4.5', sourceDateEpoch: process.env.SOURCE_DATE_EPOCH,
  commit: command('git', ['-C', root, 'rev-parse', 'HEAD']),
  tools: { node: process.version, pnpm: command('pnpm', ['--version']), go: command('go', ['version']), sdk: command('xcrun', ['--show-sdk-version']) },
  lockSha256: hash(path.join(root, 'pnpm-lock.yaml')),
  nativeLockSha256: hash(path.join(root, 'apps/sparky-desktop/native/pnpm-lock.yaml')),
  payloadSha256: Object.fromEntries(['sparky', 'native/notify.node', 'app.asar'].map(file => [file, hash(path.join(resources, file))])),
  artifacts: ['dmg', 'zip'].map(ext => { const file = `Sparky-${version}-arm64.${ext}`; return { file, bytes: fs.statSync(path.join(directory, file)).size }; }) };
fs.writeFileSync(path.join(directory, 'build-report.json'), JSON.stringify(report, null, 2) + '\n');
console.log(JSON.stringify(report, null, 2));
JS
