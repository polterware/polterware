import { vi } from "vitest";
import type { CalendarWireBridge } from "./bridge";
import type { ScheduleJSON } from "./mapping";

export type FakeOccurrence = {
  memoryId: string;
  title: string;
  occurrenceAt: string;
  fireAt?: string;
  completed?: boolean;
  overdue?: boolean;
  allDay?: boolean;
  mindId?: string;
  tz?: string;
};

export type FakeMemory = {
  id: string;
  title: string;
  rev: string;
  schedule?: ScheduleJSON;
};

export function fakeCalendarBridge(seed: {
  occurrences?: FakeOccurrence[];
  memories?: FakeMemory[];
  minds?: { id: string; name: string; color: string }[];
  weekStart?: "mon" | "sun";
} = {}) {
  const occurrences = [...(seed.occurrences ?? [])];
  const memories = new Map((seed.memories ?? []).map(memory => [memory.id, { ...memory }]));
  let listener: Parameters<CalendarWireBridge["onChange"]>[0] | undefined;
  const schedule = {
    listOccurrences: vi.fn(async (input?: { from?: string; to?: string; includeCompleted?: boolean }) => {
      const from = Date.parse(input?.from ?? "");
      const to = Date.parse(input?.to ?? "");
      return {
        occurrences: occurrences.filter(item => {
          const at = Date.parse(item.occurrenceAt);
          if (!Number.isFinite(from) || !Number.isFinite(to)) return true;
          return at >= from && at < to && (input?.includeCompleted || !item.completed);
        }),
      };
    }),
    set: vi.fn(async (input?: { schedule?: ScheduleJSON; expectedRev?: string }) => {
      const id = input?.schedule?.memoryId ?? "";
      const memory = memories.get(id);
      if (memory && input?.expectedRev && input.expectedRev !== memory.rev) {
        throw Object.assign(new Error("memory rev conflict"), { code: "conflict" });
      }
      if (memory) {
        memory.schedule = { ...memory.schedule, ...input?.schedule, memoryId: id };
        memory.rev = String(BigInt(memory.rev) + 1n);
      }
      return { schedule: input?.schedule };
    }),
  };
  const wire = {
    memories: {
      create: vi.fn(async (input?: { title?: string }) => {
        const memory: FakeMemory = { id: `created-${memories.size + 1}`, title: input?.title ?? "Untitled memory", rev: "1" };
        memories.set(memory.id, memory);
        return { memory };
      }),
      get: vi.fn(async (input?: { id?: string }) => {
        const memory = memories.get(input?.id ?? "");
        if (!memory) throw new Error("Memory unavailable. It may have been deleted.");
        return { memory: { ...memory, schedule: memory.schedule } };
      }),
    },
    minds: {
      list: vi.fn(async () => ({ minds: seed.minds ?? [{ id: "mind-1", name: "Home", color: "#7EA8F0" }] })),
    },
    schedule,
    settings: {
      get: vi.fn(async () => ({ settings: { calendarWeekStart: seed.weekStart ?? "mon" } })),
    },
    onChange: vi.fn(callback => {
      listener = callback;
      return () => { listener = undefined; };
    }),
  } as unknown as CalendarWireBridge;
  return {
    wire,
    schedule,
    occurrences,
    memories,
    emit: () => listener?.({ resync: false, changes: [{ entity: "schedule", id: "memory-1", op: "update" }] }),
  };
}
