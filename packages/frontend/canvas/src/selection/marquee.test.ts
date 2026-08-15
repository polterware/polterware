import { describe, expect, it } from "vitest";
import { createEmptyDocument } from "../document/document";
import { addNode } from "../document/mutations";
import { marqueeModeFromDrag, selectMarquee } from "./selection";

describe("marquee", () => {
  it("detects contain vs intersect mode from drag direction", () => {
    expect(marqueeModeFromDrag(0, 100)).toBe("contain");
    expect(marqueeModeFromDrag(100, 0)).toBe("intersect");
  });

  it("selects fully contained nodes in contain mode", () => {
    let doc = createEmptyDocument({ systemId: "s" });
    doc = addNode(doc, { name: "A", type: "api", x: 10, y: 10, width: 50, height: 40, id: "a" });
    doc = addNode(doc, { name: "B", type: "api", x: 200, y: 200, width: 50, height: 40, id: "b" });
    const sel = selectMarquee(doc, { x: 0, y: 0, width: 100, height: 100 }, "contain");
    expect(sel.nodeIds).toEqual(["a"]);
  });

  it("selects intersecting in intersect mode", () => {
    let doc = createEmptyDocument({ systemId: "s" });
    doc = addNode(doc, { name: "A", type: "api", x: 80, y: 80, width: 50, height: 40, id: "a" });
    const sel = selectMarquee(doc, { x: 0, y: 0, width: 100, height: 100 }, "intersect");
    expect(sel.nodeIds).toContain("a");
  });
});
