# W1-RECURRENCE: native-repeat transport missing from frozen contracts

- Owner: W0-CONTRACTS / integrator, with Erick's approval for a freeze change.
- Blocked sub-task: emitting native repeating reminders; concrete planning and recurrence continue independently.
- Evidence: `03-recurrence-and-reminders.md` requires `ReminderItem.Repeat *DateComponents`; at `sparky-contracts-v1.1`, `sparky/schemas/sparky/v1/reminder.proto` has only fields 1..7. Neither the domain reminder item nor the generated RPC can represent repetition. Silently emitting repeats would either lose them on the wire or change the frozen contract.
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
- Resume domain sub-task after approval: repeat support for open-ended interval=1 daily/weekly/hourly/minutely; one slot per weekday; bounded and interval>1 remain one-shots. Stable repeating identifiers and exact occurrence resolution for Done must be agreed with the native owner (a fixed occurrence timestamp cannot identify future deliveries indefinitely).
