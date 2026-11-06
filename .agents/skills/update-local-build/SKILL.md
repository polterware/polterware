---
name: update-local-build
description: >-
  Rebuild the Polterware desktop app of this repo from the
  current working tree and install it over the copy in /Applications on the
  user's Mac, so it can be tested before a real release. Use when the user asks
  to update, refresh, rebuild or reinstall the local build / installed app, or
  to test today's version of an installed app.
---

# Update local build

Installs an unreleased, ad-hoc signed build of Polterware over its installed copy. Dispatch and Vitals build in their own repositories. Running it is an explicit request, so building is allowed (see AGENTS.md). Do not open the app or start previews afterwards unless asked.

## Steps

1. **Pick the version.** `tooling/release/bundle.sh` refuses to reuse an output folder, and the workspace version in `Cargo.toml` is the real release version, so use `RELEASE_VERSION=<n>-local`. Take the highest `0.1.N-local` under `target/release-bundle/` and use N+1; with none, use the installed version's patch + 1.
2. **Build** (arm64 macOS only; takes minutes, so redirect the log to the scratchpad):
   ```bash
   RELEASE_VERSION=0.1.N-local bash tooling/release/bundle.sh > <scratchpad>/bundle.log 2>&1; echo $?
   ```
   On failure, read the tail of the log and fix or report; never install a failed build. The tail ends with `Built .../Polterware.app` on success. The build includes uncommitted changes; do not commit or stash for it.
3. **Quit the running app** gracefully: `osascript -e 'tell application "Polterware" to quit'`, wait a few seconds, then `pgrep -f '/Applications/Polterware.app'` must be empty. If it is still running, ask before killing it.
4. **Replace.** Look at the target first (`PlistBuddy -c 'Print CFBundleShortVersionString' /Applications/Polterware.app/Contents/Info.plist`). Move the old app to `~/.Trash/Polterware-<old version>.app` (never `rm`), then:
   ```bash
   ditto target/release-bundle/<version>/Polterware.app /Applications/Polterware.app
   xattr -dr com.apple.quarantine /Applications/Polterware.app
   ```
5. **Unregister the build folder's copy**, so the installed app stays the only registered copy:
   ```bash
   LSR=/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister
   for app in target/release-bundle/*/Polterware.app; do "$LSR" -u "$PWD/$app"; done
   "$LSR" -f /Applications/Polterware.app
   ```
6. **Verify** the installed version reads back as the new one.

## Report (in Portuguese)

Say: the version installed, which commit the build is based on plus that uncommitted changes were included, that the old app is in the Trash, and that the app was not opened or tested.

## Notes

- Local builds are ad-hoc signed; Gatekeeper rejects them, which is expected.
- Older `target/release-bundle/*-local` folders can be left alone; mention them only if disk usage matters.
