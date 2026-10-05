# W3-FIX-ASAR status

- Result: **partial overall; ASAR regression fixed and complete standard-package rerun performed**. Functional E2E: **24/25 PASS, 1 FAIL, 0 FIXME**. The active failure is a newly demonstrated stale collection title after a successful detail save; product source is outside ownership. Performance: 0/2 PASS. Live native: 0/1 PASS.
- Branch: `sparky/w3-fix-asar`, already based on `sparky-next` at `2a92c5c` when work started. Implementation commits: `24b6c49` (secure unpacked assets), `cb6f0aa` (E2E reactivation, harness corrections and request). This status is the final local report commit. No push, tag or new branch.
- Frozen contracts unchanged. No feature, design-system, Go, contract, shared-root, lockfile or performance assertion changes. `AGENTS.md` remained unchanged; no Turbo-injected edit was retained. Build output and reports remain ignored and uncommitted.

## Root cause and fix choice

SEC-01's handler opens renderer directories with `O_DIRECTORY` and then validates physical descriptors. `app.asar/out/renderer` is a virtual archive subtree, so those opens fail and the handler returns 404 for the bundled document and assets. No arbitrary file fallback is acceptable.

The preferred physical solution is implemented: `electron-builder.yml` keeps `asar: true` and unpacks only `out/renderer/**`. The handler maps only a root ending in `app.asar/out/renderer` to its physical `app.asar.unpacked/out/renderer` sibling. The document and asset URLs retain their original ASAR origin, preserving existing navigation, trusted-sender and CSP behavior without edits to those owners' files.

The URL allowlist still uses only the logical renderer root; direct unpacked URLs and Main/preload/archive siblings are denied. Missing unpacked output fails closed. A symlinked unpacked root is also denied. Existing nested descriptor checks, symlink/hardlink denial, MIME allowlist, 8 MiB cap, `nosniff`, unsupported-platform denial and catch-to-404 behavior remain. There is no `net.fetch(file://)` fallback or archive-read bypass.

## Regression and security evidence

- Existing Main/preload unit tests plus the new packaged-root test: **23 files, 136 tests PASS**. The new unit test uses a non-directory `app.asar`, serves only the unpacked renderer through virtual URLs, and denies direct physical URLs, archive siblings, missing assets, unsupported types, oversized files, symlink files/directories/root and hardlinks.
- `e2e/renderer-package.spec.ts` reads the **real built archive** using the packager's installed `@electron/asar`: index/JS/CSS exist, every allowlisted renderer entry is marked unpacked and physically present. It launches the real standard executable, completes onboarding, renders main content, fetches HTML/JS/CSS with correct MIME and `nosniff`, and denies a disposable sentinel, Main JS and the direct unpacked document with **404**.
- An original-versus-fixed handler comparison against that actual packaged renderer root returned **404 before / 200 after** for `index.html`. The comparison used the original source from `sparky-next` in `/tmp`; no product source was temporarily reverted.
- Real Electron security PoC **PASS** after the final renderer build: `tests/security/electron-boundaries.cjs`, with installed Playwright tooling and this worktree's core binary. Sentinel `blocked:404`; Node/process/ipcRenderer undefined; popup denied; sandbox/context isolation verified; enforced CSP has no wildcard/loopback/Nominatim exception; MCP connection probe succeeds after enabling it in the disposable core. Log: `/tmp/w3-fix-asar-security-final.log`.

## Standard package and functional rerun

`bash apps/sparky-desktop/tests/scripts/prepare-e2e.sh` was run with **unchanged script semantics** and succeeded, including core, SDK driver, native addon, renderer and unsigned directory package. It was repeated after the final handler change. No `asar=false`, replacement app identity, alternate renderer build or diagnostic package was used for acceptance.

Executable: `apps/sparky-desktop/test-results/build/mac-arm64/Sparky.app/Contents/MacOS/Sparky`. All databases and renderer profiles were per-test disposable directories; the existing `/Applications/Sparky.app` process was preserved. Functional tests use the existing notification stub; live tests below use the real addon.

