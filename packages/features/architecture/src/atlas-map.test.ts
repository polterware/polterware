import { describe, expect, it } from "vitest";
import type { ArchitectureDocument, ArchNode } from "@polterware/frontend-canvas";
import { absorbNestEdit, nestChildren, placeNestRow, projectAtlasWorld } from "./atlas-map";
import { ATLAS_LEVEL_KEY, ATLAS_PARENT_KEY } from "./nest-layout";

function card(id: string, parentId: string, x: number, y = 0): ArchNode {
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
    metadata: { [ATLAS_LEVEL_KEY]: "component", [ATLAS_PARENT_KEY]: parentId },
  };
}

function doc(nodes: ArchNode[]): ArchitectureDocument {
  return { nodes, relationships: [], boundaries: [] } as unknown as ArchitectureDocument;
}

describe("domain rows", () => {
  it("puts cards on fixed slots in the given order", () => {
    const local = doc([card("core", "platform", 0), card("cli", "platform", 300, 40), card("api", "platform", 700)]);
    const next = placeNestRow(local, ["api", "core", "cli"]);
    expect(next.nodes.map((node) => [node.id, node.x, node.y])).toEqual([
      ["core", 244, 0],
      ["cli", 488, 0],
      ["api", 0, 0],
    ]);
  });

  it("lists a domain's cards in row order without the excluded one", () => {
    const local = doc([card("api", "platform", 488), card("core", "platform", 0), card("other", "product", 0)]);
    expect(nestChildren(local, "platform", "cli").map((node) => node.id)).toEqual(["core", "api"]);
    expect(nestChildren(local, "platform", "core").map((node) => node.id)).toEqual(["api"]);
  });
});

describe("absorbNestEdit", () => {
  it("keeps a card's offset when its domain moves with it", () => {
    const domain = { ...card("platform", "", 0), metadata: { [ATLAS_LEVEL_KEY]: "domain", [ATLAS_PARENT_KEY]: "" } };
    const local = { ...doc([domain, card("core", "platform", 0), card("api", "platform", 244)]), viewport: { x: 0, y: 0, zoom: 1 } };
    const moved = projectAtlasWorld({ ...local, nodes: local.nodes.map((node) => (node.id === "platform" ? { ...node, x: 500, y: 300 } : node)) });
    const next = absorbNestEdit(local, moved);
    expect(next.nodes.map((node) => [node.id, node.x, node.y])).toEqual([
      ["platform", 500, 300],
      ["core", 0, 0],
      ["api", 244, 0],
    ]);
  });
});
