import { describe, expect, it } from "vitest";
import {
  addRelationship,
  createEmptyDocument,
  docAddNode,
  updateRelationship,
  deleteSelection,
} from "@polterware/frontend-canvas";

describe("relationships", () => {
  it("creates reconnects and deletes", () => {
    let doc = createEmptyDocument({ systemId: "s" });
    doc = docAddNode(doc, { name: "API", type: "api", x: 0, y: 0, id: "a" });
    doc = docAddNode(doc, { name: "DB", type: "database", x: 100, y: 0, id: "b" });
    doc = docAddNode(doc, { name: "Cache", type: "cache", x: 200, y: 0, id: "c" });
    doc = addRelationship(doc, {
      sourceNodeId: "a",
      targetNodeId: "b",
      type: "reads_from",
    });
    const id = doc.relationships[0]!.id;
    doc = updateRelationship(doc, id, { targetNodeId: "c", label: "cache" });
    expect(doc.relationships[0]!.targetNodeId).toBe("c");
    doc = deleteSelection(doc, {
      nodeIds: [],
      relationshipIds: [id],
      boundaryIds: [],
    });
    expect(doc.relationships).toHaveLength(0);
  });
});
