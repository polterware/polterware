# w1-native-notify-c status
- Result: **partial**
- Branch / last commit: `sparky/w1-native-notify-c`; this status commit. Continuation after `git merge sparky-next` (W1-CORE / W1-ELECTRON). Implementation commits: `1e555c3`, `30c12c4`, `19691c2`, plus the follow-up e2e/heartbeat commit. Local only; no push or tags.
- Host: macOS 26.6.2, arm64. Electron 44.4.5, ABI 149, N-API 8.

## Repeat gap (frozen contract)

`ReminderItem` v1.1 has no `repeat` field. The real `ReminderService.Plan` **omits** domain `Repeat` rows (open-ended interval-1 daily/weekly/monthly) and never serializes `rem:<id>:repeat` / `rem:<id>:wdN` as one-shots. Bounded series and `every > 1` arm as concrete one-shot occurrences (`rem:<id>:<ms>` / `one:`) and are re-armed by start, change stream, wake, timezone, delivery, and the 6 h timer. Open-ended interval-1 rules stay unarmed natively until a contract change adds Repeat to the wire. Main rejects leaked `:repeat`/`:wdN` identifiers.

## Acceptance criteria

| Criterion | Result | Evidence |
|---|---|---|
| Hook Main: delegate before ready; Plan/Done/Snooze via authenticated clients; `--background-replan` without a window; 10 s idle that resets and awaits in-flight work; re-plan on start/change/wake/timezone/delivery/6 h; last-window quit; never stop core | **Pass (code)** | `src/main/index.ts` + `reminders/controller.ts`. `window-all-closed` → `app.quit()`. Core spawn is detached/`unref`. Unit tests: 89 passed. |
| Build wiring: ABI prebuild outside asar; no fallback timer; no keep-alive Electron | **Pass** | `package.json` `build:native`; `electron-builder.yml` extraResources `native/build/notify.node` → `native/notify.node`. |
| 200 memories respect `Budget` | **Pass (RPC + unit)** | Core Plan budget 60; `normalizePlan` / `rpc.test.ts`. Live native arming not measured (permission). |
| Schedule edit re-arms within 2 s while running | **Pass (controller)** | 500 ms debounce, 1.5 s cap (`controller.test.ts`). Not measured against a granted OS session. |
| Timezone change re-plans | **Pass (wiring)** | Native `NSSystemTimeZoneDidChangeNotification` + e2e hook `emitSystemEvent`. |
| Armed reminders fire with Electron closed | **Fail (blocked)** | Module is the same UNUserNotificationCenter path. `e2e.sh` reached a real temp core (`db=/tmp/sparky-w1-native-notify-c.*/db/sparky.db`) then `FAIL: notification permission is not_determined; native did not arm`. |
| Complete from notification updates open UI live | **Fail (blocked)** | Same TCC gate; action path applies `CompleteOccurrence` then refreshes (`controller.test.ts`). |
| No secrets or note content beyond the 140-char body in logs | **Pass** | `log.test.ts`; `notify.log` projects `{at,event,reason,added,removed,armed,elapsedMs}` only. |

## Verification

- `pnpm --filter @polterware/sparky-desktop typecheck` → **pass**
- `pnpm --filter @polterware/sparky-desktop test` → **pass**, 14 files / 89 tests
- `bash apps/sparky-desktop/native/scripts/e2e.sh` → **fail**: real core + temp DB + temp `--user-data-dir`; `Health.dbPath` matched the temp database; `notify.log` had `delegate-installed` then `refresh-failed` (`start`/`change`); core `ReminderService.Status.permission=not_determined`. No `~/Library/Application Support/Sparky` writes.
- `git diff --check` → **pass**

## Deviations from the plan (and why)

- Unique-bundle copied `.app` (`com.polterware.sparky.e2e-w1`) never entered TCC (`tccutil` OSStatus -10814). This agent session cannot click the macOS Allow prompt (no Accessibility; Cursor sits in front). E2E therefore launches the pinned Electron 44.4.5 binary with a temp app dir, temp `--user-data-dir`, and temp `SPARKY_CORE_DB`. Production remains `appId: com.polterware.sparky` with the addon outside asar.
- Heartbeat maps native camelCase (`notDetermined`) to the core token `not_determined`. `provisional` is reported as `authorized` because the core switch has no provisional case.
- E2E CJS trampoline (`e2e-main.cjs`) loads ESM Main inside the unpacked app (Chromium does not boot `"type":"module"` Main in a copied default_app).

## Requests filed

None new. Existing `w1-native-notify-1.md` (repeat on the wire) remains the contract follow-up for the gap above.

## Follow-ups for other WS / integrator

- Re-run `bash apps/sparky-desktop/native/scripts/e2e.sh` after granting Notifications to the e2e Electron identity (or a signed Sparky.app). Then confirm closed-Electron delivery and notification Done/Snooze with the UI open.
- Integrator: merge this branch after W1-ELECTRON; do not treat native Repeat as shipped.
