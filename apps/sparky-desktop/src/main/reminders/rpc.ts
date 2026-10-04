import { timestampDate, timestampFromDate } from "@bufbuild/protobuf/wkt";
import type { CoreClients } from "../core/client.js";
import { BUDGET, type Authorization, type NativeStatus, type ReminderCore, type Reminder } from "./types";
const leakedRepeat = /:(?:repeat|wd\d+)$/;
// Core stores snake_case permission tokens; the native module reports camelCase.
const heartbeatAuthorization: Record<Authorization, string> = {
  authorized: "authorized",
  provisional: "authorized",
  denied: "denied",
  notDetermined: "not_determined",
  unknown: "not_determined",
};
export function createReminderCore(clients: Pick<CoreClients, "reminders" | "schedule" | "settings">): ReminderCore {
  const { reminders, schedule: schedules, settings } = clients;
  const callOptions = () => ({ timeoutMs: 4000 });
  return {
    async plan(now) {
      const response = await reminders.plan({ now: timestampFromDate(now), budget: BUDGET, horizon: { seconds: 30n * 24n * 60n * 60n, nanos: 0 } }, callOptions());
      const { settings: config } = await settings.get({}, callOptions());
      return response.items.map((item): Reminder => {
        if (!item.fireAt || !item.occurrenceAt || !["reminder", "reminder-focus", "focus-phase"].includes(item.category)) throw new Error("Invalid core reminder plan");
        // Frozen ReminderItem has no repeat field. Never arm a native-repeat identifier as a one-shot.
        if (leakedRepeat.test(item.id)) throw new Error("Invalid core reminder plan");
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
    async heartbeat(status: NativeStatus) {
      await reminders.heartbeat({ authorization: heartbeatAuthorization[status.authorization] || "not_determined", armedCount: status.pendingCount }, callOptions());
    },
  };
}
