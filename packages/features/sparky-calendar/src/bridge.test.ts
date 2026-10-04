import { describe, expect, it, vi } from "vitest";
import { ListOccurrencesRequestSchema, ListOccurrencesResponseSchema, SettingsSchema, SetScheduleRequestSchema } from "@polterware/sparky-contracts";
import { fromJson, toJson } from "@bufbuild/protobuf";
import { createCalendarBridge, revisionString, type CalendarWireBridge } from "./bridge";

describe("protobuf JSON calendar bridge", () => {
  it("sends expectedRev as a decimal string and keeps RFC 3339 timestamps", async () => {
    const set = vi.fn(async (input?: unknown) => {
      expect(input).toMatchObject({
        expectedRev: "2",
        schedule: { memoryId: "memory-1", fireAt: "2026-10-03T21:30:00.000Z" },
      });
      expect(typeof (input as { expectedRev: string }).expectedRev).toBe("string");
      return { schedule: { memoryId: "memory-1", fireAt: "2026-10-03T21:30:00.000Z", tz: "UTC" } };
    });
    const list = vi.fn(async (input?: unknown) => {
      expect(input).toMatchObject({ from: "2026-10-01T00:00:00.000Z", to: "2026-11-01T00:00:00.000Z" });
      return { occurrences: [{ memoryId: "memory-1", title: "Read", occurrenceAt: "2026-10-03T21:30:00.000Z" }] };
    });
    const bridge = createCalendarBridge({
      memories: {},
      minds: {},
      schedule: { set, listOccurrences: list },
      settings: {},
      onChange: () => () => {},
    } as unknown as CalendarWireBridge);
    await bridge.schedule.set({ schedule: { memoryId: "memory-1", fireAt: "2026-10-03T21:30:00.000Z", tz: "UTC" }, expectedRev: 2 as never });
    const listed = await bridge.schedule.listOccurrences({ from: "2026-10-01T00:00:00.000Z", to: "2026-11-01T00:00:00.000Z" });
    expect(listed).toMatchObject({ occurrences: [{ occurrenceAt: "2026-10-03T21:30:00.000Z" }] });
    expect(set).toHaveBeenCalledOnce();
  });

  it("round-trips ListOccurrences and Set through protobuf JSON", () => {
    const list = toJson(ListOccurrencesRequestSchema, fromJson(ListOccurrencesRequestSchema, {
      from: "2026-10-01T00:00:00.000Z",
      to: "2026-11-01T00:00:00.000Z",
      includeCompleted: true,
    }));
    expect(typeof (list as { from: string }).from).toBe("string");
    const set = toJson(SetScheduleRequestSchema, fromJson(SetScheduleRequestSchema, {
      schedule: { memoryId: "memory-1", fireAt: "2026-10-03T21:30:00.000Z", tz: "UTC" },
    }));
    expect(typeof (set as { schedule: { fireAt: string } }).schedule.fireAt).toBe("string");
  });

  it("normalizes missing revisions to 0 without using numbers", () => {
    expect(revisionString(undefined)).toBe("0");
    expect(revisionString(9007199254740993n)).toBe("9007199254740993");
  });
  it("round-trips calendarWeekStart and occurrence.tz as declared contract fields", () => {
    expect(toJson(SettingsSchema, fromJson(SettingsSchema, { calendarWeekStart: "sun" }))).toMatchObject({ calendarWeekStart: "sun" });
    const json = toJson(ListOccurrencesResponseSchema, fromJson(ListOccurrencesResponseSchema, { occurrences: [{ memoryId: "memory-1", occurrenceAt: "2026-10-04T01:00:00.000Z", tz: "America/Sao_Paulo" }] }));
    expect(json).toMatchObject({ occurrences: [{ tz: "America/Sao_Paulo" }] });
  });

});
