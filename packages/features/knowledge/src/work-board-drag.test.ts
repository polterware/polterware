import { describe, expect, it } from "vitest";
import { columnAt, type ColumnSpan } from "./work-board-drag";

const spans: ColumnSpan[] = [
  { stage: "shape", left: 0, right: 100 },
  { stage: "plan", left: 112, right: 212 },
  { stage: "build", left: 224, right: 324 },
];

describe("columnAt", () => {
  it("returns the column containing x", () => {
    expect(columnAt(spans, 150)).toBe("plan");
  });

  it("snaps a point in the gap to the nearer column", () => {
    expect(columnAt(spans, 104)).toBe("shape");
    expect(columnAt(spans, 108)).toBe("plan");
  });

  it("clamps past either end", () => {
    expect(columnAt(spans, -40)).toBe("shape");
    expect(columnAt(spans, 900)).toBe("build");
  });

  it("returns null without columns", () => {
    expect(columnAt([], 10)).toBeNull();
  });
});
