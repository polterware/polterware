import { describe, expect, it, vi } from "vitest";
import { createScheduleBridge, revisionString, type ScheduleWireBridge } from "./bridge";
import { SetScheduleRequestSchema } from "@polterware/sparky-contracts";
import { fromJson, toJson } from "@bufbuild/protobuf";

describe("protobuf JSON schedule bridge", () => {
  it("sends expectedRev as a decimal string and keeps RFC 3339 timestamps", async () => {
    const set = vi.fn(async (input?: unknown) => {
      expect(input).toMatchObject({ schedule: { memoryId: "memory-1", fireAt: "2026-10-03T21:30:00.000Z" } });
      return { schedule: { memoryId: "memory-1", fireAt: "2026-10-03T21:30:00.000Z", tz: "UTC" } };
    });
    const complete = vi.fn(async (input?: unknown) => {
      expect(input).toMatchObject({ memoryId: "memory-1", occurrenceAt: "2026-10-03T21:30:00.000Z" });
      return { completion: { memoryId: "memory-1", occurrenceAt: "2026-10-03T21:30:00.000Z" } };
    });
    const bridge = createScheduleBridge({
      memories: {},
      schedule: { set, completeOccurrence: complete },
      reminders: { snooze: async () => ({ oneOffId: "one-1", fireAt: "2026-10-03T21:40:00.000Z" }) },
      settings: {},
      onChange: () => () => {},
    } as unknown as ScheduleWireBridge);
    const saved = await bridge.schedule.set({ schedule: { memoryId: "memory-1", fireAt: "2026-10-03T21:30:00.000Z", tz: "UTC" } });
    expect(saved).toMatchObject({ schedule: { fireAt: "2026-10-03T21:30:00.000Z" } });
    await bridge.schedule.completeOccurrence({ memoryId: "memory-1", occurrenceAt: "2026-10-03T21:30:00.000Z" });
    expect(set).toHaveBeenCalledOnce();
  });

  it("canonicalizes numeric expectedRev to a string before the wire", async () => {
    const get = vi.fn(async () => ({ memory: { id: "memory-1", rev: "2" } }));
    const bridge = createScheduleBridge({
      memories: { get },
      schedule: {},
      reminders: {},
      settings: {},
      onChange: () => () => {},
    } as unknown as ScheduleWireBridge);
    await bridge.memories.get({ id: "memory-1", expectedRev: 1 as never });
    expect(get).toHaveBeenCalledWith(expect.objectContaining({ expectedRev: "1" }));
    expect(typeof (get.mock.calls as unknown as { expectedRev: string }[][])[0]?.[0]?.expectedRev).toBe("string");
  });

  it("round-trips SetScheduleRequest through protobuf JSON", () => {
    const json = toJson(SetScheduleRequestSchema, fromJson(SetScheduleRequestSchema, {
      schedule: { memoryId: "memory-1", fireAt: "2026-10-03T21:30:00.000Z", tz: "UTC", freq: "daily", every: 1 },
    }));
    expect(json).toMatchObject({ schedule: { memoryId: "memory-1", freq: "daily", tz: "UTC" } });
    expect(typeof (json as { schedule: { fireAt: string } }).schedule.fireAt).toBe("string");
  });

  it("normalizes missing revisions to 0 without using numbers", () => {
    expect(revisionString(undefined)).toBe("0");
    expect(revisionString(9007199254740993n)).toBe("9007199254740993");
  });
});
