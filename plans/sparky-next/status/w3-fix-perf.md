# W3-FIX-PERF status

- Result: **done for scrolling performance in the authorized diagnostic unpacked package**; unchanged functional E2E failures are retained and routed below.
- Branch / implementation commit: `sparky/w3-fix-perf`, from `sparky-next` at `2a92c5c`; implementation `d630f22`.
- Ownership: shared `table*`, Sparky Memories, this status and own requests only. Main, shell, Go, contracts, shared configuration, lockfiles and performance assertions/budgets remain unchanged. No push or tag.

## Profile: top three causes

Original captured trace: `../w3-fix-lists/apps/sparky-desktop/test-results/w3-fix-lists-perf/results.json` (`scroll-chromium-trace`). It contains 179 main frames with **3884.49 ms** cumulative `ProxyMain::BeginMainFrame`, **3033.97 ms** `EventDispatch`, **265.31 ms** style, **221.78 ms** layout and **187.25 ms** paint. Inclusive/nested event totals are not additive. Original gate: **45.498449 FPS**, **21.978771 ms average**, **26 ms p95**, **19 initial rendered rows**, a real nonempty 10k list.

A separate CPU/counter diagnostic retained the unchanged 180-frame scrolling loop, with profiler sampling at 100 microseconds and layout/observer/DOM counters. Its instrumentation affects timing, so its **42.043571 FPS / 23.784849 ms average / 33.4 ms p95** is diagnostic, not a replacement baseline. Profile and full Chromium trace are JSON attachments in `apps/sparky-desktop/test-results/profile/test-results/profile/results.json`; its Playwright failure trace is retained beside that report.

1. **Per-row Ark completion checkbox construction: 1608.04 ms CPU sampled in its subtree**, or **8.984 ms per scroll tick**, inside **2376.86 ms** sampled row construction. The jump-scroll workload adds **4989 rows** and removes **4988**, **27.872 new rows/tick**; each checkbox creates the Ark form state machine and multiple indicator components. Row audit: **27 elements / 3 SVGs** per row, including two checkbox indicator SVGs.
2. **Repeated primary-cell JSX/icon construction: 540.81 ms** sampled in `DataTablePrimaryCell`, **3.021 ms/tick**. Its `leading` accessor evaluates a JSX getter separately for presence and rendering; the `icon` branch also rereads the JSX getter. Resolving that slot in a memo avoids redundant icon/component construction. The virtual body also ties the DOM to adapter measurement objects rather than source identity; overlapping rows must retain DOM/focus independently of the adapter.
3. **Unnecessary row geometry and layout work:** **4989 ResizeObserver.observe / 4988 unobserve** operations; **15027 offsetParent + 10018 offsetTop + 10018 offsetLeft + 5009 offsetWidth + 5009 offsetHeight = 45081 offset reads** during the diagnostic, with **20.91 ms** sampled directly in hover measurement. Original style+layout totals **487.09 ms (2.721 ms/tick)**; the trace cannot assign all of that layout time exclusively to hover. The previous highlight measures every registered row even when no hover/focus/active marker is visible. Fixed column sizing and viewport containment also remove intrinsic sizing propagation.

Allocation consequences in the original trace: **53 MinorGC events / 190.08 ms**, **8 MajorGC / 28.04 ms**, total **218.12 ms**. CPU diagnostic GC samples total **257.05 ms**. Original scroll callbacks alone total **3033.504 ms / 179 events (16.947 ms average, 24.525 ms maximum)**; scrollend totals **0.465 ms / 179 events**. Reactive work is concentrated in one virtualizer range notification per tick: **2956.48 ms** sampled below its onChange (includes descendant row creation). Reconciliation itself samples **5.42 ms**; selected/filter/store rows do not depend on scroll offsets. Row measurement by the virtualizer is absent (no `measureElement` is installed). Diagnostic `getBoundingClientRect` count during this no-pointer loop is **0**, while offset reads are the churn described above.

