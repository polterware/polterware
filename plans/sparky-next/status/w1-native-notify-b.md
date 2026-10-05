# w1-native-notify-b status
- Result: **partial**
- Branch / last commit: `sparky/w1-native-notify-b`; this status commit. Continuation of `sparky/w1-native-notify` after `git merge sparky-next` (domain, schedule, sqlite). Implementation commits: `520cbd6`, `ef61e61`, `47f64b7`, `41763dd`. Local only; no push or tags.
- Host: macOS 26.6.2, arm64. Electron 44.4.5, ABI 149, N-API 10 (addon targets NAPI_VERSION=8).

## Acceptance criteria

| Criterion | Result | Evidence |
|---|---|---|
| W0-NATIVE-NOTIFY list against the **real** core | **Fail (blocked)** | `bash apps/sparky-desktop/native/scripts/e2e.sh` exit 2: `sparky 0.0.0-dev: status (stub)`. W1-CORE / W1-ELECTRON not merged. See request 2. |
| 200 memories respect `Budget` | **Pass (delivery boundary)** | `normalizePlan` accepts 60 and throws on 200 (`plan.test.ts`). Planner ceiling is 60 after the schedule merge. |
| Complete from notification updates open UI live | **Fail (blocked)** | Action applies `CompleteOccurrence` then refreshes (`controller.test.ts`). No production Main/renderer Watch path in this worktree. |
| Schedule edit re-arms within 2 s while Sparky runs | **Pass (controller)** | 500 ms debounce, 1.5 s max wait under sustained writes, serialized Plan/apply. Not measured against a live window. |
| Timezone change re-plans | **Pass (wiring)** | Native `NSSystemTimeZoneDidChangeNotification` → controller refresh. Injected in unit tests. No system timezone change performed. |
| Armed reminders fire with Electron closed | **Fail (blocked vs real core)** | Same UNUserNotificationCenter module as the W0 spike. Lifecycle/addon load pass; real delivery/actions need an installed Main + real core. |
| No secrets or note content beyond the 140-char body in logs | **Pass** | `log.test.ts`, `controller.test.ts` (failed snooze), `plan.test.ts` (Unicode 140). `notify.log` projects only `{at,event,reason,added,removed,armed,elapsedMs}`. |

W0 runtime items that this WS can show without the real core:

- Pinned ABI build → **pass**: `native/scripts/build.sh` → Electron 44.4.5, N-API 8, arm64, macOS 15+.
- Addon lifecycle (delegate before ready, TSFN abort, close/reopen) → **pass**: `native/scripts/lifecycle.sh` → `{"passed":true,"cycles":26,"electron":"44.4.5","abi":"149","authorization":"notDetermined"}`.
- Calendar occurrence resolution (delivery date, all-day, DST, hourly/minutely, bounds) → **pass**: `native/scripts/test.sh`.
- Fallback timer path → **not adopted** (unchanged from O1-final).

## Verification

- `pnpm --filter @polterware/sparky-desktop test` → **pass**, 5 files / 40 tests.
- `bash apps/sparky-desktop/native/scripts/e2e.sh` → **exit 2 (blocked)**, stub CLI, no synthetic success.
- `bash apps/sparky-desktop/native/scripts/test.sh` → **pass**.
- `bash apps/sparky-desktop/native/scripts/lifecycle.sh` → **pass** after resolving `Electron.app` without capturing install logs.
- `git diff --check sparky-next...HEAD` → **pass**.
- Ownership → **pass**: diff is `apps/sparky-desktop/native/**`, `apps/sparky-desktop/src/main/reminders/**`, `plans/sparky-next/requests/w1-native-notify-*.md`, this status. W1-ELECTRON files untouched.

## Deviations from the plan (and why)

- Real-core E2E and live UI acceptance cannot pass until W1-CORE and W1-ELECTRON land. e2e fails its preflight instead of using the W0 synthetic core.
- Frozen `ReminderItem` v1.1 has no `repeat`. After the schedule merge the planner emits `Repeat`; Main accepts planner-shaped descriptors (default `second=0`) but does not reconstruct recurrence. RPC mapping waits on request 1.
- Last-window-quit and production `installReminders` call sites are W1-ELECTRON (request 2). This WS exposes `installReminders` / `isBackgroundReminderLaunch` / `prepareLaunch` as the thin Main hooks.
- Notification-relevant change hints are not on the frozen Change message (request 3). Debounced entity-filtered refresh + content hash is the bounded v1.1 behavior.
- Lifecycle used a throwaway bundle id (`com.polterware.sparky.native-lifecycle`); no production permission prompt and no Notification Center click were performed in this continuation.

## Requests filed

- `plans/sparky-next/requests/w1-native-notify-1.md` — aligned with merged `schedule.DateComponents` (`ReminderDateComponents` field 8).
- `plans/sparky-next/requests/w1-native-notify-2.md` — Main wiring, builder extraResources, core hidden relaunch.
- `plans/sparky-next/requests/w1-native-notify-3.md` — optional notification-relevance hint.

## Follow-ups for other WS

- W0-CONTRACTS / integrator: apply request 1; W1-CORE must map domain `Repeat` and must not serialize a repeating item as a one-shot.
- W1-ELECTRON: call `installReminders` before `app.whenReady()`, last-window-quit, Watch → `onChanges`, prepend `build:native`.
- W1-CORE: Health/Memory/Schedule/Reminder RPCs + coalesced `--background-replan` after notification-relevant writes.
- Integrator: rerun `native/scripts/e2e.sh` after those land.
