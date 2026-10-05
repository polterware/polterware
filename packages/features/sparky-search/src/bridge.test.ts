import { describe, expect, it, vi } from "vitest";
import { SearchMemoriesRequestSchema } from "@polterware/sparky-contracts";
import { fromJson, toJson } from "@bufbuild/protobuf";
import { createSearchBridge, type SearchWireBridge } from "./bridge";

describe("protobuf JSON search bridge", () => {
  it("sends RFC 3339 date filters and validates SearchService.Search output", async () => {
    const search = vi.fn(async (input?: unknown) => {
      expect(input).toMatchObject({
        query: "milk", mindId: "mind-1", includeDescendants: true, tagIds: ["tag-1"], status: "active", limit: 50,
        dueFrom: "2026-10-04T00:00:00.000Z", dueTo: "2026-10-04T23:59:59.999Z",
      });
      return { hits: [{ memoryId: "memory-milk", title: "Buy milk", score: 0.1, snippet: "Get <b>milk</b>", matchedIn: ["note"] }], notice: "" };
    });
    const bridge = createSearchBridge({
      search: { search, status: async () => ({ ftsDocuments: 3 }) },
      minds: { list: async () => ({ minds: [] }) },
      tags: { list: async () => ({ tags: [] }) },
      onChange: () => () => {},
      app: { openDeepLink: async () => {} },
    } as unknown as SearchWireBridge);
    const result = await bridge.search.search({
      query: "milk", mindId: "mind-1", includeDescendants: true, tagIds: ["tag-1"], status: "active", limit: 50,
      dueFrom: "2026-10-04T00:00:00.000Z", dueTo: "2026-10-04T23:59:59.999Z",
    });
    expect(result.hits?.[0]?.memoryId).toBe("memory-milk");
    expect(search).toHaveBeenCalledOnce();
  });

  it("round-trips SearchMemoriesRequest timestamps as RFC 3339 strings", () => {
    const json = toJson(SearchMemoriesRequestSchema, fromJson(SearchMemoriesRequestSchema, {
      query: "milk", dueFrom: "2026-10-04T00:00:00.000Z", updatedTo: "2026-10-04T23:59:59.999Z", limit: 50,
    }));
    expect(json).toMatchObject({ query: "milk", limit: 50 });
    expect(String((json as { dueFrom: string }).dueFrom)).toContain("2026-10-04");
  });
});