No row/descendant filter or backdrop-filter is active. The only row shadow is the checkbox's small inset token shadow. Original cumulative paint is **187.25 ms / 360 events**, layerization **42.23 ms / 179 events**; these are smaller than row JS. macOS vibrancy stays unchanged in Main/shell and is not established as the dominant cause. The actual packaged GPU/environment details and after traces will be recorded below.

## Fixes

- Memoize the primary-cell leading JSX slot; resolve an icon getter once.
- Render virtual rows using stable source entries with id keys in the virtualizer; preserve overlapping DOM, keyboard focus, activation, selection and nested controls.
- Opt Memories into a fixed **48 px row pitch (44 px row + 4 px existing gap)** and overscan **4**; retain default estimated-height/overscan behavior for Polterware. Fixed table columns avoid intrinsic sizing; strict containment belongs on the sized flex viewport. Native table rows cannot safely use size containment, so no invalid `contain: strict`/`content-visibility` is applied to them.
- Register row hover geometry only when a pointer/focus or active marker needs it, preserving the shared spring highlight. No-marker scrolls attach no row observers and perform no row layout reads. Hover/focus still uses the existing shared measured geometry rather than changing unrelated motion primitives.
- Shared `DataTable.Checkbox` uses a labeled native binary checkbox and table token styling, replacing the expensive completion-only form state machine. Selection remains independent; native Space/focus semantics, controlled checked state and row-action isolation remain.

## Requests

- [w3-fix-perf-1](../requests/w3-fix-perf-1.md): shared configuration hardcodes development Solid exports in packaged builds. Exact Vite condition delta supplied; no shared file edited.
- Existing ASAR handling request w3-fix-lists-1 remains owned by parallel Main/packaging work. All runs here use the diagnostic unpacked package (`--config.asar=false`), not standard ASAR acceptance.
- Startup/FTS/core cold-start work from w3-e2e-9 is outside this ownership and is not claimed fixed.

## Unchanged performance gate

The spec `apps/sparky-desktop/tests/perf/desktop.spec.ts` is byte-for-byte unchanged from `sparky-next`: same real seed, nonempty/subset assertions, 180-frame loop, average **<=17.2 ms** and p95 **<=20 ms**. No assertion, timer, trace category, viewport, refresh-rate cap or virtualizer was removed/relaxed. Each run uses a fresh throwaway database/user-data directory, real core/Main/preload/renderer, and the existing notification stub.

| Run | FPS | Average ms | p95 ms | Initial rows / dataset | Result |
| --- | ---: | ---: | ---: | --- | --- |
| Original captured W3-FIX-LISTS | 45.498449494 | 21.978770950 | 26 | 19 / 10000 | FAIL |
| Before CPU/counter diagnostic (instrumented) | 42.043571233 | 23.784849162 | 33.4 | 19 / 10000 | FAIL, diagnostic |
| After initial row fixes, before final icon/id memoization | 117.477193673 | 8.512290503 | 10.2 | 15 / 10000 | PASS |
| Final unchanged spec, fresh run 1 | 118.394073682 | 8.446368715 | 10.2 | 15 / 10000 | PASS |
| Final unchanged spec, fresh run 2 | 118.660921445 | 8.427374302 | 10.2 | 15 / 10000 | PASS |

First after report: `apps/sparky-desktop/test-results/w3-fix-perf-after-1/results.json`. Final report and both Chromium/metrics attachments: `apps/sparky-desktop/test-results/w3-fix-perf-final/results.json`. Successful runs retain their Chromium traces as report attachments; the fixture intentionally removes Playwright failure-only zip traces on success. Artifacts are ignored/local and must be copied before deleting this worktree.

| Renderer main-thread cumulative work | Original | Final 1 | Final 2 |
| --- | ---: | ---: | ---: |
| BeginMainFrame | 3884.490 ms / 179 | 1266.200 ms / 180 | 1243.545 ms / 180 |
| Scroll event handler | 3033.504 ms / 179 | 567.449 ms / 179 | 575.480 ms / 179 |
| Style | 265.310 ms / 179 | 190.946 ms / 296 | 174.854 ms / 179 |
| Layout | 221.780 ms / 179 | 171.162 ms / 180 | 170.870 ms / 179 |
| Paint | 187.250 ms / 360 | 149.834 ms / 360 | 148.120 ms / 360 |
| MinorGC | 190.080 ms / 53 | 32.781 ms / 7 | 33.995 ms / 7 |
| MajorGC | 28.040 ms / 8 | 13.858 ms / 3 | 16.622 ms / 3 |
| Layerize | 42.230 ms / 179 | 45.649 ms / 180 | 44.540 ms / 180 |
| Animation-frame callback | 550.040 ms / 537 | 412.908 ms / 346 | 368.236 ms / 180 |

