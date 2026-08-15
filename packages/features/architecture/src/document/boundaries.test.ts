import { describe, expect, it } from "vitest";
import { createEmptyDocument, docAddNode } from "@polterware/frontend-canvas";
import { createBoundaryFromSelection, setBoundaryCollapsed } from "./boundaries";

describe("boundaries", () => {
  it("assigns membership via boundaryId", () => {
    let doc = createEmptyDocument({ systemId: "s" });
    doc = docAddNode(doc, { name: "A", type: "api", x: 0, y: 0, id: "a" });
    doc = docAddNode(doc, { name: "B", type: "api", x: 100, y: 0, id: "b" });
    doc = createBoundaryFromSelection(doc, ["a", "b"], "Domain");
    expect(doc.boundaries).toHaveLength(1);
    expect(doc.nodes.every((n) => n.boundaryId === doc.boundaries[0]!.id)).toBe(true);
  });

  it("collapses boundary", () => {
    let doc = createEmptyDocument({ systemId: "s" });
    doc = docAddNode(doc, { name: "A", type: "api", x: 0, y: 0, id: "a" });
    doc = createBoundaryFromSelection(doc, ["a"]);
    const id = doc.boundaries[0]!.id;
    doc = setBoundaryCollapsed(doc, id, true);
    expect(doc.boundaries[0]!.collapsed).toBe(true);
  });
});
