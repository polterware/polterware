# Integrate the root Sparky release command and ignore local artifacts

Owner: W0-SCAFFOLD / integrator. W3-PACKAGING does not edit shared root files.

The release entrypoint builds arm64 Go + pinned Electron 44.4.5 native notifications + production Electron/Vite output, signs ad-hoc before creating DMG/ZIP, verifies the bundle and writes SHA256SUMS. It never publishes. Until this delta is applied, run `pnpm --filter @polterware/sparky-desktop exec bash scripts/dist-mac.sh`.

Apply this exact diff; no package manifest or lockfile regeneration is needed:

```diff
--- a/package.json
+++ b/package.json
@@
     "test:e2e:sparky": "pnpm --filter @polterware/sparky-desktop test:e2e",
+    "dist:sparky:mac": "pnpm --filter @polterware/sparky-desktop exec bash scripts/dist-mac.sh",
--- a/.gitignore
+++ b/.gitignore
@@
 apps/sparky-desktop/out/
+apps/sparky-desktop/release/
 apps/sparky-desktop/test-results/
```

The release directory is excluded locally through this worktree's Git exclude until integration. The desktop manifest already pins Electron exactly; w3-e2e-8 is satisfied in the starting tree. `.github/workflows/sparky-release.yml` is explicitly owned by W3-PACKAGING for this task and is committed directly; no other workflow is changed.
