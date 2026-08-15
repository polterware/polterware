import { describe, expect, it } from "vitest";
import { createEmptyDocument, docAddNode } from "@polterware/frontend-canvas";
import { prepareImport } from "./import-json";

describe("import json", () => {
  it("rejects malformed without providing replace doc", () => {
    const cur = createEmptyDocument({ systemId: "s" });
    const r = prepareImport(cur, "{nope");
    expect(r.ok).toBe(false);
  });

  it("rejects unknown version", () => {
    const cur = createEmptyDocument({ systemId: "s" });
    const r = prepareImport(cur, JSON.stringify({ formatVersion: 99, nodes: [], relationships: [] }));
    expect(r.ok).toBe(false);
  });

  it("accepts valid v1 and keeps backup", () => {
    let cur = createEmptyDocument({ systemId: "s" });
    cur = docAddNode(cur, { name: "Old", type: "api", x: 0, y: 0 });
    const incoming = createEmptyDocument({ systemId: "s" });
    const body = JSON.stringify({
      ...incoming,
      nodes: [
        {
          id: "n1",
          name: "New",
          type: "api",
          origin: "manual",
          state: "unknown",
          x: 0,
          y: 0,
          width: 100,
          height: 40,
          zIndex: 0,
        },
      ],
    });
    const r = prepareImport(cur, body);
    expect(r.ok).toBe(true);
    if (r.ok) {
      expect(r.doc.nodes[0]?.name).toBe("New");
      expect(r.backup).toContain("Old");
    }
  });
});
