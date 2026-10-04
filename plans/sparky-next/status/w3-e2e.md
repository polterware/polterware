# W3-E2E status

- Result: **partial**. The unsigned packaged runner works; all eleven original spec files were executed. Product bugs, native authorization, performance misses, and unapplied shared-file requests prevent a complete green gate.
- Branch: `sparky/w3-e2e`, based on `sparky-next` at `0a5dcfc3d74a120f953e1be3ba234a64bf234c11`. Implementation commits: `a3d8459` and `9b0eab7`; this status is included in the final verification commit.
- Contracts: `sparky-contracts-v1.6` preserved. No product, Main, preload, native, shared root, contract, or other package source was edited. No push or tag.
- Final functional run: **14/20 cases passed, 6 explicit fixmes, 0 unexpected failures** across 16 spec files. **7/11 original files fully pass** (9/13 original cases pass); five of seven added cases pass. This is not reported as full E2E acceptance.

## Acceptance and implementation

| Criterion | Result / evidence |
| --- | --- |
| Runnable unsigned app | PASS: `bash apps/sparky-desktop/tests/scripts/prepare-e2e.sh`; core, SDK helper, real arm64 native N-API addon, electron-vite and electron-builder directory package built. Signing explicitly skipped; no distributable/signing claimed. |
| Runner and isolation | PASS: serial, one worker, 60-second default test timeout, failure traces, list/HTML/JSON reporters; private per-test data/environment files and DB; detached-core teardown validates DB, PID and command before termination. |
| All eleven original specs run | PASS for execution, FAIL for full product acceptance. Baseline executed all 13 original cases; the subsequent unquarantined functional run executed all 20 cases before product failures were marked fixme with requests. |
| Additional functional scenarios | Onboarding already in shell. Real MCP stdio and HTTP SDK writes ran but visible rows are blocked. Conflict, crash/recovery, window-close daemon lifetime, Mind cascade, and DST pass. |
| 10k seed | PASS: domain-service Go fixture for core performance; real authenticated RPC client for desktop. No direct SQLite writes. |
| Performance report | Recorded below. Dedicated core run passes; earlier misses retained. Desktop startup proxy misses; FPS blocked. |
| Live native G1 reminder | FAIL before scheduling: real native authorization rejected in the unsigned package; manual gate remains. No stub delivery claimed. |
| Root command and artifact ignore | Pending integrator request 1. Desktop script works; root files untouched. |
| CI/headless acceptance | UNVERIFIED: only local macOS packaged runs performed. No CI/shared workflow edits authorized. |

No Main changes were needed. The existing `SPARKY_NOTIFY_MODULE` override supplies a test-only notification stub for functional/performance tests. `SPARKY_NOTIFY_E2E` stays unset; live tests load the real module. Existing `SPARKY_CORE_DB` in the Main environment-file hook derives the isolated core data directory. No global environment variables or settings are set by the prepare script.

The real MCP SDK client is a Go test-only executable using the already installed SDK, invoked by Playwright. Its stdio transport launches the actual `sparky -db <isolated-db> mcp`; HTTP uses the actual `/mcp` endpoint and bearer from the isolated private token file. This avoids an extra desktop dependency outside the authorized Playwright manifest change. Tokens/core.json are not reporter attachments.

Test-side corrections include actual checkbox/button roles, accessible labels, onboarding synchronization, asynchronous settings polling, protobuf default values, actual textarea values, native dialog/microphone test doubles, and a real CDP drag/drop. Schedule uses a mouse Save click because shell keyboard shortcuts consume button Space/Enter (request 6); the original memory keyboard scenario retains its intent and stays blocked. Legacy 10k coverage is now in the dedicated performance suite.

## Per-spec outcome

Paths below are relative to `apps/sparky-desktop/test-results/`. Passing tests discard traces. Fixme traces come from the **executed, unquarantined** `functional` run. `final/results.json` and `final/html/` contain the final 14-pass/6-fixme report. To rerun blocked cases after product fixes, set `SPARKY_E2E_RECHECK_GAPS=1`.

