# Startup remains over budget with production Solid and lazy Map views

Owner: Main / shell / feature-registry owners (W0-SCAFFOLD / integrator). No budget change requested.

## Standard-package measurements

Both packages were produced by `bash apps/sparky-desktop/tests/scripts/prepare-e2e.sh`, with the merged production Solid export condition, standard ASAR handling, Electron 44.4.5, unsigned arm64. Each launch used a fresh disposable database and renderer profile with the existing notification stub. No unpacked diagnostic package, personal data, cache flush or assertion relaxation was used.

| Run | Launch to visible main/onboarding, ms | Median, ms |
| --- | --- | ---: |
| Before this batch's Map lazy loading | 1716.268, 1199.152, 1145.948, 1133.154, 1473.219 | 1199.152 |
| First after series (retained, includes a 4229 ms outlier) | 4229.031, 1038.087, 1105.871, 1041.267, 1147.549 | 1105.871 |
| Final after series, no concurrent task-launched suite/build/check | 1094.380, 1069.366, 1065.938, 1091.828, 1328.570 | **1091.828** |

The unchanged gate is `<1000 ms`. All five final samples miss; the median exceeds it by **91.828 ms**. The measurement includes Electron/Main/core/preload/renderer and onboarding and is a conservative interactive proxy for the architecture's renderer FMP budget. Do not equate it with a navigation-relative paint metric. Earlier series overlapped some short unit-test work; the final series is the acceptance measurement. The host was not a dedicated benchmark machine.

## Profile and ownership boundary

Separate instrumented startup runs captured Chromium/V8 CPU traces and timestamps. These runs are diagnostic and add tracing overhead; their durations do not replace the five uninstrumented samples above.

| Milestone / work | Before Map lazy loading | After |
| --- | ---: | ---: |
| Main's environment hook reached (file mtime relative to launch) | 590.854 ms | 533.395 ms |
| Core state file written (mtime; not authenticated readiness) | 894.018 ms | 862.704 ms |
| Renderer navigation time origin relative to launch | 1102.200 ms | 1071.800 ms |
| Navigation-relative Chromium first meaningful paint | 473.213 ms | 511.838 ms |
| Approximate launch-to-FMP from those two clocks | 1575.413 ms | 1583.638 ms |
| Initial renderer JS, raw build output | 3241.04 KB | 3145.33 KB |
| Renderer module parse, background thread | 65.84 ms | 81.03 ms |
| Renderer module evaluation, main thread | 103.06 ms | 109.84 ms |
| Sampled preload JS self time, whole trace | 0.46 ms | 0.39 ms |

Parse/evaluation overlap other work and must not be added as exclusive costs. Preload sampling is not its full initialization wall time. Chromium FMP coincides with FCP in these fresh onboarding runs. The navigation-relative paint is under one second; the launch-to-visible proxy remains over budget. The trace has profiler-start overhead (48.38 ms in the after run), Playwright instrumentation, and no clean-machine/cache-cold qualification.

MapLibre's **1293.12 KB** chunk was already dynamically imported inside the shared map engine and **does not load in either startup trace**. It is not an established startup cause. This batch additionally made MapPage, PlaceCard and MapSettingsPanel lazy, removing **95.71 KB (2.95%)** from the initial bundle. Offline Map, places and Settings pass in the full standard-package suite. No claim that the median difference is exclusively caused by this change is made.

Remaining investigation / requested changes outside ownership:

1. `apps/sparky-desktop/src/main/core/process-supervisor.ts`: a newly spawned core always waits `delay(200)` before its first state/health attempt. Main then awaits authenticated `reconnect()` and `refreshMapPolicy()` before `showWindow()`. Profile this readiness path and consider bounded faster polling for a new child (for example 50 ms), preserving build replacement, authenticated health, database/PID identity and timeout checks. The deterministic 200 ms polling floor is a candidate, not a measured 200 ms saving.
2. `apps/sparky-desktop/src/main/index.ts`: separate Electron/bootstrap imports, app readiness, core attach/handshake and settings-policy RPC in a Main-owned timing regression. The after trace has **~1.072 s before renderer navigation**, so a renderer-only optimization cannot explain all of launch latency. Security/CSP policy must remain established before loading the document; do not bypass it.
3. `apps/sparky-desktop/src/renderer/features.ts` and remaining feature owners: the registry eagerly imports every feature; **3.145 MB** of initial JS remains after owned Map laziness. Profile and lazily register heavy routes/editors without changing feature contributions or shell contracts. The trace's module evaluation is **109.84 ms**; do not promise savings equal to the whole bundle's evaluation time.
4. `packages/features/sparky-shell/src/shell.tsx` / `connection.ts`: content is gated by `reminders.status`, then shell minds/count and Memories collection/metadata queries start. Measure their critical-path latency and duplicate Mind reads before deferring anything. Settings panels connect only when rendered; the initial Welcome step has no settings query. Main's map-policy settings read is the established startup settings call. No unsupported settings/onboarding-query attribution or security weakening is requested.

## Evidence and rerun

Reports under `apps/sparky-desktop/test-results/`: `w3-memories-startup-before`, `w3-memories-startup-after`, **`w3-memories-startup-final`**, `w3-memories-profile-before`, `w3-memories-profile-after`. Profile JSON attachments: `startup-milestones`, `startup-chromium-trace`, `desktop-performance`. Preserve the ignored reports before deleting the worktree.

```bash
SPARKY_E2E_EXECUTABLE="$PWD/apps/sparky-desktop/test-results/build/mac-arm64/Sparky.app/Contents/MacOS/Sparky" \
  SPARKY_E2E_RUN=startup-check SPARKY_E2E_PERFORMANCE=1 \
  pnpm --filter @polterware/sparky-desktop exec playwright test -g 'startup and idle' --repeat-each=5
# Separate diagnostic run only: add SPARKY_E2E_PROFILE=1, without --repeat-each.
```

No Main, shell, shared registry, design-system, Go, contract, lockfile or root configuration file was edited. Full functional E2E passes **25/25**; the unchanged desktop perf suite passes scroll and fails startup (**1/2**).
