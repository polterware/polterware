# Memory collection keeps the old title after a successful detail save

Owner: Memories feature owner / integrator. Product source is outside W3-FIX-ASAR ownership.

## Symptom and reproduction

On the standard ASAR package produced by unchanged `prepare-e2e.sh`, the now-unquarantined keyboard scenario successfully opens New memory with Enter and creates it with Space. The detail input accepts `Edited memory`; Meta+S completes with `Live · Saved`. An authenticated preload read of the real core confirms the persisted title is `Edited memory`.

After Meta+J closes the detail panel, the same memory's collection row still has accessible name and visible title `Keyboard memory`. It never exposes an `Edited memory` row during the 10-second locator timeout. This blocks the subsequent pin, complete, undo and delete keyboard steps. It is not the resolved native-button interception, empty virtualization, or ASAR asset issue. No new quarantine was added: the case remains active and fails.

Reproduce from repository root after preparing the standard package:

```bash
SPARKY_E2E_EXECUTABLE="$PWD/apps/sparky-desktop/test-results/build/mac-arm64/Sparky.app/Contents/MacOS/Sparky" \
  SPARKY_E2E_RUN=w3-fix-asar-keyboard \
  pnpm --filter @polterware/sparky-desktop exec playwright test e2e/memories.spec.ts -g 'keyboard create'
```

## Evidence

- Repeated failures: full `w3-fix-asar-functional` run, targeted `w3-fix-asar-recheck`, `w3-fix-asar-memories`, and `w3-fix-asar-keyboard` runs.
- Latest isolated report: `apps/sparky-desktop/test-results/w3-fix-asar-keyboard/results.json`.
- Trace: `apps/sparky-desktop/test-results/w3-fix-asar-keyboard/artifacts/e2e-memories-keyboard-crea-afed5-in-complete-undo-and-delete/trace.zip`.
- The `saved-title-and-collection` attachment records `persistedTitles: ["Edited memory"]` while the row for the same memory is named `Keyboard memory` and displays `Keyboard memoryMemoriesPin`. These are synthetic titles from a disposable database, not personal data.
- Its `error-context.md` records the stale row after panel closure. The initial full run had the same symptom before the additional persisted-title assertion and diagnostic attachment.

## Requested fix and acceptance

Investigate collection cache/change invalidation across detail saves and panel closure in `packages/features/sparky-memories/src`, including the store and route/detail composition. The evidence establishes the divergence; no specific internal cause is claimed.

Add a product regression for a detail edit reaching the real collection store, fix the invalidation, then run the unchanged keyboard actions through pin, complete, undo and delete on the standard ASAR package. Keep the real-core persistence check, row-title assertion, and all security restrictions. Do not replace the user edit with a bridge mutation or weaken the expected title.
