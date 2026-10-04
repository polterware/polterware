import { vi } from "vitest";
import type { ScheduleWireBridge } from "./bridge";
import type { ScheduleJSON } from "./mapping";

export type FakeMemory = {
  id: string;
  status?: string;
  rev?: string;
  schedule?: ScheduleJSON;
  completions?: { occurrenceAt: string; completedAt?: string }[];
};

export function fakeScheduleBridge(seed: FakeMemory = { id: "memory-1", status: "active", rev: "1" }) {
  const memory: FakeMemory = { completions: [], ...seed };
  let listener: Parameters<ScheduleWireBridge["onChange"]>[0] | undefined;
  const schedule = {
    set: vi.fn(async (input?: { schedule?: ScheduleJSON }) => {
      memory.schedule = { ...memory.schedule, ...input?.schedule, memoryId: memory.id };
      return { schedule: memory.schedule };
    }),
    clear: vi.fn(async () => {
      memory.schedule = undefined;
      return {};
    }),
    preview: vi.fn(async (input?: { schedule?: ScheduleJSON; limit?: number }) => ({
      occurrences: input?.schedule?.fireAt ? [input.schedule.fireAt] : [],
      description: "Preview unavailable in the fixture until the core responds.",
    })),
    completeOccurrence: vi.fn(async (input?: { occurrenceAt?: string }) => {
      const occurrenceAt = input?.occurrenceAt ?? "";
      memory.completions = [...(memory.completions ?? []).filter(item => item.occurrenceAt !== occurrenceAt), { occurrenceAt, completedAt: new Date().toISOString() }];
      return { completion: { memoryId: memory.id, occurrenceAt } };
    }),
    reopenOccurrence: vi.fn(async (input?: { occurrenceAt?: string }) => {
      memory.completions = (memory.completions ?? []).filter(item => item.occurrenceAt !== input?.occurrenceAt);
      return {};
    }),
    setFocusRecipe: vi.fn(async (input?: { enabled?: boolean; workMin?: number; shortBreakMin?: number; longBreakMin?: number; cycles?: number; autoContinue?: boolean }) => {
      if (!memory.schedule) throw new Error("Save a schedule before enabling a focus recipe.");
      memory.schedule = {
        ...memory.schedule,
        focusEnabled: input?.enabled,
        focusWorkMin: input?.workMin,
        focusShortBreakMin: input?.shortBreakMin,
        focusLongBreakMin: input?.longBreakMin,
        focusCycles: input?.cycles,
        focusAutoContinue: input?.autoContinue,
      };
      return { schedule: memory.schedule };
    }),
  };
  const reminders = {
    status: vi.fn(async () => ({ permission: "authorized", armed: 1, planned: 1, notifierLastSeenAt: new Date().toISOString() })),
    snooze: vi.fn(async (input?: { occurrenceAt?: string; minutes?: number }) => ({
      oneOffId: "one-1",
      fireAt: new Date(Date.parse(input?.occurrenceAt ?? Date.now().toString()) + (input?.minutes ?? 10) * 60_000).toISOString(),
    })),
  };
  const wire = {
    memories: {
      get: vi.fn(async () => ({ memory: { id: memory.id, title: "Scheduled", status: memory.status ?? "active", rev: memory.rev ?? "1", note: "", completions: memory.completions ?? [], ...(memory.schedule ? { schedule: memory.schedule } : {}) } })),
      list: vi.fn(async () => ({ memories: [], total: 0 })),
    },
    minds: { list: vi.fn(async () => ({ minds: [] })) },
    tags: { list: vi.fn(async () => ({ tags: [] })) },
    search: { search: vi.fn(async () => ({ hits: [] })) },
    schedule,
    reminders,
    settings: {
      get: vi.fn(async () => ({ settings: { notificationsSnoozeMinutes: 10, focusWorkMin: 25, focusShortBreakMin: 5, focusLongBreakMin: 15, focusCycles: 4, focusAutoContinue: true } })),
    },
    onChange: vi.fn(callback => {
      listener = callback;
      return () => { listener = undefined; };
    }),
    app: {
      onDeepLink: vi.fn(() => () => {}),
      getDeepLink: vi.fn(async () => null),
      requestNotificationAuthorization: vi.fn(async () => true),
    },
  } as unknown as ScheduleWireBridge;
  return {
    wire,
    memory,
    emit: () => listener?.({ resync: false, changes: [{ entity: "schedule", id: memory.id, op: "update" }] }),
  };
}
