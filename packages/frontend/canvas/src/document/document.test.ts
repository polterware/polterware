import { describe, expect, it } from "vitest";
import { createEmptyDocument, validateDocument } from "./document";
import { serializeDocument, deserializeAndMigrate } from "./serialize";
import { addNode, addRelationship, updateRelationship } from "./mutations";

describe("ArchitectureDocument v1", () => {
  it("round-trips serialize", () => {
    let doc = createEmptyDocument({ systemId: "sys-local" });
    doc = addNode(doc, { name: "API", type: "api", x: 10, y: 20 });
    doc = addNode(doc, { name: "DB", type: "database", x: 200, y: 20 });
    const a = doc.nodes[0]!.id;
    const b = doc.nodes[1]!.id;
    doc = addRelationship(doc, {
      sourceNodeId: a,
      targetNodeId: b,
      type: "reads_from",
    });
    const json = serializeDocument(doc);
    const { doc: round } = deserializeAndMigrate(json, "sys-local");
    expect(round.formatVersion).toBe(1);
    expect(round.nodes).toHaveLength(2);
    expect(round.relationships).toHaveLength(1);
    expect(round.relationships[0]!.type).toBe("reads_from");
    expect(round.relationships[0]!.origin).toBe("manual");
  });

  it("round-trips focused relationship types and explicit origin", () => {
    let doc = createEmptyDocument({ systemId: "sys-local" });
    doc = addNode(doc, { name: "Service", type: "service", x: 10, y: 20 });
    doc = addNode(doc, { name: "Runtime", type: "runtime_host", x: 240, y: 20 });
    doc = addRelationship(doc, {
      sourceNodeId: doc.nodes[0]!.id,
      targetNodeId: doc.nodes[1]!.id,
      type: "deployed_on",
      origin: "discovered",
    });
    doc = addRelationship(doc, {
      sourceNodeId: doc.nodes[1]!.id,
      targetNodeId: doc.nodes[0]!.id,
      type: "connects_to",
      origin: "inferred",
      label: "Legacy custom label",
    });
    doc = updateRelationship(doc, doc.relationships[1]!.id, { type: "calls" });
    doc = updateRelationship(doc, doc.relationships[1]!.id, { type: "connects_to" });

    const { doc: round } = deserializeAndMigrate(serializeDocument(doc), "sys-local");

    expect(round.relationships[0]!.type).toBe("deployed_on");
    expect(round.relationships[0]!.origin).toBe("discovered");
    expect(round.relationships[1]!.type).toBe("connects_to");
    expect(round.relationships[1]!.origin).toBe("inferred");
    expect(round.relationships[1]!.label).toBe("Legacy custom label");
  });

  it("reads legacy relationship types without requiring origin", () => {
    let doc = createEmptyDocument({ systemId: "sys-local" });
    doc = addNode(doc, { name: "Service", type: "service", x: 10, y: 20 });
    doc = addNode(doc, { name: "Runtime", type: "runtime_host", x: 240, y: 20 });
    doc = addRelationship(doc, {
      sourceNodeId: doc.nodes[0]!.id,
      targetNodeId: doc.nodes[1]!.id,
      type: "deploys_to",
    });

    const serialized = JSON.parse(serializeDocument(doc)) as {
      relationships: Array<Record<string, unknown>>;
    };
    delete serialized.relationships[0]!.origin;

    const { doc: round } = deserializeAndMigrate(JSON.stringify(serialized), "sys-local");

    expect(round.formatVersion).toBe(1);
    expect(round.relationships[0]!.type).toBe("deploys_to");
    expect(round.relationships[0]!.origin).toBeUndefined();
  });

  it("validates duplicate ids", () => {
    const doc = createEmptyDocument({ systemId: "s" });
    doc.nodes = [
      {
        id: "n1",
        name: "A",
        type: "api",
        origin: "manual",
        state: "unknown",
        x: 0,
        y: 0,
        width: 100,
        height: 40,
        zIndex: 0,
      },
      {
        id: "n1",
        name: "B",
        type: "api",
        origin: "manual",
        state: "unknown",
        x: 0,
        y: 0,
        width: 100,
        height: 40,
        zIndex: 1,
      },
    ];
    const issues = validateDocument(doc);
    expect(issues.some((i) => i.message.includes("Duplicate"))).toBe(true);
  });
});
