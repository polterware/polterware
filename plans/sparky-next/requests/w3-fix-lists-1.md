# Packaged renderer assets return 404 inside ASAR

Owner: W3-SECURITY / Main / packaging. Blocks standard packaged W3 E2E verification.

The unchanged `prepare-e2e.sh` succeeds, but all five selected Memories, Map and agent-write cases fail before onboarding on a blank window. A fresh isolated launch and reload reports two asset 404s. The JS/CSS files are present in `app.asar/out/renderer/assets` (verified with `@electron/asar.listPackage`).

`apps/sparky-desktop/src/main/renderer-protocol.ts` opens `root` and nested paths using `fs/promises.open` with `O_DIRECTORY`. The configured renderer root is inside `app.asar`; those archive entries are not physical directories. Its catch returns 404. `apps/sparky-desktop/electron-builder.yml` enables `asar: true`. This handler was introduced after the original W3-E2E packaged evidence.

Please make the secure renderer-asset path compatible with the packaged archive (or deliberately ship renderer assets unpacked), keeping the traversal, symlink, hardlink, type and size restrictions. Add a regression against the actual packaged layout and rerun the default `prepare-e2e.sh` harness. Main, packaging and shared configuration are outside w3-fix-lists ownership and remain unchanged.

Evidence: `apps/sparky-desktop/test-results/w3-fix-lists/results.json` and failure traces under `w3-fix-lists/artifacts`. A diagnostic unsigned directory package uses CLI `--config.asar=false` with output `test-results/build-unpacked`; its list/spec/FPS results are recorded separately in `status/w3-fix-lists.md`. It does not establish acceptance of the standard ASAR package.
