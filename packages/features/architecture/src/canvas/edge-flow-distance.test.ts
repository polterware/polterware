import { describe, expect, it } from "vitest";
import { nextEdgeFlowDistance } from "./edge-flow-distance";

describe("nextEdgeFlowDistance", () => {
  it("advances along a stable path length", () => {
    expect(nextEdgeFlowDistance(10, 100, 100, 5)).toBe(15);
  });

  it("wraps when advancing past the end", () => {
    expect(nextEdgeFlowDistance(98, 100, 100, 5)).toBe(3);
  });

  it("keeps fractional progress when the path shortens", () => {
    // 50% of 200px → 50% of 100px
    expect(nextEdgeFlowDistance(100, 200, 100, 0)).toBe(50);
  });

  it("keeps fractional progress when the path lengthens", () => {
    // 25% of 100px → 25% of 200px
    expect(nextEdgeFlowDistance(25, 100, 200, 0)).toBe(50);
  });

  it("returns 0 for non-positive next length", () => {
    expect(nextEdgeFlowDistance(40, 100, 0, 5)).toBe(0);
  });
});
