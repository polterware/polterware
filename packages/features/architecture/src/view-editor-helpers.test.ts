import { describe, expect, it } from "vitest";
import type { ArchNode } from "@polterware/frontend-canvas";
import {
  memberLayerForType,
  membersFromSelection,
  nodesFromArchitectureDocument,
} from "./view-editor-helpers";

function node(partial: Partial<ArchNode> & Pick<ArchNode, "id" | "name" | "type">): ArchNode {
  return {
    origin: "manual",
    state: "planned",
    x: 0,
    y: 0,
    width: 160,
    height: 88,
    zIndex: 1,
    ...partial,
  };
}

describe("view editor helpers", () => {
  it("maps canvas nodes to editor rows without fixture names", () => {
    const rows = nodesFromArchitectureDocument([
      node({ id: "n-pg", name: "PostgreSQL", type: "database" }),
      node({ id: "n-gw", name: "API Gateway", type: "api_gateway" }),
    ]);
    expect(rows).toEqual([
      { id: "n-pg", name: "PostgreSQL", type: "Database", layer: "Data" },
      { id: "n-gw", name: "API Gateway", type: "API Gateway", layer: "Edge" },
    ]);
  });

  it("keeps existing member labels when re-saving a view", () => {
    expect(memberLayerForType("ci_pipeline")).toBe("Delivery");
    expect(membersFromSelection(
      ["n-pay", "missing"],
      [{ id: "n-pay", name: "Payments", type: "Service", layer: "Application" }],
      [{ id: "n-pay", name: "Payment Service", type: "Service", layer: "Application" }],
    )).toEqual([
      { nodeId: "n-pay", name: "Payments", type: "Service", layer: "Application" },
    ]);
  });
});
