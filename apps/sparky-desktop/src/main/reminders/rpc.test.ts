import { createRouterTransport } from "@connectrpc/connect";
import { timestampFromDate } from "@bufbuild/protobuf/wkt";
import { ReminderService, ScheduleService, SettingsService, type PlanRemindersRequest, type CompleteOccurrenceRequest, type SnoozeReminderRequest, type ReminderHeartbeatRequest, type AckReminderRequest } from "@polterware/sparky-contracts";
import { expect, it, vi } from "vitest";
import { createReminderCore } from "./rpc";
it("maps frozen RPCs with Budget, exact completion keys, configured snooze and heartbeat", async () => {
  const memoryId = "01ARZ3NDEKTSV4RRFFQ69G5FAV", now = new Date("2026-10-04T11:00:00.000Z"), at = timestampFromDate(now);
  const plan = vi.fn((_request: PlanRemindersRequest) => ({ items: [{ id: "one:1", memoryId, fireAt: at, occurrenceAt: at, title: "Synthetic", body: "Synthetic", category: "reminder" }] }));
  const completeOccurrence = vi.fn((_request: CompleteOccurrenceRequest) => ({})), snooze = vi.fn((_request: SnoozeReminderRequest) => ({})), heartbeat = vi.fn((_request: ReminderHeartbeatRequest) => ({})), ack = vi.fn((_request: AckReminderRequest) => ({}));
  const transport = createRouterTransport(({ service }) => {
    service(ReminderService, { plan, snooze, heartbeat, ack });
    service(ScheduleService, { completeOccurrence });
    service(SettingsService, { get: () => ({ settings: { notificationsSound: "none", notificationsSnoozeMinutes: 15 } }) });
  });
  const core = createReminderCore(transport), items = await core.plan(now);
  expect(plan.mock.calls[0]?.[0]).toMatchObject({ budget: 60, horizon: { seconds: 2592000n } });
  expect(items[0]).toMatchObject({ fireAt: now.toISOString(), occurrenceAt: now.toISOString(), sound: "none" });
  await core.complete(memoryId, now.toISOString()); await core.snooze(memoryId, now.toISOString());
  await core.heartbeat({ authorization: "authorized", pendingCount: 60 }); await core.ack("one:1", "done");
  expect(completeOccurrence.mock.calls[0]?.[0]).toMatchObject({ memoryId, occurrenceAt: at });
  expect(snooze.mock.calls[0]?.[0]).toMatchObject({ memoryId, minutes: 15, occurrenceAt: at });
  expect(heartbeat.mock.calls[0]?.[0]).toMatchObject({ authorization: "authorized", armedCount: 60 });
  expect(ack.mock.calls[0]?.[0]).toMatchObject({ id: "one:1", outcome: "done" });
});