Final full command:

```bash
SPARKY_E2E_EXECUTABLE="$PWD/apps/sparky-desktop/test-results/build/mac-arm64/Sparky.app/Contents/MacOS/Sparky" \
  SPARKY_E2E_RUN=w3-fix-asar-final2 \
  pnpm --filter @polterware/sparky-desktop test:e2e
```

Final evidence: `apps/sparky-desktop/test-results/w3-fix-asar-final2/results.json`, `html/`, and `artifacts/`. **25 cases across 17 specs: 24 passed, 1 failed, 0 skipped/fixme, 0 retries; 75.961 seconds.** No `SPARKY_E2E_RECHECK_GAPS` flag was needed. Both old Memories/agent-writes fixmes are removed, and no new quarantine was added.

| Spec | PASS | FAIL | FIXME | Result / evidence |
| --- | ---: | ---: | ---: | --- |
| agent-writes.spec.ts | 2 | 0 | 0 | Real stdio and HTTP SDK writes appear in the open window. |
| attachments.spec.ts | 3 | 0 | 0 | Mixed files, granted-path drop and denied microphone. |
| calendar.spec.ts | 2 | 0 | 0 | Slot creation and keyboard/focus behavior. |
| conflict.spec.ts | 1 | 0 | 0 | Retained draft and Reload/Overwrite controls. |
| dst.spec.ts | 1 | 0 | 0 | Local daily time across DST. |
| focus.spec.ts | 1 | 0 | 0 | Start/pause/skip/reset/Commander and recorded sessions. |
| lifecycle.spec.ts | 2 | 0 | 0 | Core crash recovery and last-window daemon lifetime. |
| map.spec.ts | 1 | 0 | 0 | Offline network guards and real place title. |
| memories.spec.ts | 1 | 1 | 0 | Virtualized scroll PASS; saved-title collection invalidation FAIL, request 1. |
| mind-cascade.spec.ts | 1 | 0 | 0 | Parent/child removal moves memories to Inbox. |
| minds.spec.ts | 1 | 0 | 0 | Create/rename/delete through the actual editor. |
| renderer-package.spec.ts | 1 | 0 | 0 | Actual ASAR metadata, physical assets, render smoke and sentinel 404. |
| review.spec.ts | 1 | 0 | 0 | Range, summary, heatmap and chart tables. |
| schedule.spec.ts | 1 | 0 | 0 | Daily schedule, occurrence preview and snooze. |
| search.spec.ts | 1 | 0 | 0 | Commander, query and FTS-only copy. |
| settings.spec.ts | 2 | 0 | 0 | Distinct Map settings, settings persistence and token rotation. |
| shell.spec.ts | 2 | 0 | 0 | Native Enter/Space, navigation, Commander and details. |
| **Total** | **24** | **1** | **0** | Complete default functional suite. |

The active Memories failure occurs after successful native keyboard creation and persistence: core title is `Edited memory`, but after Meta+J the row still says `Keyboard memory`. The test checks the typed value and real-core persistence and attaches `saved-title-and-collection`; it retains all later pin/complete/undo/delete actions. Those later actions remain unverified because the stale title blocks them. [Request 1](../requests/w3-fix-asar-1.md) includes reproduction, repeated failures and trace paths. No product implementation outside ownership was changed.

The initial full run `w3-fix-asar-functional` had 22 PASS / 3 FAIL. Two test defects were corrected: Focus's partial `Short break` text selector matched several valid elements; the scroll scenario assumed that default due-date order matched creation/title order. The final tests select the exact Focus heading and explicitly sort Memories by Title before scrolling to title 00119. A subsequent full run `w3-fix-asar-final` had 23 PASS / 2 FAIL because teardown called Playwright's disposed application channel after normal last-window exit. The fixture now captures the process at launch. Its stranded disposable core was identity-checked, terminated and cleaned; the final complete run above verifies the correction. Earlier reports/traces remain available and are not counted as final passes.