| Spec | Final cases | Evidence / blocker |
| --- | --- | --- |
| shell | 1 PASS | Fresh onboarding, navigation, Commander quick add, real core, detail panel. |
| memories | 1 FIXME | Request 6 (keyboard interception), also request 3 (rows); `functional/artifacts/e2e-memories-keyboard-crea-afed5-in-complete-undo-and-delete/trace.zip`. |
| minds | 1 FIXME | Request 5 (route renders collection instead of editor); `functional/artifacts/e2e-minds-create-rename-an-1802f-d-with-cascade-confirmation/trace.zip`. |
| schedule | 1 PASS | Daily schedule, occurrence preview, snooze persistence with notification stub. |
| attachments | 3 PASS | 20 mixed files, preview paths, real granted-path drop, denied microphone copy. |
| focus | 1 PASS | Start, pause, skip, reset, Commander Start focus. OS delivery remains manual. |
| calendar | 1 FIXME | Request 4: slot creates memory but does not open details; `functional/artifacts/e2e-calendar-create-a-memo-f88a6-king-an-empty-calendar-slot/trace.zip`. |
| review | 1 PASS | Range, summary, heatmap, chart tables. |
| map | 1 FIXME | Request 2: list limit 1000 violates contract max 500; `functional/artifacts/e2e-map-map-offline-tiles--58caf-est-and-keep-the-place-list/trace.zip`. Offline tile assertions cannot finish. |
| settings | 1 PASS | Persistent settings, MCP off, real HTTP token rotation, honest FTS-only copy. |
| search | 1 PASS | Commander navigation, empty state, query, FTS5 copy. |
| mind-cascade (added) | 1 PASS | Real bridge deletion of parent/child; affected memory moves to Inbox. Does not replace the blocked UI editor scenario. |
| agent-writes (added) | 2 FIXME | Both SDK writes succeed and live count changes; title rows do not render (request 3). `functional/artifacts/e2e-agent-writes-agent-std-ddc74-invalidates-the-open-window/trace.zip` and `e2e-agent-writes-agent-htt-86914-invalidates-the-open-window/trace.zip` under the same artifacts directory. |
| conflict (added) | 1 PASS | Real stale revision, draft retention, Reload/Overwrite controls and Reload discards stale draft. |
| lifecycle (added) | 2 PASS | SIGKILL core, new PID, persisted data and window recovery; last-window close leaves authenticated core health alive. |
| dst (added) | 1 PASS | Daily 09:00 America/New_York on March 7–9, 2026 remains 09:00; UTC changes from 14:00 to 13:00. |
| live/reminders | 1 FAIL | Real addon, authorization failure before arming; request 7. `live/artifacts/live-reminders-real-native-b34f7-lectron-and-the-core-closed/trace.zip`. |
| perf/desktop | 2 FAIL | Startup proxy above limit; 10k empty virtualized list blocks scroll measurement. Traces listed below. |

## Performance

Machine: Apple M1 Pro, 16 GiB, arm64, macOS 26.6.2 (25G83). Node 26.8.2, pnpm 12.6.0, Playwright 1.63.0, Electron 44.4.5. Native deployment target macOS 15+. Synthetic data only. Final core perf run had no Electron tests running; no OS cache flush was performed. No budget was relaxed.

