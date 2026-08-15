import { describe, expect, it } from "vitest";
import type { CanvasModel } from "@polterware/frontend-canvas";
import { assignLayer } from "./layers";
import { filterNodes, groupNodesByLayer } from "./map-view";

const model: CanvasModel = {
  revision: 0,
  selection: [],
  nodes: [
    { id: "1", x: 0, y: 0, label: "PostgreSQL", kind: "resource" },
    { id: "2", x: 0, y: 0, label: "Auth Service", kind: "resource" },
    { id: "3", x: 0, y: 0, label: "GitHub Actions", kind: "resource" },
  ],
  edges: [],
};

describe("architecture map view", () => {
  it("assigns layers", () => {
    expect(assignLayer(model.nodes[0]!)).toBe("data");
    expect(assignLayer(model.nodes[1]!)).toBe("applications");
    expect(assignLayer(model.nodes[2]!)).toBe("cloud");
  });

  it("groups nodes by layer", () => {
    const groups = groupNodesByLayer(model.nodes);
    expect(groups.data).toHaveLength(1);
    expect(groups.applications).toHaveLength(1);
    expect(groups.cloud).toHaveLength(1);
  });

  it("filters nodes by search", () => {
    const filtered = filterNodes(model, "auth");
    expect(filtered.nodes).toHaveLength(1);
    expect(filtered.nodes[0]!.id).toBe("2");
  });
});
