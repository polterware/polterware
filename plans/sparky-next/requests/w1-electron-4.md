# W1-ELECTRON request 4: updater dependency and asset integration

Owner: W0-SCAFFOLD / integrator (lockfile), W1-ASSETS (icons).

`electron-updater` is not in Sparky's dependency set. Adding it requires the root `pnpm-lock.yaml`, outside Owns. Please add `"electron-updater": "^6.8.9"` to Sparky dependencies using the same version as Polterware (`apps/desktop/package.json` declares ^6.8.9) and update the lockfile. Keep update checks/downloads/installation disabled until W3-PACKAGING provides the approved feed and signing. W1-ELECTRON leaves honest disabled status and stops this dependency sub-task.

Builder now references W1-ASSETS-owned `resources/icon.icns`, `resources/app-icons`, and `resources/logo`, and declares macOS 15 minimum plus the microphone usage text. Those resources do not exist in this scaffold. Merge W1-ASSETS before packaging. The ported artifact test is explicitly skipped while icon.icns is absent; builder assertions still run. W1-NATIVE-NOTIFY's exact addon path/build command must be supplied separately (request 3).
