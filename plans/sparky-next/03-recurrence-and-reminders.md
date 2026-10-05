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
keep the first `Budget` (**nearest-first globally**, not per memory), generating occurrences **lazily** (k-way merge, stop at `Budget`;
never expand a minutely/hourly series into thousands of dates). **Native repeating triggers** (as the Swift app does) are emitted
for open-ended rules the OS can express: daily at a time, weekly on weekdays at a time, hourly (every 1) at a minute, minutely
(every 1) — item carries `Repeat *DateComponents` and counts as one slot per weekday; such triggers never run out. Bounded
rules (until/count) and `every > 1` are armed as concrete one-shot items (≤ `Budget`). A repeating trigger cannot skip a completed
occurrence (same as Swift); `Done` completes the occurrence in the core and the series continues.

Re-plan triggers (**debounced 500 ms**; only changes to title, schedule, status or completions — not note edits beyond the 140-char body — cause a re-plan): Electron start, every change-stream message touching `memories`/`schedules`/`completions`, a delivered
notification, system wake, timezone change, and a 6-hour timer. The core also exposes `Plan` for `sparky status`/`doctor`.

## Delivery: native module inside Electron Main (decision O1, 2026-10-04)

A launchd helper does not stay alive (W0-NOTIFIER evidence), so delivery uses the same mechanism as the current Swift app: the
**Sparky app itself** schedules `UNNotificationRequest`s under its own bundle identity; the OS holds and delivers them with every
Sparky process closed. Electron has no scheduling API, so Main loads a small **native module** (`apps/sparky-desktop/native/notify`).

Module API (JS): `requestAuthorization()`, `apply(plan)` (diff: add/replace/remove by identifier + content hash), `pending()`,
`removeAll(prefix)`, `onAction(cb)`, `status()` → `{authorization, pendingCount}`. Implementation form decided by spike
W0-NATIVE-NOTIFY: Objective-C++ N-API addon (node-addon-api, prebuilt for the pinned Electron ABI) **or** a Swift dynamic library
with a C ABI loaded through `koffi`. The existing Swift spike code under `apps/sparky-notifier` is the starting point.

Behavior
1. Plan: Main calls `ReminderService.Plan` (same RPC), diffs against `pending()` by identifier (`rem:`/`one:`) **and** content hash
   (`title|body|fireAt|category|repeat`), adds/replaces/removes. Never touches foreign identifiers.
2. Categories/actions: `reminder` → **Done** (`CompleteOccurrence`), **Snooze** (`Snooze` RPC), **Open** (default → `sparky://memory/<id>`);
   `reminder-focus` adds **Start focus**; `focus-phase`: no actions (armed through `ArmOneOff`, cancelled with `CancelOneOff`).
3. Authorization: requested in onboarding from Sparky's identity (must go through LaunchServices-started app, not a raw exec);
   denied → status `permission denied`, nothing is armed, already-armed ones stay with the OS.
4. **Actions with Sparky closed**: the OS relaunches Sparky in the background; Main registers the notification delegate as the first
   thing it does, runs `ensureCore`, applies `Complete`/`Snooze`, re-plans, and quits if no window was opened (idle 10 s). Target:
   action applied in < 1.5 s after click on an M-series Mac (measured in the spike; if not met, document and show it honestly).
5. **Re-plan triggers**: Electron start, change stream while running, system wake, timezone change, delivered notification, 6-hour
   timer while running, and **core writes while Electron is closed**: when a schedule/status write arrives from MCP or the CLI and no
   Sparky window/Main is running, the core launches `open -g -j -a Sparky --args --background-replan` (hidden), Main re-plans and quits.
   If the spike shows this hidden relaunch is not viable, reminders armed from agent writes will be refreshed the next time Sparky
   runs, and the UI says so.
6. Honest status (UI chip, `ReminderService.Status`): `armed N · refreshed <time>` / `permission denied` / `refresh pending (opens with Sparky)`.
7. Sound: `notifications.sound` maps to bundled sound files in the app bundle (W1-ASSETS).
8. Logs: `<data>/run/notify.log`.

Limits and known behavior: pending cap on macOS 26.6.2 accepted ≥ 100 (spike); `Budget` stays 60. Minimum macOS 15. Works ad-hoc signed
(the Swift app already ships this way); Developer ID is **optional** and would allow a future always-alive helper that re-arms without
Sparky running.

## Fallback

If the native module cannot schedule reliably under Electron (spike W0-NATIVE-NOTIFY fails): Electron Main posts notifications from a
core-driven timer while it runs in the background, and the UI states "Reminders need Sparky running". The core planner and RPC
surface stay identical, so only the delivery adapter changes.