| Measurement | Declared budget | Observed | Outcome |
| --- | --- | --- | --- |
| Core idle RSS | < 60 MB | 46.125 MiB after 5-second settle | PASS |
| Main / renderer idle RSS | No numeric budget declared | 151.484 / 109.359 MiB | Recorded |
| Core start, fresh DB, authenticated healthy | Listening < 300 ms | Final 10 samples 27.714–102.822 ms; earlier first launch **1007.903 ms**, subsequent 9 starts 29.296–35.113 ms | Final PASS; earlier MISS retained; health check is stricter than listening, disk-cache cold start unverified |
| List 10k, limit 50, 200 queries | p95 < 30 ms | Final **15.995 ms**; earlier 18.230 ms | PASS |
| FTS 10k, broad query, 50 hits, 200 queries | p95 < 30 ms | Final **26.662 ms**; earlier concurrent run **30.146 ms** | Final PASS; earlier MISS retained |
| Desktop startup proxy | FMP < 1000 ms | **1193.071 ms**, process launch to visible main/onboarding | Proxy MISS; exact Chromium FMP UNVERIFIED |
| Virtualized 10k list / FPS | Virtualized and smooth; plan has no numeric FPS threshold | DB/count 10000, **0 rendered rows** | FAIL / FPS BLOCKED; no empty-list FPS claimed |

Go p95 uses nearest-rank sample 190 of 200, no warmup exclusions. Core startup has ten separate new DB/process samples. Desktop FPS implementation records Chromium events and animation-frame intervals once a nonempty virtualized list exists; its 60 Hz check is an operational diagnostic, not a replacement or relaxation of the plan. It never reached scroll tracing because the row assertion failed. The Playwright failure trace still exists.

Evidence: `test-results/go-perf.txt`; `perf-final/results.json`; `perf-final/artifacts/perf-desktop-startup-and-idle-process-RSS/trace.zip`; `perf-final/artifacts/perf-desktop-10k-memories--e1bc0-nd-list-scrolling-FPS-trace/trace.zip`. Early core violations above are retained from the earlier command output, not overwritten with a claimed cold-start pass. Request 9 routes investigation to product owners.

## Verification and reproduction

```bash
# Prepare prints only the absolute executable on stdout; build logs go to stderr.
E2E_APP="$(bash apps/sparky-desktop/tests/scripts/prepare-e2e.sh)"
SPARKY_E2E_EXECUTABLE="$E2E_APP" SPARKY_E2E_RUN=final \
  pnpm --filter @polterware/sparky-desktop exec playwright test
SPARKY_E2E_EXECUTABLE="$E2E_APP" SPARKY_E2E_RUN=perf-final SPARKY_E2E_PERFORMANCE=1 \
  pnpm --filter @polterware/sparky-desktop exec playwright test
SPARKY_E2E_EXECUTABLE="$E2E_APP" SPARKY_E2E_RUN=live SPARKY_E2E_LIVE_NOTIFY=1 \
  pnpm --filter @polterware/sparky-desktop exec playwright test
pnpm --filter @polterware/sparky-desktop typecheck
pnpm --filter @polterware/sparky-desktop test
pnpm --filter @polterware/sparky-desktop exec tsc --noEmit -p tests/tsconfig.json
cd sparky/backend
SPARKY_PERF_CORE_BINARY="$PWD/../../dist/sparky" \
  go test -tags sparkyperf ./tests/perf -v -count=1
go test ./tests/... -race -count=1
go vet -tags sparkyperf ./tests/...
```

Final verification before the status commit: desktop typecheck PASS, test harness typecheck PASS; **21 Main/preload unit files, 129 tests PASS**; Go integration with `-race` PASS (MCP and repository suites; fixture/driver have no standalone tests); Go vet including tagged performance source PASS; tagged Go performance suite PASS; `git diff --check` PASS. Performance-only tests intentionally run without race instrumentation.

The literal requested `pnpm --filter @polterware/sparky-desktop typecheck test` was attempted and failed with TS5042 because pnpm forwards `test` as a TypeScript argument. Both scripts were then run separately and passed, including the final verification. CI, full repository regressions, and unrelated UI/design/contracts checks were not substituted for this task's checks.

## Requests and integration