Final maximum BeginMainFrame is **13.673 / 13.685 ms**; maximum scroll handler **9.023 / 9.853 ms**, versus original **24.525 ms**. Scroll JS is reduced approximately **81%**. Totals include nested/inclusive events and are not a sum of exclusive costs.

## Required verification before final commit

- Design system test: **PASS, 29 files / 202 tests**, includes Polterware sorting, active/selection, activation, details and selection toolbar behaviors. Enhanced virtualized regression keeps the same row DOM and keyboard focus across an overlapping range change and an appended data row. New checkbox regression covers native focus, controlled checked updates and isolation from selection/activation.
- Memories test: **PASS, 5 files / 38 tests**, including asynchronous nonempty virtualization, keyboard workflows and native completion without opening details.
- Both touched packages' typecheck: **PASS**.
- `pnpm design:lint`: **PASS**.
- `pnpm test:ui-opacity`: **PASS, 10 tests**.
- Whole-repository `pnpm typecheck`: **PASS, 27 tasks**.
- `git diff --check`: **PASS**; rerun after final status edits.
- No `AGENTS.md` change was injected by Turbo. No shared/root file, lockfile, package output or test-results artifact is staged/committed.

## Reproduction

From this worktree's root:

```bash
bash apps/sparky-desktop/tests/scripts/prepare-e2e.sh
CSC_IDENTITY_AUTO_DISCOVERY=false pnpm --dir apps/sparky-desktop exec electron-builder \
  --mac --dir --publish never --config.directories.output=test-results/build-unpacked \
  --config.mac.identity=null --config.asar=false
SPARKY_E2E_EXECUTABLE="$PWD/apps/sparky-desktop/test-results/build-unpacked/mac-arm64/Sparky.app/Contents/MacOS/Sparky" \
  SPARKY_E2E_PERFORMANCE=1 SPARKY_E2E_RUN=w3-fix-perf-final \
  pnpm --filter @polterware/sparky-desktop exec playwright test \
  perf/desktop.spec.ts -g '10k memories' --repeat-each=2
```

The standard package preparation builds its assets; profiling/acceptance here explicitly uses the subsequent **diagnostic unpacked package**. This does not claim standard ASAR loading, signing/native notification acceptance, true FMP or startup/FTS budgets from w3-e2e-9. Environment: Apple M1 Pro, **16 GiB**, arm64/macOS **26.6.2 (25G83)**, Electron **44.4.5**, pnpm **12.6.0**; display/GPU details and pointer-present profile below. No browser/GPU/headless limit prevented the frame gate from passing.

## Pointer-present CPU/geometry profile and GPU

`apps/sparky-desktop/test-results/profile-after/results.json` retains the final CPU profile, full Chromium trace, row audit, counters and GPU/display attachment. The independent diagnostic keeps the unchanged 180-frame loop and assertions, enables 100-microsecond CPU sampling, and places the pointer inside the list before scrolling. It includes a 200 ms pointer-entry settling interval in the geometry counter window. **PASS: 109.433270160 FPS, 9.137988827 ms average, 16.5 ms p95, 15 initial rows / 10000**. This is additional instrumented hover evidence, not a replacement for the two unchanged spec runs.

