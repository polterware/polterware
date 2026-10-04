# W1-NATIVE-NOTIFY request 1 — repeating plan contract

- Owner: W0-CONTRACTS / integrator, with Erick's approval required for the frozen contract.
- Blocked sub-task: receiving native repeating descriptors from the real core.
- Evidence: `sparky-contracts-v1.1:sparky/schemas/sparky/v1/reminder.proto` defines ReminderItem fields 1–7 only; `domains/schedule/types.go` has no Repeat either. Doc 03 requires open-ended rules to use native calendar triggers. Reconstructing recurrence in Main would duplicate the planner and violate the core authority.
- Proposed exact schema addition (leave fields 1–7 unchanged):

```proto
message ReminderRepeat {
  string tz = 1;
  optional int32 month = 2;
  optional int32 day = 3;
  optional int32 weekday = 4; // Foundation: 1 Sunday ... 7 Saturday
  optional int32 hour = 5;
  optional int32 minute = 6;
  optional int32 second = 7;
}
// Inside ReminderItem:
ReminderRepeat repeat = 8;
```

- Add equivalent optional Go DateComponents to schedule.ReminderItem; W1-RECURRENCE must emit stable repeating identifiers and descriptors (one slot per weekday). Repeat actions must resolve the **delivered** occurrence, not the first planned occurrence stored in userInfo. The native adapter supports this by resolving the matching calendar components around the OS delivery date, bounded below by the planned occurrence.
- Regenerate TS/Go, update the breaking baseline with approval, and publish the new local contract tag according to the protocol.
- Native support and pure TS tests can proceed within Owns. No frozen file was changed; RPC repeat mapping stops until approval.
