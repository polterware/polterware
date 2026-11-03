import { describe, expect, it } from "vitest";
import { layoutSync } from "./layout";

describe("auto layout", () => {
  it("lays out horizontally preserving pins", () => {
    const res = layoutSync({
      mode: "horizontal",
      nodes: [
        { id: "a", x: 0, y: 0, width: 100 },
        { id: "b", x: 0, y: 0, width: 100 },
        { id: "p", x: 500, y: 500, layoutPinned: true },
      ],
    });
    const p = res.nodes.find((n) => n.id === "p")!;
    expect(p.x).toBe(500);
    const a = res.nodes.find((n) => n.id === "a")!;
    const b = res.nodes.find((n) => n.id === "b")!;
    expect(b.x).toBeGreaterThan(a.x);
  });
});
