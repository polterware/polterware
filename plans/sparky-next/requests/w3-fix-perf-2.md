# Stabilize the Memories functional E2E ordering and dialog focus

Owner: W3-E2E / shared Dialog owner for any focus defect established during follow-up.

The unchanged functional suite was run in the diagnostic unpacked package with `SPARKY_E2E_RECHECK_GAPS=1`. Both tests fail; this is not a functional acceptance pass. The unchanged performance spec passes twice and is unrelated to the proposed functional-harness changes.

1. The seed gives all rows equal due dates/sortOrder and the list is loaded in descending updated order. `Performance memory 00119` is the first row; scrolling to the bottom correctly shows `Performance memory 00000`. The original renderer at branch base `2a92c5c` fails identically. Minimal exact assertion delta for the currently observed default order:

```diff
- await expect(page.getByRole("row", { name: "Performance memory 00119", exact: true })).toBeVisible();
+ await expect(page.getByRole("row", { name: "Performance memory 00000", exact: true })).toBeVisible();
```

Alternatively explicitly choose Title ascending through the `Sort memories` button and its Title menu item before asserting the last row, and retain the 00119 expectation. Make the ordering part of the fixture rather than an implicit assumption.

2. The final renderer reaches deletion confirmation, but immediate focus/Enter activation closes the dialog without showing `Deletion queued:`. A diagnostic copy waits for the named dialog to be visible, waits 250 ms for focus settlement, then focuses Delete and presses Enter; it passes the full keyboard create/edit/pin/complete/undo/delete/persistence scenario. This establishes a readiness/timing dependency, not the precise cause. Capture the actual focused element at keydown/keyup and synchronize the test with dialog readiness/focus; route any late autofocus overriding explicit user focus to the shared Dialog owner. The 250 ms wait is isolated diagnostic evidence, not a proposed permanent sleep or production change.

Controlled original-renderer comparison also fails the keyboard scenario earlier at `row.focus()` after rename: it retains the previous row label. W3-FIX-PERF's id-keyed body fixes that stale virtual content, with a unit regression for refreshed row content; the original run therefore cannot independently establish the deletion timing failure.

Evidence:

- Final unchanged functional report: `apps/sparky-desktop/test-results/w3-fix-perf-functional/results.json`, **0 pass / 2 fail**, with failure Playwright traces.
- Controlled original renderer: `apps/sparky-desktop/test-results/w3-fix-perf-functional-baseline/results.json`, **0 pass / 2 fail**. This uses Main/core/native resources from the final diagnostic package and the renderer extracted from the initial standard build's ASAR into a separate ignored unpacked copy; no tracked source changes.
- Adjusted diagnostic copy: `apps/sparky-desktop/test-results/functional-diagnostic/results.json`, **2 pass**, with only bottom-row expectation and dialog focus-settlement changes described above. The tracked E2E specs are untouched; diagnostic passes do not replace the unchanged-suite failures.

`apps/sparky-desktop/tests/e2e/memories.spec.ts` and shared `dialog*` are outside this workstream's ownership. No performance budget, measurement loop or assertion change is requested.