## Unchanged performance checks on the same standard package

Run: `SPARKY_E2E_PERFORMANCE=1 SPARKY_E2E_RUN=w3-fix-asar-perf`, with the same executable and `test:e2e` command. `perf/desktop.spec.ts`: **0 PASS / 2 FAIL / 0 FIXME**; `tests/perf/**` is unchanged.

| Measurement | Observed | Unchanged gate / result |
| --- | ---: | --- |
| Interactive startup proxy | 1965.108 ms | <1000 ms: FAIL; exact Chromium FMP is not established by this proxy. |
| Core idle RSS | 38.141 MiB | <60 MB: PASS. |
| Main / renderer idle RSS | 180.219 / 142.281 MiB | Recorded; no numeric gate. |
| 10k virtualized rows | 19 | Nonempty and <100: PASS. |
| Scroll average / FPS | 22.488 ms / 44.469 FPS | Average ≤17.2 ms: FAIL. |
| Scroll p95 | 33.2 ms | Above 20 ms; metric recorded, later assertion is not reached after the average fails. |

Evidence: `test-results/w3-fix-asar-perf/results.json`, `scroll-performance`, `scroll-chromium-trace` and retained Playwright traces. Machine: arm64/macOS 26.6.2, Node 26.8.2, pnpm 12.6.0, Electron 44.4.5. Another `w3-fix-perf` Electron/seed harness was active on this host during the measurements, so this is not an isolated-machine comparison or an ASAR performance attribution. No budget was relaxed. The performance owner should consume these standard-package traces; existing requests `w3-fix-lists-2` and `w3-e2e-9` already route these gates.

## Live native rerun

Run: `SPARKY_E2E_LIVE_NOTIFY=1 SPARKY_E2E_RUN=w3-fix-asar-live`, same standard executable and real addon. `live/reminders.spec.ts`: **0 PASS / 1 FAIL / 0 FIXME**. Authorization again throws `PublicIpcError: Native notification operation failed`; native logs show `delegate-installed` then `refresh-failed`. This matches existing [w3-e2e-7](../requests/w3-e2e-7.md). No reminder was armed; closed-process delivery, Done/Snooze, sleep/wake and reboot are not claimed.

Trace: `test-results/w3-fix-asar-live/artifacts/live-reminders-real-native-b34f7-lectron-and-the-core-closed/trace.zip`. No ad-hoc signing, personal database or manual OS preference change was used to turn this failure into a pass.

## Verification and handoff

- Desktop `typecheck`: PASS.
- Desktop `test`: PASS, 23 files / 136 tests.
- Harness `tsc --noEmit -p tests/tsconfig.json`: PASS after the final fixture/assertion changes.
- `pnpm design:lint`: PASS.
- `git diff --check` and owned-path review: PASS. No shared-root, `AGENTS.md`, lockfile or perf-source delta.
- Verification logs: `/tmp/w3-fix-asar-{typecheck,unit,design,e2e-ts,diff}-final.log`; build log `/tmp/w3-fix-asar-prepare-final.log`. Functional, performance and native failures are retained explicitly above.
- ASAR requests `w3-fix-lists-1` and `w3-fix-shell-1` are addressed. The old quarantine removal requested by `w3-fix-shell-2` is implemented; full Memories acceptance now needs request 1, a distinct live collection invalidation defect.
- Polterware CLI: the main-checkout binary ran `format` once. `status`, `query cards`, `apply /tmp/w3-fix-asar.pw.md` (summary/rationale) and closing `handoff` reject this unregistered worktree with `no project for this directory`. No unrelated store was selected or initialized; this task-authorized status/request is the fallback handoff record.
- Integrator: apply the secure ASAR fix, resolve request 1 through the Memories owner, integrate the performance owner's changes, then rerun the active keyboard and unchanged performance gates. Native authorization remains a separate existing release gate. Full green product E2E acceptance is not claimed.
