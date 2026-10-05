import { describe, expect, it, vi } from "vitest";
import { createMapBridge, revisionString, type MapWireBridge } from "./bridge";
import { SetPlaceRequestSchema } from "@polterware/sparky-contracts";
import { fromJson } from "@bufbuild/protobuf";

describe("protobuf JSON place bridge", () => {
  it("sends expectedRev as a decimal string on Set and Clear", async () => {
    const set = vi.fn(async (input?: unknown) => {
      expect(input).toMatchObject({ expectedRev: "9007199254740993", place: { memoryId: "memory-1", lat: -23.55, lng: -46.63 } });
      expect(typeof (input as { expectedRev: string }).expectedRev).toBe("string");
      return { place: { memoryId: "memory-1", lat: -23.55, lng: -46.63, event: "enter", updatedAt: "2026-10-04T12:00:00.000Z" } };
    });
    const clear = vi.fn(async (input?: unknown) => {
      expect(input).toMatchObject({ memoryId: "memory-1", expectedRev: "4" });
      expect(typeof (input as { expectedRev: string }).expectedRev).toBe("string");
      return {};
    });
    const bridge = createMapBridge({
      places: { set, clear, listInBounds: async () => ({ places: [] }) },
      memories: { get: async () => ({ memory: { id: "memory-1", rev: "1" } }) },
      settings: { get: async () => ({ settings: { mapOnline: false } }) },
      onChange: () => () => {},
    } as unknown as MapWireBridge);
    await bridge.places.set({ expectedRev: "9007199254740993", place: { memoryId: "memory-1", lat: -23.55, lng: -46.63, event: "enter" } } as never);
    await bridge.places.clear({ memoryId: "memory-1", expectedRev: "4" } as never);
    expect(set).toHaveBeenCalledOnce();
    expect(clear).toHaveBeenCalledOnce();
  });

  it("canonicalizes numeric expectedRev before the wire", async () => {
    const set = vi.fn(async () => ({ place: { memoryId: "memory-1" } }));
    const bridge = createMapBridge({
      places: { set, clear: async () => ({}), listInBounds: async () => ({ places: [] }) },
      memories: {},
      settings: {},
      onChange: () => () => {},
    } as unknown as MapWireBridge);
    await bridge.places.set({ expectedRev: 1 as never, place: { memoryId: "memory-1", lat: 0, lng: 0 } } as never);
    const sent = (set.mock.calls as unknown as [{ expectedRev: string }][])[0]![0]!;
    expect(sent.expectedRev).toBe("1");
  });

  it("accepts a Place payload through protobuf JSON", () => {
    const json = fromJson(SetPlaceRequestSchema, { place: { memoryId: "memory-1", lat: -23.55, lng: -46.63, event: "enter" } }, { ignoreUnknownFields: true });
    expect(json.place?.memoryId).toBe("memory-1");
  });

  it("normalizes missing revisions to 0", () => {
    expect(revisionString(undefined)).toBe("0");
    expect(revisionString(9007199254740993n)).toBe("9007199254740993");
  });
});
