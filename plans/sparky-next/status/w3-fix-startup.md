# w3-fix-startup status
- Result: **done**
- Branch / last commit: `sparky/w3-fix-startup` (implementation commits `493a67a`, `217f791`, `c7b1794`, `2b85f45`; this status is the final verification commit). Based on `sparky-next` at `56d75c4`.
- Contracts: `sparky-contracts-v1.6` preserved. No Go, design-system, lockfile, `scripts/`, `electron-builder.yml`, shared root, or `features.ts` edits. No push or tag.

## Acceptance

| Criterion | Result / evidence |
| --- | --- |
| Main/core readiness path | **PASS.** A newly spawned child polls at 25 ms then 50 ms (`nextCorePollDelay`) with a 12 s wall timeout. Existing daemons keep the 200 ms floor. Build replacement, authenticated health, DB/PID identity unchanged. `SPARKY_STARTUP_TIMING=1` logs `electron-ready`, `core-attach`, `policy-rpc`, `window-show`. Tests: `process-supervisor.test.ts` "spawned core poll timing"; `startup-timing.test.ts`. |
| Show window earlier | **Not applied.** `installSessionSecurity` still runs before `showWindow()`/document load. Creating the window before `attachCore()` would let the renderer call IPC while `getClients()` throws, and `createWindow` auto-shows on `ready-to-show` (`window.ts` is outside Owns). CSP stays restrictive defaults until `refreshMapPolicy`. |
| Shell connection gating | **PASS.** Inbox count starts while reminder status is still in flight (`independentQueryKey`). A registered Minds `sidebarTree` skips the shell `minds.list`. Tests: "starts independent queries with the reminder probe"; "skips duplicate Mind reads when a sidebar tree is registered". |
| Feature registry lazy load | **PASS** in feature `index.tsx` files (Calendar, Review, Focus, Schedule editor, Settings panels) via `lazy(() => import(...))`. Sidebar, commands and deep links stay on the eager feature objects. `apps/sparky-desktop/src/renderer/features.ts` is unchanged (W0-SCAFFOLD shared file; metadata imports must stay eager). |
| Search deep link | **PASS.** `SearchPage` applies trimmed `q` (max 512) to the input, store, and FTS `Search`. Test that fails without the effect: `search-page.test.tsx` "applies a trimmed FTS query from sparky://search?q= and executes search". |
| Startup median < 1000 ms | **PASS** on the isolated final series. Gate unchanged. |
| Initial JS | **2894.67 KB** entry (`out/renderer/assets/index-Lua_5Maa.js`) vs **3145.33 KB** before. MapLibre remains a 1293.12 KB lazy chunk. |
| E2E / perf | Functional **25/25**. Desktop perf **2/2**. |

## Startup measurements

Standard package: `bash apps/sparky-desktop/tests/scripts/prepare-e2e.sh`. Electron 44.4.5, unsigned arm64, fresh disposable DB, notification stub. No assertion or budget change.

| Run | Launch to visible main/onboarding, ms | Median, ms |
| --- | --- | ---: |
| Before (w3-fix-memories-1) | 1094.380, 1069.366, 1065.938, 1091.828, 1328.570 | **1091.828** |
| First after this batch (includes a 3674 ms first-launch outlier) | 3674.469, 910.235, 898.531, 1037.491, 922.030 | 922.030 |
| Final, no concurrent suite/build/check | 879.780, 898.631, 887.308, 909.975, 919.762 | **898.631** |

The unchanged gate is < 1000 ms. All five final samples pass; the median is **898.631 ms**. Host is not a dedicated benchmark machine.

```bash
SPARKY_E2E_EXECUTABLE="$PWD/apps/sparky-desktop/test-results/build/mac-arm64/Sparky.app/Contents/MacOS/Sparky" \
  SPARKY_E2E_RUN=startup-final SPARKY_E2E_PERFORMANCE=1 \
  pnpm --filter @polterware/sparky-desktop exec playwright test -g 'startup and idle' --repeat-each=5
```

## Verification

- Touched packages `test` and `typecheck`: PASS (desktop 139; shell 101; search 24; calendar 46; review 70; focus 21; schedule 47; settings 38).
- `pnpm design:lint`: PASS.
- Whole-repo `pnpm typecheck`: PASS.
- `git diff --check`: PASS.
- Packaged E2E `SPARKY_E2E_RUN=w3-fix-startup-e2e`: **25/25**.
- Packaged perf `SPARKY_E2E_RUN=w3-fix-startup-perf`: **2/2** (startup 898.676 ms in that run; scroll FPS pass).
- Browser not used beyond Playwright Electron.

## Deviations

- Window is still shown after authenticated `reconnect()` + `refreshMapPolicy()`, not earlier.
- `features.ts` was not edited; lazy registration lives in the owned feature `index.tsx` files so contributions and shell contracts stay the same.

## Requests filed

None.

## Follow-ups

None.
