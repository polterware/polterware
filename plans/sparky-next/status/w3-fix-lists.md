# W3-FIX-LISTS status

- Result: **partial**. Virtualized rows and Map pagination are fixed and verified. The standard ASAR package fails before onboarding in unchanged Main asset handling; the diagnostic unpacked package renders rows, but the existing keyboard scenario and measured FPS gate still fail. No full packaged acceptance is claimed.
- Branch: `sparky/w3-fix-lists`, from `sparky-next` at `dd404d8`. Implementation commits: `a70b1f6` (table mount lifecycle and row regressions), `6577a64` (Map pagination and offline spec). This status/request is the final local commit.
- Frozen contracts: `sparky-contracts-v1.6` unchanged. No shell, Main, Go, contracts, shared root files, lockfiles or build outputs changed in the commits. No push or tag.

## Root cause and fixes

**Empty virtualized tables:** `DataTable.Table` publishes its scroll ref while Solid's cloned template nodes still belong to an inert document. `createVirtualizer` eagerly attaches in a computed before mount, caches that scroll element, and captures a null `ownerDocument.defaultView`. Its rectangle/offset observers do not attach. Mount preserves the element identity, so the adapter does not retry attachment. Temporary diagnostic instrumentation reported count 1000, element height 240, scroll element present, null target window, rectangle 0×0, and empty range/items. The instrumentation was removed.

`DataTable.Body` now exposes the scroll element only after mount. This attaches observers to the actual document/window. Virtualization, overscan, estimated row heights, public APIs, shared tokens and nonvirtualized behavior are preserved. Memories needs no production sizing/CSS change.

The regression `renders and scrolls a measured virtualized viewport mounted after body creation` failed on the original implementation: the first row remained absent even with measured `offsetHeight`/`offsetWidth` and a ResizeObserver mock. The original suite had 200 passes and this one failure. With the fix it renders a bounded subset and scrolling to offset 20000 renders row 500 and removes row 0. The feature regression `renders a nonempty virtualized subset after asynchronous fake-bridge loading` exercises 1200 memories through the fake preload bridge with virtualization enabled.

**Map query:** `memories.list({ hasPlace: true, limit: 1000 })` violates the frozen 500-row page bound. The store now follows `nextCursor` in pages of 500, guards each asynchronous response with a load generation, and invalidates pending loads on disconnect/dispose. Repeated cursors and more than 20 pages fail explicitly rather than displaying partial titles as live; the total safeguard is 10000 titles. The fake bridge now implements the page bound and cursor semantics.

`paginates 1200 place titles in pages of at most 500` verifies three requests, all 1200 places, and the final memory title. Additional tests cover stale pagination/newer bounds, disposal, repeated cursors and bounded traversal. The Map E2E seeds a real place and asserts its memory title. The previously quarantined test also needed to dismiss the first-run disclosure before querying the inert background main/navigation. Offline-tiles and geocoding network assertions remain intact.

## Other feature-store audit

`rg -n 'limit:' packages/features/sparky-*/src` and a broader `limit` search across stores/bridges found no other out-of-range query. Memories already paginates at 500. Activity uses 100, Mind counts use 1, search uses 20/50 and schedule preview uses 5. `schedule.listOccurrences` uses 10000, which is explicitly allowed by `sparky/backend/domains/schedule/occurrences.go` (a different bound from Memory/Activity); it is not reduced to 500. Other owners' stores remain untouched.

## Browser and unsigned Electron evidence

Headless Chromium ran the existing W3-POLISH fake-bridge audit harness without `preview=list`: Inbox rendered all five matching rows at widths 800, 1280 and 1920, `data-virtualized=true`, viewport heights 485/525/525, no page errors.

The authorized `prepare-e2e.sh` built the core, SDK driver, real native addon, renderer and unsigned directory package successfully. **All five selected specs failed before onboarding** on a blank window in the standard ASAR package. JS/CSS requests return 404 although their archive entries exist. The unchanged secure renderer handler opens the archive's renderer root as a physical directory; [request 1](../requests/w3-fix-lists-1.md) routes this to Main/packaging. Standard results/traces: `apps/sparky-desktop/test-results/w3-fix-lists/`.

For list-specific diagnosis, the same build was packaged with CLI `--config.asar=false` into ignored `test-results/build-unpacked`, retaining the actual Main/preload/core and security handlers. This is **diagnostic unpacked-package evidence**, not acceptance of the default ASAR package.

