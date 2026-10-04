# Recurrence and reminders

Reference behavior (read-only): `/Users/erickpatrickbarcelos/codes/sparky-workspace/sparky/sparky/Model/Triggers/ScheduleConfig.swift`,
`Executors/ScheduledTriggerPlanner.swift`, `Model/Recurrence/*`. The Swift code is the **reference for intent**, not for bugs:
the divergences listed under "Intentional differences" are required.

## Schedule model

`schedules` row ([02-data-and-contracts.md](02-data-and-contracts.md)): `fire_at` (first occurrence instant), `tz`, `all_day`,
`freq` (NULL = one-shot), `every`, `weekday_mask` (weekly only; bit0 = Sunday), `end_type` (`never|until|count`), `until_at`
(inclusive), `occurrence_count`. User-visible frequencies: hourly, daily, weekly, monthly, yearly (+ a one-shot). `minutely` exists
in the model and MCP only.

## Occurrence semantics (normative)

Let `anchor` = `fire_at` converted to wall-clock in `tz`. Occurrences are a strictly increasing sequence `o₀ < o₁ < …`.

1. **One-shot**: only `fire_at`.
2. **daily / monthly / yearly**: `oₖ` = anchor + `k·every` units, computed **from the anchor each time** (never iteratively) in
   wall-clock, then resolved to an instant in `tz`. Monthly/yearly clamp to the last valid day (Jan 31 + 1 month = Feb 28/29)
   but the next occurrence is again derived from the anchor day (Jan 31 → Feb 28 → Mar 31).
3. **weekly without weekdays**: anchor + `k·every` weeks.
4. **weekly with `weekday_mask`**: occurrences fall on the selected weekdays at the anchor's time of day, only in weeks where
   `weekIndex % every == 0`, where `weekIndex` counts ISO weeks (**Monday-start**) from the week containing the anchor.
   Occurrences before the anchor instant are skipped. (`count` counts each weekday occurrence.)
5. **hourly / minutely**: elapsed-time arithmetic (`anchor + k·every·unit` in absolute time), not wall-clock.
6. **`all_day`**: occurrences are dates; time of day is ignored in the sequence. The notification time is the setting
   `notifications.allDayTime` (default 09:00) in `tz`. Calendar shows them in the all-day lane.
7. **End**: `count` limits the total number of occurrences (including `o₀`); `until` is inclusive and compared as instants.
8. **DST** (wall-clock rules only): a local time that does not exist (spring forward) resolves to the **same instant the
   skipped hour maps to, shifted forward by the gap** (02:30 → 03:30); an ambiguous local time (fall back) resolves to the
   **first** occurrence (the earlier instant).
9. Hard caps: `Between(from, to)` returns at most 10 000 occurrences (error `INVALID_ARGUMENT` beyond that); a rule is
   rejected if `every × unit` is < 1 minute or the series is infinite but the requested range exceeds 5 years.

### Intentional differences from Swift

- Swift ignores `every`, `until` and `count` for weekday-mask schedules; Sparky Next honors them.
- Swift computes some sequences iteratively (month-end drift); Sparky Next derives from the anchor.
- Swift stores `weekdayMask` with Sunday = bit 1 (`1<<weekday`); Sparky Next uses bit0 = Sunday.
- Swift treats completion per calendar day; Sparky Next keys completions by **exact occurrence instant**.

## API of the pure Go package `domains/schedule`

```go
type Rule struct{ Anchor time.Time; TZ *time.Location; AllDay bool; Freq Freq; Every int; WeekdayMask uint8; End End }
func (r Rule) Validate() error
func (r Rule) Next(after time.Time) (time.Time, bool)                 // first occurrence strictly after `after`
func (r Rule) At(from, to time.Time, limit int) ([]time.Time, error)  // occurrences in [from, to)
func (r Rule) Count(from, to time.Time) (int, error)
func Describe(r Rule, locale string) string                           // human summary: "Every 2 weeks on Mon, Wed until Dec 20"
```

No I/O, no globals, `clock` injected where "now" matters. Fuzz test: `At(from,to)` is strictly increasing and every element
satisfies `Next(prev) == element`.

## Test vectors (all must pass; store as `testdata/recurrence_vectors.json`)

Times shown in the rule's `tz`; `→` lists the occurrences (local time, UTC offset in parentheses where DST matters).

| # | Rule | Output |
|---|------|--------|
| 1 | daily, every 1, anchor 2026-03-01 09:00 America/Sao_Paulo, first 3 | 03-01, 03-02, 03-03 at 09:00 (-03:00) |
| 2 | monthly, every 1, anchor 2026-01-31 09:00 America/Sao_Paulo, count 4 | 01-31, 02-28, 03-31, 04-30 at 09:00 |
| 3 | yearly, every 1, anchor 2028-02-29 09:00 UTC, first 5 | 2028-02-29, 2029-02-28, 2030-02-28, 2031-02-28, 2032-02-29 |
| 4 | weekly Mon+Wed, every 1, anchor Mon 2026-10-05 08:00 UTC, first 4 | 10-05, 10-07, 10-12, 10-14 |
| 5 | weekly Mon+Wed, every 2, anchor Mon 2026-10-05 08:00 UTC, first 4 | 10-05, 10-07, 10-19, 10-21 |
| 6 | weekly Mon+Wed, every 1, anchor Wed 2026-10-07 08:00 UTC, first 3 | 10-07, 10-12, 10-14 (Mon 10-05 is before the anchor) |
| 7 | weekly Mon+Wed, count 3, anchor Mon 2026-10-05 08:00 UTC | 10-05, 10-07, 10-12 |
| 8 | daily, anchor 2026-03-07 02:30 America/New_York, first 3 | 03-07 02:30 EST = 07:30Z; 03-08 02:30 does not exist → 03:30 EDT = 07:30Z; 03-09 02:30 EDT = 06:30Z |
| 9 | daily, anchor 2026-10-31 01:30 America/New_York, first 3 | 10-31 01:30 EDT = 05:30Z; 11-01 01:30 **first** (EDT) = 05:30Z; 11-02 01:30 EST = 06:30Z |
| 10 | hourly, every 6, anchor 2026-10-03 22:00 America/Sao_Paulo, first 4 | 22:00, 10-04 04:00, 10:00, 16:00 (-03:00), absolute spacing 6 h |
| 11 | daily, anchor 2026-10-01 09:00 UTC, until 2026-10-03 09:00 (inclusive) | 10-01, 10-02, 10-03 |
| 12 | monthly, every 2, anchor 2026-08-31 09:00 UTC, first 4 | 08-31, 10-31, 12-31, 2027-02-28 |
| 13 | one-shot 2026-10-03 21:30 UTC, `Next(after=2026-10-03 21:30)` | none (strictly after) |
| 14 | all-day daily, anchor date 2026-10-03, tz America/Sao_Paulo, notification time 09:00 | notifications at 09:00 -03:00 each day |