| Request | Follow-up |
| --- | --- |
| [1](../requests/w3-e2e-1.md) | Exact root script/artifact-ignore delta and integrator lock regeneration. |
| [2](../requests/w3-e2e-2.md) | Map request exceeds list limit; paginate inside the product. |
| [3](../requests/w3-e2e-3.md) | Virtualized rows stay empty at one and 10k memories, blocking list/MCP/FPS visibility. |
| [4](../requests/w3-e2e-4.md) | Calendar slot creates memory but detail editor does not open. |
| [5](../requests/w3-e2e-5.md) | Mind route overridden by memory collection; editor inaccessible. |
| [6](../requests/w3-e2e-6.md) | Shell collection keyboard shortcuts intercept Enter/Space on controls. |
| [7](../requests/w3-e2e-7.md) | Real native authorization fails in unsigned package; G1 remains manual. |
| [8](../requests/w3-e2e-8.md) | Exact Electron pin or native-owner qualification before dependency resolution changes. |
| [9](../requests/w3-e2e-9.md) | Investigate startup/FTS violations and measure exact FMP; keep budgets. |

An unfrozen install resolved Electron ^44.4.5 to 44.5.1, rejected by the native build equality guard. The successful package used local ignored tooling with 44.4.5; frozen current lock also resolves 44.4.5. This workaround is not a tracked dependency fix. Native compilation succeeded; no live authorization success followed.

Root `.gitignore` is pending request 1. A worktree-local Git exclude keeps `apps/sparky-desktop/test-results/` ignored now. Test tooling output stays under ignored `tests/build/`. Auto-generated lockfile changes were restored; no build output, results, node_modules, lockfile, or AGENTS changes are committed. Report/trace files are local artifacts and must be copied before worktree removal.

Polterware `format` ran once. `status`, `query cards`, `apply` and `handoff` were attempted using the available binary and rejected with `no project for this directory`. No project was initialized or another project's store selected; no Work card/store write succeeded. This mandated task status/request documentation follows the explicit Sparky workstream protocol.

## Native observations and remaining manual steps for Erick

The real unsigned addon loaded. Before authorization, `reminders.status` returned `{"planned":0,"armed":0,"permission":"not_determined"}`. Authorization rejected with `PublicIpcError: Native notification operation failed`; native log recorded `delegate-installed` and `refresh-failed` for `start`. Main masks the native error, so the precise UNUserNotificationCenter/signing cause is unknown. No permission grant, pending notification, closed-process delivery, Done, Snooze, sleep/wake or reboot success was observed. The test fails before creating its reminder; this is not a fake pass.

1. Build with the prepare command above and rerun live mode with the real addon. Save the failure trace/logs if the unsigned package still rejects authorization. Do not use `SPARKY_NOTIFY_MODULE` or a notification stub for this gate.
2. In a disposable macOS account, have the native/packaging owner compare this unsigned app against the known ad-hoc-signed identity (for example `codesign --force --deep --sign - <Sparky.app>`). Run the signed copy normally and use Sparky's Enable notifications flow; check System Settings > Notifications > Sparky and record permission. Ad-hoc signing is a manual diagnostic; it does not establish release-signing acceptance.
3. Create a synthetic one-shot reminder and a native-supported repeating schedule. Verify real status reports armed and record pending native requests. Quit Sparky, identify its own core PID from that account's data/run/core.json, terminate only that process, and observe actual Notification Center delivery at the scheduled time with both processes stopped.
4. Use Done and Snooze in Notification Center with Sparky closed. Record hidden relaunch and persisted completion/new scheduled time; reopen Sparky to confirm. The disposable account avoids relaunch into a personal/default profile when custom test launch flags are absent.
5. Start a one-minute Focus phase, minimize Sparky, and observe real completion delivery. Repeat reminders across sleep/wake and a reboot, and verify timezone/reconciliation behavior. Record exact macOS permission, signing identity, schedule, time and outcome. Do not mark G1 passed until these real behaviors succeed.

Remaining automated follow-ups: integrate requests, rebuild, rerun with `SPARKY_E2E_RECHECK_GAPS=1`, collect a meaningful 10k scroll trace and exact FMP, and run CI. The denied-microphone test does not establish a real recording permission/session pass; blocked Map does not establish tile/CSP/network acceptance. Those Wave 2 live checks remain outside the passing claims above.
