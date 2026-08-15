import { describe, expect, it } from "vitest";
import { createEmptyDocument } from "./document";
import { addNode } from "./mutations";
import { alignNodes, distributeNodes, reverseRelationship } from "./align";
import { addRelationship } from "./mutations";

describe("align / distribute / reverse", () => {
  it("aligns left", () => {
    let doc = createEmptyDocument({ systemId: "s" });
    doc = addNode(doc, { name: "A", type: "api", x: 100, y: 0 });
    doc = addNode(doc, { name: "B", type: "api", x: 200, y: 50 });
    const ids = doc.nodes.map((n) => n.id);
    doc = alignNodes(doc, ids, "left");
    expect(doc.nodes.every((n) => n.x === 100)).toBe(true);
  });

  it("distributes horizontal", () => {
    let doc = createEmptyDocument({ systemId: "s" });
    doc = addNode(doc, { name: "A", type: "api", x: 0, y: 0 });
    doc = addNode(doc, { name: "B", type: "api", x: 50, y: 0 });
    doc = addNode(doc, { name: "C", type: "api", x: 100, y: 0 });
    const ids = doc.nodes.map((n) => n.id);
    doc = distributeNodes(doc, ids, "horizontal");
    const xs = doc.nodes.map((n) => n.x).sort((a, b) => a - b);
    expect(xs[1]! - xs[0]!).toBeCloseTo(xs[2]! - xs[1]!);
  });

  it("reverses relationship", () => {
    let doc = createEmptyDocument({ systemId: "s" });
    doc = addNode(doc, { name: "A", type: "api", x: 0, y: 0 });
    doc = addNode(doc, { name: "B", type: "service", x: 100, y: 0 });
    doc = addRelationship(doc, {
      sourceNodeId: doc.nodes[0]!.id,
      targetNodeId: doc.nodes[1]!.id,
      type: "calls",
    });
    const rid = doc.relationships[0]!.id;
    doc = reverseRelationship(doc, rid);
    expect(doc.relationships[0]!.sourceNodeId).toBe(doc.nodes[1]!.id);
    expect(doc.relationships[0]!.targetNodeId).toBe(doc.nodes[0]!.id);
    expect(doc.relationships[0]!.sourcePortId).toBeUndefined();
    expect(doc.relationships[0]!.targetPortId).toBeUndefined();
  });

  it("swaps existing port IDs without inventing fallbacks", () => {
    let doc = createEmptyDocument({ systemId: "s" });
    doc = addNode(doc, { name: "A", type: "api", x: 0, y: 0 });
    doc = addNode(doc, { name: "B", type: "service", x: 100, y: 0 });
    doc = addRelationship(doc, {
      sourceNodeId: doc.nodes[0]!.id,
      targetNodeId: doc.nodes[1]!.id,
      sourcePortId: "legacy-out",
      targetPortId: "legacy-in",
      type: "calls",
    });

    doc = reverseRelationship(doc, doc.relationships[0]!.id);

    expect(doc.relationships[0]!.sourcePortId).toBe("legacy-in");
    expect(doc.relationships[0]!.targetPortId).toBe("legacy-out");
  });
});
