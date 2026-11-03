import { describe, expect, it } from "vitest";
import { createEmptyDocument } from "../document/document";
import { addNode, addRelationship } from "../document/mutations";
import { buildFragment, pasteFragment } from "./clipboard";

describe("clipboard", () => {
  it("rewrites ids and keeps only internal relationships", () => {
    let doc = createEmptyDocument({ systemId: "s" });
    doc = addNode(doc, { name: "A", type: "api", x: 0, y: 0, id: "a" });
    doc = addNode(doc, { name: "B", type: "database", x: 100, y: 0, id: "b" });
    doc = addNode(doc, { name: "C", type: "service", x: 200, y: 0, id: "c" });
    doc = addRelationship(doc, { sourceNodeId: "a", targetNodeId: "b", type: "reads_from" });
    doc = addRelationship(doc, { sourceNodeId: "b", targetNodeId: "c", type: "calls" });

    const frag = buildFragment(doc, {
      nodeIds: ["a", "b"],
      relationshipIds: [],
      boundaryIds: [],
    });
    expect(frag.nodes).toHaveLength(2);
    expect(frag.relationships).toHaveLength(1);

    const { doc: next, newSelection } = pasteFragment(doc, frag, { x: 20, y: 20 });
    expect(next.nodes).toHaveLength(5);
    expect(newSelection.nodeIds).toHaveLength(2);
    expect(newSelection.nodeIds.every((id) => id !== "a" && id !== "b")).toBe(true);
    const newRels = next.relationships.filter((r) => newSelection.relationshipIds.includes(r.id));
    expect(newRels).toHaveLength(1);
    expect(newSelection.nodeIds).toContain(newRels[0]!.sourceNodeId);
    expect(newSelection.nodeIds).toContain(newRels[0]!.targetNodeId);
  });
});
