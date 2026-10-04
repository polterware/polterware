# W3-FIX-SHELL status

- Result: **partial overall; all four owned product fixes PASS**. The seven owned E2E cases pass in the unsigned directory package with ASAR disabled. The standard ASAR package remains blocked by an outside-owner renderer asset issue; see request 1. No pass is claimed for that configuration or for the unowned memories spec.
- Branch: `sparky/w3-fix-shell`, already based directly on `sparky-next` at `dd404d8c16bff995591af4eb75efadabfa1fb73e` when the task began. Implementation commits: `d4e39c4`, `3e7ecea`, `d299a0d`; this status and the final Mind acceptance selectors are included in the last verification commit.
- Contracts: frozen and unchanged. No Main, Go, memories, design-system, shared root, lockfile or build-output changes. The Map change is exactly one settings registration ID. Nothing pushed or tagged.

## Per-item acceptance

| Item | Result | Regression and E2E evidence |
| --- | --- | --- |
| Shell Enter/Space | PASS | `leaves native controls, editable ancestors and overlays out of collection shortcuts` failed before the fix and now passes. Native controls, interactive ARIA controls, editable ancestors and dialog/menu/listbox contexts are exempt; already-prevented events stay untouched. Existing collection shortcut parity still passes. E2E `shell: Enter and Space activate native buttons and Save schedule preserves memory status` passes for both keys on Commander/New memory, Enter on the collection creation button, and Space on a live schedule save while the selected memory stays active. |
| Mind editor routing | PASS | `composes the registered Mind editor with its explicit mindId memory collection` failed before the fix and now passes. The registered Mind route renders its editor plus the registry-provided memory collection; `mindId` remains explicit and required. Name/color/icon, Move to, breadcrumb, child Minds and delete remain available. E2E `create, rename, and delete a Mind with cascade confirmation` passes from a card and the sidebar, renames the parent, reaches its memory collection, creates a child and verifies one child's memory moves to Inbox on cascade delete. The matching fixme is removed. |
| Duplicate Map settings ID | PASS | E2E `settings: Map Tiles and App Map select distinct panels` failed before the fix at Address search and now passes. IDs are `map-tiles` and `settings-map`; both actual panels render and only the selected rail entry has `aria-current=page`. |
| Calendar slot creation | PASS | Baseline E2E created the memory but failed to open details: the alert showed undeclared `expectedRev` rejected by `SetScheduleRequest`. Calendar now sends only the frozen request fields and validates requests strictly; no contract change. The created memory is selected immediately, including if the subsequent schedule write fails. `selects the created memory even when saving its slot schedule fails` and both corrected store request tests failed before their fixes and now pass. E2E `create a memory by clicking an empty calendar slot` passes, including persisted occurrence/timezone checks. The matching fixme is removed. |
| Calendar keyboard | PASS | `uses one grid Tab stop and moves actual focus across week slots without hijacking toolbar arrows` failed before the fix and now passes. `moves month focus across the visible boundary and navigates occurrences within a day` covers month boundaries, focus/highlight parity and day occurrences. The calendar region has one roving Tab stop, full date names, actual focus movement, and native Enter activation; radios, toolbar/date-picker controls and their portals retain keyboard behavior. E2E `calendar: roving focus, native toolbar arrows and Enter activation` passes for week slots, view radio arrows, date-picker popup arrows, day slot Enter and occurrence arrow/Enter activation. |

## Final spec results

Run: `apps/sparky-desktop/test-results/w3-fix-shell-final2/results.json` and `html/`. **7 passed, 0 failed, 0 skipped/fixme; 19.5 seconds.** Existing notification stub and private databases were used; no live native-reminder acceptance is claimed.

| Spec | Result / test names |
| --- | --- |
| `shell.spec.ts` | 2 PASS: `shell: Enter and Space activate native buttons and Save schedule preserves memory status`; `shell: keyboard navigation, Commander quick add, real core, and global details` |
| `minds.spec.ts` | 1 PASS: `create, rename, and delete a Mind with cascade confirmation` |
| `calendar.spec.ts` | 2 PASS: `create a memory by clicking an empty calendar slot`; `calendar: roving focus, native toolbar arrows and Enter activation` |
| `settings.spec.ts` | 2 PASS: `settings: Map Tiles and App Map select distinct panels`; `settings: round-trip, MCP off test, and token rotation against the real core` |

