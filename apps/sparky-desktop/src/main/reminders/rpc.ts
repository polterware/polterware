import { createClient, type Transport } from "@connectrpc/connect";
import { timestampDate, timestampFromDate } from "@bufbuild/protobuf/wkt";
import { ReminderService, ScheduleService, SettingsService } from "@polterware/sparky-contracts";
import { BUDGET, type ReminderCore, type Reminder } from "./types";
export function createReminderCore(transport: Transport): ReminderCore {
  const reminders = createClient(ReminderService, transport), schedules = createClient(ScheduleService, transport), settings = createClient(SettingsService, transport);
  const callOptions = () => ({ timeoutMs: 4000 });
  return {
    async plan(now) {
      const response = await reminders.plan({ now: timestampFromDate(now), budget: BUDGET, horizon: { seconds: 30n * 24n * 60n * 60n, nanos: 0 } }, callOptions());
      const { settings: config } = await settings.get({}, callOptions());
      return response.items.map((item): Reminder => {
        if (!item.fireAt || !item.occurrenceAt || !["reminder", "reminder-focus", "focus-phase"].includes(item.category)) throw new Error("Invalid core reminder plan");
        // Contract v1.1 has no repeat descriptor. Request 1 owns that blocked mapping.
        return { id: item.id, memoryId: item.memoryId, occurrenceAt: timestampDate(item.occurrenceAt).toISOString(), fireAt: timestampDate(item.fireAt).toISOString(), title: item.title, body: item.body,
          category: item.category as Reminder["category"], sound: config?.notificationsSound || "default" };
      });
    },
    async complete(memoryId, occurrenceAt) { await schedules.completeOccurrence({ memoryId, occurrenceAt: timestampFromDate(new Date(occurrenceAt)) }, callOptions()); },
    async snooze(memoryId, occurrenceAt) {
      const { settings: config } = await settings.get({}, callOptions());
      await reminders.snooze({ memoryId, occurrenceAt: timestampFromDate(new Date(occurrenceAt)), minutes: config?.notificationsSnoozeMinutes || 10 }, callOptions());
    },
    async ack(id, outcome) { await reminders.ack({ id, outcome }, callOptions()); },
    async heartbeat(status) { await reminders.heartbeat({ authorization: status.authorization, armedCount: status.pendingCount }, callOptions()); },
  };
}
