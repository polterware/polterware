#!/usr/bin/env bash
set -euo pipefail
UPDATER="$(cd "$(dirname "$0")" && pwd)"
# shellcheck source=sparkle.env
source "$UPDATER/sparkle.env"
mkdir -p "$UPDATER/vendor"
ARCHIVE="$UPDATER/vendor/Sparkle-$SPARKLE_VERSION.tar.xz"
if [[ ! -f "$ARCHIVE" ]]; then
  curl --fail --location --retry 3 "https://github.com/sparkle-project/Sparkle/releases/download/$SPARKLE_VERSION/Sparkle-$SPARKLE_VERSION.tar.xz" -o "$ARCHIVE.tmp"
  mv "$ARCHIVE.tmp" "$ARCHIVE"
fi
printf '%s  %s\n' "$SPARKLE_SHA256" "$ARCHIVE" | shasum -a 256 -c -
# Always extract from the verified archive rather than trusting a previously unpacked copy.
tar -xJf "$ARCHIVE" -C "$UPDATER/vendor"
