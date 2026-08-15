#!/bin/bash
set -euo pipefail

APP_DISPLAY_NAME="Polterware"
REPO="rckbrcls/polterware"
GITHUB_API="https://api.github.com/repos/${REPO}/releases"

usage() {
  cat << EOF
Usage: install.sh [--version <version>]

Options:
  --version <version>  Install a specific version (ex: 0.1.0)
  -h, --help           Show this help
EOF
}

VERSION=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --version)
      VERSION="$2"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown argument: $1"
      usage
      exit 1
      ;;
  esac
done

if [ "$(uname -s)" != "Darwin" ]; then
  echo "This installer only supports macOS."
  exit 1
fi

if ! command -v python3 >/dev/null 2>&1; then
  echo "Python 3 is required to parse GitHub API responses."
  exit 1
fi

TMP_DIR=$(mktemp -d)
cleanup() { rm -rf "$TMP_DIR"; }
trap cleanup EXIT

# bash 3.2 + set -u cannot expand empty arrays ("${arr[@]}").
# Branch on token instead of building optional curl args.
TOKEN="${GH_TOKEN:-${GITHUB_TOKEN:-}}"
curl_gh() {
  if [ -n "$TOKEN" ]; then
    curl -H "Authorization: Bearer ${TOKEN}" "$@"
  else
    curl "$@"
  fi
}

echo "Fetching release metadata..."
if [ -n "$VERSION" ]; then
  VERSION="${VERSION#v}"
  RELEASE_JSON=$(curl_gh -fsSL \
    -H "Accept: application/vnd.github+json" \
    -H "X-GitHub-Api-Version: 2022-11-28" \
    "${GITHUB_API}/tags/v${VERSION}") || {
    echo "Failed to fetch release v${VERSION} from GitHub."
    exit 1
  }
else
  # Prefer newest non-draft release that has desktop .dmg/.zip assets.
  # (Avoids older CLI-only releases that may still be "latest" by semver.)
  RELEASES_JSON=$(curl_gh -fsSL \
    -H "Accept: application/vnd.github+json" \
    -H "X-GitHub-Api-Version: 2022-11-28" \
    "${GITHUB_API}?per_page=30") || {
    echo "Failed to fetch releases from GitHub."
    echo "If the repository is private, authenticate first:"
    echo "  export GH_TOKEN=\$(gh auth token)  # needs repo read access"
    echo "  curl -fsSL https://rckbrcls.com/api/polterware/install | bash"
    exit 1
  }
  RELEASE_JSON=$(printf '%s' "$RELEASES_JSON" | python3 -c '
import json, sys
releases = json.load(sys.stdin)
for rel in releases:
    if rel.get("draft") or rel.get("prerelease"):
        continue
    assets = rel.get("assets") or []
    if any(a.get("name", "").lower().endswith((".dmg", ".zip")) for a in assets):
        print(json.dumps(rel))
        sys.exit(0)
print("ERROR: No published release with .dmg/.zip assets found.", file=sys.stderr)
sys.exit(3)
') || {
    echo "No desktop release with macOS assets found. Publish Release Desktop first."
    exit 1
  }
fi

ASSET_JSON=$(printf '%s' "$RELEASE_JSON" | python3 -c '
import json, sys
data = json.load(sys.stdin)
assets = [a for a in data.get("assets", []) if a.get("name", "").lower().endswith((".dmg", ".zip"))]
if not assets:
    print("ERROR: No .dmg/.zip assets found.", file=sys.stderr)
    sys.exit(3)

def score(name: str):
    n = name.lower()
    return (
        0 if n.endswith(".zip") else 1,
        0 if "arm64" in n or "universal" in n else 1,
        0 if "mac" in n else 1,
        name,
    )

assets.sort(key=lambda a: score(a.get("name", "")))
a = assets[0]
print(json.dumps({
    "name": a["name"],
    "url": a["browser_download_url"],
    "size": a.get("size"),
    "tag": data.get("tag_name"),
}))
')

ASSET_NAME=$(echo "$ASSET_JSON" | python3 -c 'import json,sys; print(json.load(sys.stdin)["name"])')
ASSET_URL=$(echo "$ASSET_JSON" | python3 -c 'import json,sys; print(json.load(sys.stdin)["url"])')
ASSET_SIZE=$(echo "$ASSET_JSON" | python3 -c 'import json,sys; print(json.load(sys.stdin).get("size") or "")')
TAG=$(echo "$ASSET_JSON" | python3 -c 'import json,sys; print(json.load(sys.stdin).get("tag") or "")')

DOWNLOAD_PATH="$TMP_DIR/$ASSET_NAME"
echo "Using release ${TAG:-unknown}"
echo "Downloading $ASSET_NAME..."
curl_gh -fL "$ASSET_URL" -o "$DOWNLOAD_PATH"

if [ -n "$ASSET_SIZE" ]; then
  DOWNLOADED_SIZE=$(stat -f%z "$DOWNLOAD_PATH")
  if [ "$DOWNLOADED_SIZE" -ne "$ASSET_SIZE" ]; then
    echo "Downloaded file size mismatch (expected $ASSET_SIZE, got $DOWNLOADED_SIZE)."
    exit 1
  fi
fi

EXTRACT_DIR="$TMP_DIR/extract"
mkdir -p "$EXTRACT_DIR"
APP_PATH=""

if [[ "$ASSET_NAME" == *.zip ]]; then
  echo "Extracting zip..."
  ditto -x -k "$DOWNLOAD_PATH" "$EXTRACT_DIR"
  APP_PATH=$(find "$EXTRACT_DIR" -maxdepth 3 -name "*.app" -print -quit || true)
elif [[ "$ASSET_NAME" == *.dmg ]]; then
  echo "Mounting dmg..."
  MOUNT_POINT="$TMP_DIR/mnt"
  mkdir -p "$MOUNT_POINT"
  hdiutil attach -nobrowse -readonly -mountpoint "$MOUNT_POINT" "$DOWNLOAD_PATH" >/dev/null
  cleanup_dmg() {
    hdiutil detach "$MOUNT_POINT" >/dev/null 2>&1 || true
    rm -rf "$TMP_DIR"
  }
  trap cleanup_dmg EXIT
  APP_PATH=$(find "$MOUNT_POINT" -maxdepth 2 -name "*.app" -print -quit || true)
fi

if [ -z "$APP_PATH" ] || [ ! -d "$APP_PATH" ]; then
  echo "No .app bundle found in the release asset."
  exit 1
fi

APP_BUNDLE_NAME=$(basename "$APP_PATH")
TARGET_DIR="/Applications"
if [ ! -w "$TARGET_DIR" ]; then
  TARGET_DIR="$HOME/Applications"
  mkdir -p "$TARGET_DIR"
fi
TARGET_PATH="$TARGET_DIR/$APP_BUNDLE_NAME"

if [ -d "$TARGET_PATH" ]; then
  echo "Removing existing $TARGET_PATH"
  rm -rf "$TARGET_PATH"
fi

echo "Installing to $TARGET_DIR"
ditto "$APP_PATH" "$TARGET_PATH"
if command -v xattr >/dev/null 2>&1; then
  xattr -dr com.apple.quarantine "$TARGET_PATH" 2>/dev/null || true
fi

echo "✅ ${APP_DISPLAY_NAME} installed at $TARGET_PATH"
echo "Open with: open \"$TARGET_PATH\""
