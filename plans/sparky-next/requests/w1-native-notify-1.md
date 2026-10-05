# W1-NATIVE-NOTIFY request 1 — repeating plan contract

- Owner: W0-CONTRACTS / integrator, with Erick's approval required for the frozen contract.
- Blocked sub-task: receiving native repeating descriptors from the real core.
- Status after `sparky-next` merge: `domains/schedule` now emits `ReminderItem.Repeat *DateComponents` for open-ended interval-1 minutely/hourly/daily/weekly (one slot per weekday; IDs `rem:<memoryID>:repeat` and `rem:<memoryID>:wd<n>`, Sunday=1). Frozen `ReminderItem` at `sparky-contracts-v1.1` still has fields 1–7 only. Reconstructing recurrence in Main would duplicate the planner and violate the core authority.
- Native adapter (this WS): accepts the planner-shaped descriptors, defaults omitted `second` to 0, diffs by id + content hash + repeat, and resolves Done from the OS delivery date. RPC mapping stays blocked until the wire field exists. W1-CORE must not serialize a repeating item as a concrete one-shot.
- Proposed exact schema addition (leave fields 1–7 unchanged; matches merged `schedule.DateComponents`, no month/day):

```proto
message ReminderDateComponents {
  string tz = 1;
  optional int32 weekday = 2; // Foundation: 1 Sunday ... 7 Saturday
  optional int32 hour = 3;
  optional int32 minute = 4;
  optional int32 second = 5;
}
// Inside ReminderItem:
ReminderDateComponents repeat = 8;
```

- Regenerate TS/Go, update the breaking baseline with approval, and publish the new local contract tag according to the protocol.
- Repeat actions must resolve the **delivered** occurrence, not the first planned occurrence stored in userInfo. No frozen file was changed.

**Status: applied 2026-10-04 as contract v1.4** (`ReminderItem.repeat = 8`; core and Main mappings done, see 00-decisions.md).