The controlled baseline package used unchanged product source at the initial branch HEAD. Run `w3-fix-shell-red-renderer` executed all seven cases: one passed and six failed. It reproduced the Calendar request rejection, 182 grid Tab stops, hidden Mind editor, duplicate Map selection and swallowed Commander Enter. The original Settings case additionally had stale expectations for copy already changed by W3-POLISH; those expectations now verify the current equivalent user-facing statements about disabled MCP, unavailable login startup, word matching and unavailable semantic search. No Settings production copy changed in this task.

## Verification before the last commit

| Gate | Result |
| --- | --- |
| `sparky-shell` test / typecheck | PASS: 98 tests |
| `sparky-minds` test / typecheck | PASS: 13 tests |
| `sparky-calendar` test / typecheck | PASS: 46 tests |
| `sparky-settings` test / typecheck | PASS: 38 tests |
| `sparky-map` test / typecheck | PASS: 16 tests; registration-only source change |
| Total package tests | PASS: 211 tests, 25 files |
| `pnpm design:lint` | PASS |
| Whole-repo `pnpm typecheck` | PASS: 27 tasks |
| Desktop E2E `tsc --noEmit -p tests/tsconfig.json` | PASS |
| `git diff --check` | PASS |

Final package commands selected the five packages explicitly with `pnpm --filter @polterware/feature-sparky-<name>`, running `test` and `typecheck` separately. Logs are `/tmp/w3-fix-shell-packages-final-{test,ts}.log`, `/tmp/w3-fix-shell-final-{design,repo-ts,e2e-ts,diff}.log` and `/tmp/w3-fix-shell-e2e-final2.log`.

## Harness deviations and reproduction

The authorized standard `prepare-e2e.sh` was run successfully. Its original app collided with Erick's existing `/Applications/Sparky.app` at the single-instance check. A temporary bundle ID/product name solved that without closing his app. The launched ASAR package then returned 404 for the renderer document and showed a blank window. Source inspection and the successful `asar=false` build isolate the issue to the secure renderer filesystem path through ASAR; its swallowed internal exception was not claimed as directly observed.

The working test package retains the same Main/preload, native addon, renderer and core; only local builder flags differ. No shared harness or packaging source was edited. The final renderer build and package commands were:

```bash
pnpm --filter @polterware/sparky-desktop exec electron-vite build
CSC_IDENTITY_AUTO_DISCOVERY=false pnpm --filter @polterware/sparky-desktop exec electron-builder \
  --mac --dir --publish never --config.mac.identity=null --config.asar=false \
  --config.appId=com.polterware.sparky.w3-fix-shell --config.productName=Sparky-W3-Fix-Shell \
  --config.directories.output=test-results/green-unpacked2
SPARKY_E2E_EXECUTABLE="$PWD/apps/sparky-desktop/test-results/green-unpacked2/mac-arm64/Sparky-W3-Fix-Shell.app/Contents/MacOS/Sparky-W3-Fix-Shell" \
  SPARKY_E2E_RUN=w3-fix-shell-final2 pnpm --filter @polterware/sparky-desktop exec playwright test \
  e2e/shell.spec.ts e2e/minds.spec.ts e2e/calendar.spec.ts e2e/settings.spec.ts
```

Build output, native output and test results are ignored and uncommitted. AGENTS.md and pnpm-lock.yaml remain unchanged; no Turbo-injected change was present to retain or revert.

## Requests and handoff

- [Request 1](../requests/w3-fix-shell-1.md): fix default ASAR renderer asset loading; exact temporary shared-harness flag delta supplied. Standard packaged acceptance remains blocked.
- [Request 2](../requests/w3-fix-shell-2.md): W3-FIX-LISTS/integrator removes the unowned memories spec quarantine after resolving its independent row-rendering blocker. The owned shell regression already covers w3-e2e-6.
- Original owned issues resolved: w3-e2e-4/5/6 and w3-polish-1/3/4/8. The SearchPage part of polish-8 remains outside this task; no search source/spec was edited or claimed as verified.
- Polterware CLI: the available local binary's `format` ran once; `status`, `query cards`, `apply` of `/tmp/w3-fix-shell-work-card.pw.md` (with summary/rationale) and closing `handoff` reject this unregistered worktree with `no project for this directory`. No unrelated project/database was selected or initialized. This task-authorized status is the reviewable fallback.
