---
summary: Add calendar week start to Settings and timezone on Occurrence
rationale: W2-CALENDAR must read week start from settings.get (Mon/Sun) and render all-day dates in the occurrence's own timezone. Neither field exists on the frozen v1.4 contracts.
---

Contracts are frozen at `sparky-contracts-v1.4`. W2-CALENDAR did not edit protos.

Needed (integrator + Erick OK, then `buf breaking` baseline):

1. `sparky.v1.Settings` — additive `string calendar_week_start = 23` with values `mon` | `sun` (default `mon`). W2-SETTINGS owns the Settings UI control.
2. `sparky.v1.Occurrence` — additive `string tz = 9` (IANA name from the memory's schedule). `ListOccurrences` should populate it so the calendar does not guess the civil date.

Until then the calendar:

- reads `settings.calendarWeekStart` / `weekStart` from the raw `settings.get` JSON (unknown proto fields are kept by the package bridge) and defaults to Monday
- uses `occurrence.tz` when the wire includes it, otherwise the system time zone for grid placement and slot mapping