- Actual row audit: **44 px height / 48 px pitch**, **19 elements and 1 SVG per row**, versus original **27 / 3**. No row/descendant shadows, filters or backdrop effects remain in this fixture. Screenshot: `profile-after/artifacts/profile-10k-memories-virtu-6e9b0-nd-list-scrolling-FPS-trace/memory-list.png` (all paths relative to desktop `test-results/`).
- CPU sampled in native completion checkbox **9.83 ms**; primary cell **231.98 ms**; entire row **368.50 ms**; selection cell **19.34 ms**; generic cell **77.43 ms**; Pin Button **42.26 ms**; virtualizer onChange **578.26 ms**. These subtree totals overlap, and the workload/hover state differs from the original diagnostic. GC self samples **52.52 ms**.
- Live hover geometry still samples **24.75 ms** in measurement and **8.52 ms** in picking. Counters: **3560 rows added / 3561 removed**, **7185 observe / 7171 unobserve**, **5524 offsetParent + 3678 offsetTop + 3678 offsetLeft + 2027 offsetWidth + 2027 offsetHeight = 16934 offset reads**, and **181 getBoundingClientRect** calls for pointer picking. These pointer-present counters include pointer initialization/settlement and are not directly comparable with the original no-pointer counter window. Remaining shared hover measurement is explicitly retained; it does not prevent the pointer-present frame budget from passing.
- Native Electron window, primary display **1512x982**, scale **2**, refresh **120.0006103515625 Hz**. GPU compositing, rasterization, OpenGL, WebGL/WebGPU and hardware video decode/encode report enabled; multiple raster threads and Skia Graphite enabled. Direct display compositor/raw draw disabled-off-ok; trees-in-viz and WebNN disabled-off. No software-GPU/headless override was introduced. Refresh rate was observed, not changed. The cumulative scroll-JS reduction also establishes improvement independently of refresh-rate/FPS differences.

## Functional/a11y follow-up and final handoff

The unchanged functional `e2e/memories.spec.ts` report has **0 pass / 2 fail** in `test-results/w3-fix-perf-functional/results.json`. The first assertion assumes the wrong default order (expects 00119 at the bottom, while 00000 is actually the bottom). A controlled original-renderer package extracted from the initial ASAR reproduces this failure and also fails the keyboard test earlier at the stale post-rename row label (**0 pass / 2 fail**, `w3-fix-perf-functional-baseline/results.json`). The id-keyed body now updates virtualized row content after data changes; the shared table regression verifies this explicitly.

The final unchanged keyboard scenario reaches Delete confirmation but fails at the immediate keyboard activation/queued-deletion assertion. An ignored diagnostic copy with the correct bottom-row expectation and visible-dialog/250 ms focus settlement passes **2/2 in 7.2 s**, including last-row visibility, native keyboard actions, completion/undo, delete/undo and real persisted title. These diagnostic passes are not claimed as passes of the unchanged functional suite. [w3-fix-perf-2](../requests/w3-fix-perf-2.md) gives the E2E/shared Dialog owners the exact ordering delta and focus investigation evidence; their tracked sources remain untouched.

Polterware CLI: the available local `dist/polterware` binary ran `format` once. `status`, `query cards` and `apply /tmp/w3-fix-perf.pw.md` (frontmatter summary/rationale) reject this unregistered worktree with `no project for this directory`; closing `handoff --card sparky-w3-fix-perf --note ...` also rejects with the same error. No unrelated project/database is selected or initialized. This task-authorized status/request record is the fallback.

- Targeted acceptance: **PASS** for nonempty virtualization, both unchanged frame-budget runs, pointer-present diagnostic, shared Polterware table tests, touched package tests/typechecks and every required static gate.
- Result: **done for the scrolling-performance task using the authorized diagnostic unpacked package**. Standard ASAR acceptance remains with Main/packaging; the unchanged functional E2E failures remain explicit in request 2.
- Implementation commit: `d630f22` (`sparky(w3-fix-perf): Reduce virtual row construction and layout work`); final regression/status/request commit follows on the same branch.
- New requests: **w3-fix-perf-1** (production Solid exports), **w3-fix-perf-2** (functional ordering/dialog readiness). Existing ASAR request w3-fix-lists-1 and startup/core request w3-e2e-9 remain with their owners.
- No pushes/tags, lockfile/output/result commits, shared-file edits or AGENTS changes. Preserve/copy the ignored evidence before removing this worktree.
