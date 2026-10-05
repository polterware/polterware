import { describe, expect, it, vi } from "vitest";
import { createMemoriesBridge, revisionString, type MemoriesWireBridge } from "./bridge";
import { UpdateMemoryRequestSchema } from "@polterware/sparky-contracts";
import { fromJson, toJson } from "@bufbuild/protobuf";

describe("protobuf JSON memory bridge", () => {
  it("sends expectedRev as a decimal string and keeps RFC 3339 timestamps", async () => {
    const update = vi.fn(async (input?: unknown) => {
      expect(input).toMatchObject({ id: "memory-1", expectedRev: "9007199254740993", note: "Local" });
      expect(typeof (input as { expectedRev: string }).expectedRev).toBe("string");
      return { memory: { id: "memory-1", note: "Local", rev: "9007199254740994", dueAt: "2026-10-04T12:00:00.000Z" } };
    });
    const bridge = createMemoriesBridge({
      memories: { update },
      minds: {}, tags: {}, activity: {}, schedule: {},
      onChange: () => () => {},
      app: { openDeepLink: async () => {} },
    } as unknown as MemoriesWireBridge);
    const result = await bridge.memories.update({ id: "memory-1", expectedRev: "9007199254740993", note: "Local" });
    expect(result.memory?.rev).toBe("9007199254740994");
    expect(result.memory?.dueAt).toBe("2026-10-04T12:00:00.000Z");
    expect(update).toHaveBeenCalledOnce();
  });
  it("canonicalizes numeric expectedRev to a string before the wire", async () => {
    const update = vi.fn(async () => ({ memory: { id: "memory-1", rev: "2" } }));
    const bridge = createMemoriesBridge({
      memories: { update },
      minds: {}, tags: {}, activity: {}, schedule: {},
      onChange: () => () => {},
      app: { openDeepLink: async () => {} },
    } as unknown as MemoriesWireBridge);
    await bridge.memories.update({ id: "memory-1", expectedRev: 1 as never, title: "T" });
    expect(update).toHaveBeenCalled();
    const sent = (update.mock.calls as unknown as [{ expectedRev: string }][])[0]![0]!;
    expect(sent.expectedRev).toBe("1");
    expect(typeof sent.expectedRev).toBe("string");
  });
  it("round-trips UpdateMemoryRequest through protobuf JSON", () => {
    const json = toJson(UpdateMemoryRequestSchema, fromJson(UpdateMemoryRequestSchema, {
      id: "memory-1", expectedRev: "12", note: "Draft",
    }));
    expect(json).toMatchObject({ id: "memory-1", expectedRev: "12", note: "Draft" });
    expect(typeof (json as { expectedRev: string }).expectedRev).toBe("string");
  });
  it("normalizes missing revisions to 0 without using numbers", () => {
    expect(revisionString(undefined)).toBe("0");
    expect(revisionString(9007199254740993n)).toBe("9007199254740993");
    expect(revisionString("4")).toBe("4");
  });
});
