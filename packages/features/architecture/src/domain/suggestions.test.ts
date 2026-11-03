import { describe, expect, it } from "vitest";
import type { ArchNode } from "@polterware/frontend-canvas";
import { suggestRelationshipTypes, inferRelationshipType } from "./suggestions";

function node(partial: Partial<ArchNode> & { id: string; type: ArchNode["type"] }): ArchNode {
  return {
    name: partial.id,
    origin: "manual",
    state: "planned",
    x: 0,
    y: 0,
    width: 100,
    height: 40,
    zIndex: 0,
    ...partial,
  };
}

describe("suggestRelationshipTypes", () => {
  it("suggests calls for web → api", () => {
    const s = suggestRelationshipTypes(
      node({ id: "a", type: "web_application" }),
      node({ id: "b", type: "api" }),
    );
    expect(s[0]).toBe("calls");
  });

  it("suggests reads_from for service → database", () => {
    const s = suggestRelationshipTypes(
      node({ id: "a", type: "service" }),
      node({ id: "b", type: "database" }),
    );
    expect(s[0]).toBe("reads_from");
  });

  it("suggests consumes_from for worker → queue", () => {
    const s = suggestRelationshipTypes(
      node({ id: "a", type: "worker" }),
      node({ id: "b", type: "queue" }),
    );
    expect(s[0]).toBe("consumes_from");
  });

  it("inferRelationshipType aliases first suggestion", () => {
    const a = node({ id: "a", type: "api" });
    const b = node({ id: "b", type: "database" });
    expect(inferRelationshipType(a, b)).toBe(suggestRelationshipTypes(a, b)[0]);
  });
});