| Spec / case | Diagnostic result | Evidence |
| --- | --- | --- |
| `memories.spec.ts`: virtualized core rows and scroll to last memory | PASS: nonempty subset of 120; last row visible after scroll | `test-results/w3-fix-lists-unpacked/results.json` |
| `agent-writes.spec.ts`: stdio SDK write invalidates open window | PASS: title rendered and real core persistence verified | Same report; `SPARKY_E2E_RECHECK_GAPS=1` |
| `agent-writes.spec.ts`: HTTP SDK write invalidates open window | PASS: title rendered and real core persistence verified | Same report; `SPARKY_E2E_RECHECK_GAPS=1` |
| `map.spec.ts`: offline tiles and places | PASS after disclosure-order correction; real place/title, no tile requests, no Nominatim request before opt-in/manual search | `test-results/w3-fix-lists-map/results.json` |
| `memories.spec.ts`: keyboard create/edit/pin/complete/undo/delete | FAIL: shell consumes Enter on New memory; dialog never opens | `test-results/w3-fix-lists-unpacked/artifacts/e2e-memories-keyboard-crea-afed5-in-complete-undo-and-delete/trace.zip`; existing [w3-e2e-6](../requests/w3-e2e-6.md) |
| `perf/desktop.spec.ts`: 10k virtualization/FPS | Rows PASS: 19 rendered out of 10000. FPS gate FAIL: **45.498 FPS**, average **21.979 ms**, p95 **26 ms** (unchanged limits ≤17.2/20 ms) | `test-results/w3-fix-lists-perf/results.json`, including `scroll-performance` and `scroll-chromium-trace` attachments |

The first unpacked functional run had 3 passes and 2 failures (Map's test ordering and shell keyboard); the corrected Map rerun passed. Results are not overwritten or counted as a single fully passing run. The FPS test reached Chromium scroll tracing and captured 180 frames; no empty-list FPS is claimed. Its failure Playwright trace is under `w3-fix-lists-perf/artifacts/perf-desktop-10k-memories--e1bc0-nd-list-scrolling-FPS-trace/trace.zip`. Machine: arm64/macOS 26.6.2, Node 26.8.2, pnpm 12.6.0, Electron 44.4.5. Synthetic isolated data only.

## Verification before final commit

| Gate | Result |
| --- | --- |
| `pnpm --filter @polterware/frontend-design-system test` | PASS: 29 files, 201 tests (includes existing Polterware table behavior) |
| `pnpm --filter @polterware/feature-sparky-memories test` | PASS: 5 files, 37 tests |
| `pnpm --filter @polterware/feature-sparky-map test` | PASS: 4 files, 21 tests |
| Each touched package's `typecheck` | PASS: design-system, Memories, Map |
| `pnpm design:lint` | PASS |
| Whole-repo `pnpm typecheck` | PASS: 27 tasks |
| `pnpm --filter @polterware/sparky-desktop exec tsc --noEmit -p tests/tsconfig.json` | PASS |
| `git diff --check` | PASS |

Reproduction (repository root; default package remains blocked):

```bash
bash apps/sparky-desktop/tests/scripts/prepare-e2e.sh
CSC_IDENTITY_AUTO_DISCOVERY=false pnpm --dir apps/sparky-desktop exec electron-builder \
  --mac --dir --publish never --config.directories.output=test-results/build-unpacked \
  --config.mac.identity=null --config.asar=false
SPARKY_E2E_EXECUTABLE="$PWD/apps/sparky-desktop/test-results/build-unpacked/mac-arm64/Sparky.app/Contents/MacOS/Sparky" \
  SPARKY_E2E_RECHECK_GAPS=1 SPARKY_E2E_RUN=lists-recheck \
  pnpm --filter @polterware/sparky-desktop exec playwright test \
  e2e/memories.spec.ts e2e/map.spec.ts e2e/agent-writes.spec.ts
SPARKY_E2E_EXECUTABLE="$PWD/apps/sparky-desktop/test-results/build-unpacked/mac-arm64/Sparky.app/Contents/MacOS/Sparky" \
  SPARKY_E2E_PERFORMANCE=1 SPARKY_E2E_RUN=lists-perf-recheck \
  pnpm --filter @polterware/sparky-desktop exec playwright test perf/desktop.spec.ts -g '10k memories'
```

## Requests and handoff

- New request: [w3-fix-lists-1](../requests/w3-fix-lists-1.md), ASAR renderer asset handling. Rerun the standard package after its owner fixes this gate.
- New request: [w3-fix-lists-2](../requests/w3-fix-lists-2.md), the measured 10k scroll frame-budget miss, with the Chromium trace and unchanged thresholds.
- Existing shell-owner request [w3-e2e-6](../requests/w3-e2e-6.md) remains; the Memories keyboard fixme retains only that reason. This branch does not merge or edit the concurrent shell workstream.
- List rendering closes the code defect in w3-e2e-3 and the list part of w3-polish-5. The E2E owner can remove `agent-writes.spec.ts`'s file-level fixme after integration; that file is outside this ownership, so verification used the existing recheck environment flag.
- Map's limit defect in w3-e2e-2 is fixed; its fixme is removed. No performance-test assertions or thresholds were weakened. The now-measurable FPS miss needs performance follow-up before claiming that gate passes.
- Polterware CLI: the available binary ran `format` once. `status`, `query cards`, `apply` (prepared `/tmp/w3-fix-lists.pw.md` with summary/rationale) and closing `handoff` reject this unregistered worktree with `no project for this directory`. No other project/database was selected or initialized. This mandated workstream status is the fallback record.