Also required: golden fixtures generated from the Swift implementation for **simple** cases it handles correctly (daily,
weekly without weekdays, monthly day ≤ 28, yearly non-leap, hourly/minutely) — W1-RECURRENCE writes a throwaway Swift script
under `sparky/backend/domains/schedule/testdata/swift/` (script only, not shipped) or computes them by hand; any mismatch is
investigated and documented as an intentional difference above or fixed.

## Reminder planning (core, pure) — `domains/schedule/planner.go`

Input: all active memories with active schedules and status `active`, `now`, `Budget` (default **60**, pending-notification
cap to be confirmed by O1; treat the real limit as the hard ceiling), `Horizon` (default 30 days). Output: `[]ReminderItem`:

```go
type ReminderItem struct {
  ID           string    // "rem:<memoryID>:<occurrenceUnixMs>"
  MemoryID     string
  OccurrenceAt time.Time // the occurrence this reminder is for (completion key)
  FireAt       time.Time // when to deliver (== OccurrenceAt, or all-day time)
  Title, Body  string    // memory title; first 140 chars of the note, markdown stripped
  Category     string    // "reminder" | "reminder-focus"
}
```

Pending one-offs (`reminder_oneoffs`: focus phase ends, snoozes) are also emitted as items with ID `one:<id>` and
`Category` `focus-phase` / `reminder`; they sort with everything else. Algorithm: for every memory generate occurrences in
`[now, now+Horizon]`, skip completed occurrences, add one-offs, merge-sort by `FireAt`,
keep the first `Budget` (**nearest-first globally**, not per memory). Never emit repeating native triggers; every item is one
concrete calendar trigger. The notifier re-plans whenever it wakes, so the window slides forward.

Re-plan triggers: notifier start, every change-stream message touching `memories`/`schedules`/`completions`, a delivered
notification, system wake, timezone change, and a 6-hour timer. The core also exposes `Plan` for `sparky status`/`doctor`.

## Notifier (`apps/sparky-notifier`) behavior

1. On launch: `ensureCore` (read `core.json`, health-check; spawn the bundled `sparky serve` if missing), request
   notification authorization once, subscribe to `ChangeService.Watch`.
2. `Plan` → diff against `UNUserNotificationCenter.getPendingNotificationRequests` by identifier **and** content hash
   (`title|body|fireAt|category`); add missing, replace changed, remove stale. Identifiers start with `rem:` or `one:`; the
   helper never touches foreign identifiers.
3. Categories/actions: `reminder` → **Done** (`CompleteOccurrence`), **Snooze** (`Snooze` RPC, setting default 10 min; the core
   creates a `snooze` one-off and the next `Plan` delivers it), **Open** (default action → `sparky://memory/<id>`);
   `reminder-focus` adds **Start focus** (`sparky://memory/<id>?action=focus`). Focus phase-end notifications: category
   `focus-phase`, no actions, created by the renderer through `ArmOneOff` and cancelled with `CancelOneOff` on pause/stop.
4. Actions must work with the Electron app **closed**: they call the core directly (`Complete`/`Snooze`), then re-plan.
5. Sound: `notifications.sound` setting maps to bundled sound files in the helper bundle (see W1-ASSETS).
6. Logs to `<data>/run/notifier.log`. Reports `lastSeenAt` via `Ack`/heartbeat so the UI can show "Reminders: armed 12 · notifier running".
7. If the core is unreachable it retries with backoff and **keeps the already-armed notifications** (the OS still delivers them).

## Registration and packaging (O1)

Spike both and pick one: `SMAppService.agent(plistName:)` (launchd, `KeepAlive` restarts a crashed helper; plist in
`Contents/Library/LaunchAgents/`) vs `SMAppService.loginItem(identifier:)` (bundle in `Contents/Library/LoginItems/`).
Electron registers it through `app.setLoginItemSettings` with the matching `type`/`serviceName` (verify the API in the pinned
Electron version) or a tiny native helper if Electron cannot. Required evidence in the spike: works ad-hoc signed; survives
`kill -9`; starts at login; first-run permission prompt appears from the helper's identity (not from Electron).

## Fallback if the spike fails (decision recorded in [12-risks.md](12-risks.md))

Plan B: Electron Main posts notifications (Electron `Notification`) from a core-driven timer while the app is running in the
background (tray-less, hidden window); document honestly that reminders need Sparky running. The core planner and the RPC
surface stay identical, so only the delivery adapter changes.
