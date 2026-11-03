import { describe, expect, it } from "vitest";
import { createEmptyDocument, docAddNode } from "@polterware/frontend-canvas";
import { defaultPorts } from "./registry";

describe("create node", () => {
  it("creates with defaults and ports", () => {
    let doc = createEmptyDocument({ systemId: "s" });
    doc = docAddNode(doc, {
      name: "Orders API",
      type: "api",
      x: 10,
      y: 20,
      ports: defaultPorts("api"),
    });
    expect(doc.nodes).toHaveLength(1);
    expect(doc.nodes[0]!.type).toBe("api");
    expect(doc.nodes[0]!.ports).toHaveLength(2);
    expect(doc.nodes[0]!.origin).toBe("manual");
  });
});
