import { describe, expect, it } from "vitest";
import {
  addRelationship,
  createEmptyDocument,
  docAddNode,
  updateRelationship,
} from "@polterware/frontend-canvas";
import { exportDocumentSvg } from "./export-svg";

describe("exportDocumentSvg", () => {
  it("exports only opaque colors", () => {
    let doc = createEmptyDocument({ systemId: "system" });
    doc = docAddNode(doc, { id: "api", name: "API", type: "api", x: 0, y: 0 });
    doc = docAddNode(doc, {
      id: "database",
      name: "Database",
      type: "database",
      x: 180,
      y: 0,
    });
    doc = addRelationship(doc, {
      sourceNodeId: "api",
      targetNodeId: "database",
      type: "reads_from",
      evidence: true,
    });

    const svg = exportDocumentSvg(doc);

    expect(svg).toContain('stroke="#6f6f6f"');
    expect(svg).toContain('stroke="#2c2c2c"');
    expect(svg).toContain('fill="#ececec"');
    expect(svg).toContain('marker-end="url(#arrow)"');
    expect(svg).not.toContain('marker-start="url(#arrow)"');
    expect(svg).not.toContain("stroke-dasharray");
    expect(svg).not.toMatch(/#[0-9a-f]{4}(?:\b|["'])/i);
    expect(svg).not.toMatch(/#[0-9a-f]{8}(?:\b|["'])/i);
    expect(svg).not.toMatch(/rgba?\([^)]*\/|rgba\(/i);
  });

  it("exports the same automatic vertical route used by the canvas", () => {
    let doc = createEmptyDocument({ systemId: "system" });
    doc = docAddNode(doc, {
      id: "source",
      name: "Source",
      type: "api",
      x: 0,
      y: 0,
      width: 100,
      height: 100,
    });
    doc = docAddNode(doc, {
      id: "target",
      name: "Target",
      type: "service",
      x: 0,
      y: 200,
      width: 100,
      height: 100,
    });
    doc = addRelationship(doc, {
      sourceNodeId: "source",
      targetNodeId: "target",
      type: "calls",
    });

    const svg = exportDocumentSvg(doc);

    expect(svg).toContain("M 74 124 L 74 174 L 74 224");
  });

  it("exports Q curves for orthogonal paths", () => {
    let doc = createEmptyDocument({ systemId: "system" });
    doc = docAddNode(doc, {
      id: "a",
      name: "A",
      type: "api",
      x: 0,
      y: 0,
      width: 100,
      height: 100,
    });
    doc = docAddNode(doc, {
      id: "b",
      name: "B",
      type: "service",
      x: 200,
      y: 200,
      width: 100,
      height: 100,
    });
    doc = addRelationship(doc, {
      sourceNodeId: "a",
      targetNodeId: "b",
      type: "calls",
    });

    const svg = exportDocumentSvg(doc);
    expect(svg).toContain("Q");
  });

  it("exports rounded arrows at both ends for bidirectional relationships", () => {
    let doc = createEmptyDocument({ systemId: "system" });
    doc = docAddNode(doc, { id: "a", name: "A", type: "api", x: 0, y: 0 });
    doc = docAddNode(doc, { id: "b", name: "B", type: "service", x: 240, y: 0 });
    doc = addRelationship(doc, {
      sourceNodeId: "a",
      targetNodeId: "b",
      type: "calls",
    });
    doc = updateRelationship(doc, doc.relationships[0]!.id, {
      direction: "bidirectional",
    });

    const svg = exportDocumentSvg(doc);

    expect(svg).toContain('stroke-linecap="round"');
    expect(svg).toContain('stroke-linejoin="round"');
    expect(svg).toContain('marker-start="url(#arrow)"');
    expect(svg).toContain('marker-end="url(#arrow)"');
  });
});
