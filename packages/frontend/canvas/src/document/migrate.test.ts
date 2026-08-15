import { describe, expect, it } from "vitest";
import { deserializeAndMigrate, validateImportJson } from "./serialize";

describe("migrate legacy layout", () => {
  it("migrates format 0 nodes/edges", () => {
    const legacy = JSON.stringify({
      nodes: [
        { id: "s1", x: 0, y: 0, label: "Sys", kind: "system" },
        { id: "r1", x: 100, y: 100, label: "Res", kind: "resource", source: "seed" },
      ],
      edges: [{ id: "e1", from: "s1", to: "r1", kind: "contains", evidence: true }],
    });
    const { doc, migrated } = deserializeAndMigrate(legacy, "sys-local");
    expect(migrated).toBe(true);
    expect(doc.formatVersion).toBe(1);
    expect(doc.nodes).toHaveLength(2);
    expect(doc.relationships[0]!.type).toBe("contains");
    expect(doc.relationships[0]!.evidence).toBe(true);
    expect(doc.relationships[0]!.origin).toBe("imported");
  });

  it("rejects unknown future version without loading as empty silently on import", () => {
    const bad = JSON.stringify({ formatVersion: 99, nodes: [], relationships: [] });
    const result = validateImportJson(bad);
    expect(result.ok).toBe(false);
  });

  it("rejects malformed json on import", () => {
    const result = validateImportJson("{not json");
    expect(result.ok).toBe(false);
  });
});
