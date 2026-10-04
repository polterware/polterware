# Packaged renderer assets return 404 inside ASAR

Owner: integrator / W3-SECURITY / desktop Main. Outside W3-FIX-SHELL ownership.

`bash apps/sparky-desktop/tests/scripts/prepare-e2e.sh` builds successfully, but the packaged renderer is blank. `rendererFileResponse` opens renderer directories using file descriptors and `O_DIRECTORY`, which does not work for the virtual directories inside `app.asar`. The renderer document request returns 404. The existing packaged Main/preload and core otherwise initialize.

Evidence: `apps/sparky-desktop/test-results/w3-fix-shell-red-isolated-app2/artifacts/` (blank-window failures), and `/tmp/w3-fix-shell-debug.cjs` / the local debug output (`Failed to load resource: ... 404`, preload available, packaged renderer URL inside `app.asar`). A separate temporary bundle identity avoids the single-instance collision with Erick's running `/Applications/Sparky.app`; that app was preserved.

The same compiled renderer and Main work with `electron-builder --config.asar=false`. W3-FIX-SHELL's final renderer regressions use that configuration, unsigned, with private per-test databases and the existing notification stub. This proves the frontend fixes, not the default ASAR package.

Requested minimal harness delta in `apps/sparky-desktop/tests/scripts/prepare-e2e.sh`, pending the packaging/security owner fixing the default ASAR asset path:

```diff
 CSC_IDENTITY_AUTO_DISCOVERY=false pnpm --dir "$DESKTOP" exec electron-builder --mac --dir --publish never \
-  --config.directories.output=test-results/build --config.mac.identity=null >&2
+  --config.directories.output=test-results/build --config.mac.identity=null --config.asar=false >&2
```

The production ASAR path still needs a security-preserving fix and an actual packaged asset-loading regression. Do not bypass the renderer allowlist or filesystem checks in product code. Main, builder configuration and the shared harness were not edited here.
