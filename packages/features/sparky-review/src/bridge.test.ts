import { describe, expect, it, vi } from "vitest";
import { GetReviewRequestSchema } from "@polterware/sparky-contracts";
import { fromJson, toJson } from "@bufbuild/protobuf";
import { createReviewBridge, type ReviewWireBridge } from "./bridge";

describe("protobuf JSON review bridge", () => {
  it("sends RFC 3339 timestamps and the IANA timezone", async () => {
    const get = vi.fn(async (input?: unknown) => {
      expect(input).toMatchObject({ from: "2026-09-28T00:00:00.000Z", to: "2026-10-04T12:00:00.001Z", tz: "UTC" });
      return {
        from: "2026-09-28T00:00:00.000Z",
        to: "2026-10-04T12:00:00.001Z",
        tz: "UTC",
        streak: 4,
        completedTotal: 12,
        focusMinutes: 2,
        streakMood: "hopeful",
        completedMood: "hopeful",
        days: [],
        contribution: [],
      };
    });
    const bridge = createReviewBridge({
      review: { get },
      settings: { get: async () => ({ settings: {} }) },
      onChange: () => () => {},
    } as unknown as ReviewWireBridge);
    const result = await bridge.review.get({ from: "2026-09-28T00:00:00.000Z", to: "2026-10-04T12:00:00.001Z", tz: "UTC" });
    expect(result.streak).toBe(4);
    expect(result.completedTotal).toBe(12);
    expect(get).toHaveBeenCalledOnce();
  });

  it("round-trips GetReviewRequest through protobuf JSON", () => {
    const json = toJson(GetReviewRequestSchema, fromJson(GetReviewRequestSchema, {
      from: "2026-10-01T03:00:00.000Z",
      to: "2026-10-04T03:00:00.000Z",
      tz: "America/Sao_Paulo",
    }));
    expect(json).toMatchObject({ tz: "America/Sao_Paulo" });
    expect(String((json as { from: string }).from)).toContain("2026-10-01T03:00:00");
  });
});
