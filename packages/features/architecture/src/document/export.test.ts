import { describe, expect, it } from "vitest";
import { createEmptyDocument, docAddNode } from "@polterware/frontend-canvas";
import { exportDocumentJson, exportSelectionJson } from "./export-json";

describe("export json", () => {
  it("exports full document with formatVersion", () => {
    let doc = createEmptyDocument({ systemId: "s" });
    doc = docAddNode(doc, { name: "A", type: "api", x: 0, y: 0 });
    const json = exportDocumentJson(doc);
    const p: { formatVersion: number; nodes: object[] } = JSON.parse(json);
    expect(p.formatVersion).toBe(1);
    expect(p.nodes).toHaveLength(1);
  });

  it("exports selection fragment", () => {
    let doc = createEmptyDocument({ systemId: "s" });
    doc = docAddNode(doc, { name: "A", type: "api", x: 0, y: 0, id: "a" });
    doc = docAddNode(doc, { name: "B", type: "api", x: 10, y: 0, id: "b" });
    const json = exportSelectionJson(doc, {
      nodeIds: ["a"],
      relationshipIds: [],
      boundaryIds: [],
    });
    const p: { nodes: object[] } = JSON.parse(json);
    expect(p.nodes).toHaveLength(1);
  });
});
