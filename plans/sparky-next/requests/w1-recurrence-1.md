# W1-RECURRENCE: native-repeat transport missing from frozen contracts

- Owner: W0-CONTRACTS / integrator, with Erick's approval for a freeze change.
- Domain sub-task: **done** on `sparky/w1-recurrence-b`. `domains/schedule` emits `ReminderItem.Repeat *DateComponents` for open-ended interval=1 minutely/hourly/daily/weekly (one slot per weekday). Bounded rules, every>1, monthly and yearly stay one-shots. IDs: `rem:<memoryID>:repeat` and `rem:<memoryID>:wd<n>` (Sunday=1).
- Remaining: frozen RPC cannot transport Repeat. At `sparky-contracts-v1.1`, `sparky/schemas/sparky/v1/reminder.proto` has only fields 1..7. W1-CORE must not serialize a repeating item as a concrete one-shot (that would lose indefinite OS delivery).
- Requested exact schema addition (and regeneration by the owner):

```diff
 message ReminderItem {
   string category = 7;
+  ReminderDateComponents repeat = 8;
 }
+message ReminderDateComponents {
+  string tz = 1;
+  optional int32 weekday = 2; // Native convention: Sunday=1 .. Saturday=7.
+  optional int32 hour = 3;
+  optional int32 minute = 4;
+  optional int32 second = 5;
+}
```

- Owner should add the corresponding native transport mapping in W1-CORE/W1-NATIVE-NOTIFY, then rerun `buf breaking` against the approved baseline. No frozen files changed by W1-RECURRENCE.
- Done of a repeating delivery must resolve the occurrence instant from the fire time (a stored `OccurrenceAt` cannot identify future deliveries indefinitely).
