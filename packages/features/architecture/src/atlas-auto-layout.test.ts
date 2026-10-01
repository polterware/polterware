import { describe, expect, it } from "vitest";
import type { ArchitectureDocument, ArchNode } from "@polterware/frontend-canvas";
import { arrangeAtlas } from "./atlas-auto-layout";
import { ATLAS_LEVEL_KEY, ATLAS_PARENT_KEY, nestRowFrameSize } from "./nest-layout";

function node(id: string, level: string, parentId: string, x = 0, y = 0): ArchNode {
  return {
    id,
    name: id,
    type: "service",
    origin: "manual",
    state: "observed",
    x,
    y,
    width: 188,
    height: 56,
    zIndex: 1,
    boundaryId: null,
    metadata: { [ATLAS_LEVEL_KEY]: level, [ATLAS_PARENT_KEY]: parentId },
  };
}

function doc(nodes: ArchNode[]): ArchitectureDocument {
  return { nodes, relationships: [], boundaries: [] } as unknown as ArchitectureDocument;
}

const edge = (sourceNodeId: string, targetNodeId: string) => ({ sourceNodeId, targetNodeId });
const at = (result: ArchitectureDocument, id: string) => result.nodes.find((item) => item.id === id)!;

describe("arrangeAtlas", () => {
  const map = doc([
    node("app", "system", ""),
    node("data", "domain", "app", 0, 0),
    node("web", "domain", "app", 900, 0),
    node("store", "component", "data", 0, 0),
    node("cache", "component", "data", 244, 0),
    node("ui", "component", "web", 0, 0),
    node("loose", "component", "app", 400, 400),
  ]);

  it("puts what a card uses in a lower row", () => {
    const result = arrangeAtlas(map, [edge("ui", "store")]);
    expect(at(result, "web").y).toBeLessThan(at(result, "data").y);
  });

  it("arranges only the visible nodes and leaves the rest in place", () => {
    const visible = new Set(["web", "ui", "loose"]);
    const result = arrangeAtlas(map, [edge("ui", "store")], visible);
    expect(at(result, "data").x).toBe(0);
    expect(at(result, "data").y).toBe(0);
    expect(at(result, "web").x).toBe(48);
    expect(at(result, "web").y).toBe(48);
  });

  it("sends units with no edges to the last row", () => {
    const result = arrangeAtlas(map, [edge("ui", "store")]);
    expect(at(result, "loose").y).toBeGreaterThan(at(result, "data").y);
  });

  it("never overlaps two units", () => {
    const result = arrangeAtlas(map, [edge("ui", "store"), edge("loose", "cache")]);
    const size = (id: string) => (id === "loose" ? { width: 188, height: 56 } : nestRowFrameSize(id === "data" ? 2 : 1));
    const rects = ["data", "web", "loose"].map((id) => ({ ...at(result, id), ...size(id) }));
    for (const a of rects) {
      for (const b of rects) {
        if (a === b) continue;
        const apart = a.x + a.width <= b.x || b.x + b.width <= a.x || a.y + a.height <= b.y || b.y + b.height <= a.y;
        expect(apart).toBe(true);
      }
    }
  });

  it("orders a domain's cards toward their outside neighbours", () => {
    const wide = doc([
      node("app", "system", ""),
      node("left", "domain", "app"),
      node("right", "domain", "app"),
      node("mid", "domain", "app"),
      node("l", "component", "left"),
      node("r", "component", "right"),
      node("a", "component", "mid", 0),
      node("b", "component", "mid", 244),
      node("z", "component", "app"),
    ]);
    // z sits above left and right, pulling them apart; mid's cards face them.
    const result = arrangeAtlas(wide, [
      edge("z", "l"),
      edge("z", "r"),
      edge("l", "b"),
      edge("r", "a"),
    ]);
    const leftFirst = at(result, "left").x < at(result, "right").x;
    expect(at(result, "b").x < at(result, "a").x).toBe(leftFirst);
  });

  it("leaves system nodes alone", () => {
    const result = arrangeAtlas(map, [edge("ui", "store")]);
    expect(at(result, "app")).toBe(map.nodes[0]);
  });
});
