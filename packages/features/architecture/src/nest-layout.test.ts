import { describe, expect, it } from "vitest";
import {
  NEST_LEAF_W,
  NEST_PAD,
  decideDrop,
  nestDropIndex,
  nestRowFrameSize,
  nestRowOrder,
  nestSlotX,
  routeAroundCards,
  type NestHitNode,
} from "./nest-layout";

function hit(id: string, parentId: string, level: NestHitNode["level"], x: number, y: number, width: number, height: number): NestHitNode {
  return {
    id,
    parentId,
    level,
    depth: level === "component" ? 2 : 1,
    x,
    y,
    width,
    height,
    content: { x: x + 16, y: y + 44, width: width - 32, height: height - 60 },
  };
}

const platform = hit("platform", "system", "domain", 0, 0, 600, 120);
const data = hit("data", "system", "domain", 800, 0, 400, 120);
const core = hit("core", "platform", "component", 16, 44, 188, 56);
const nodes = [platform, data, core];

describe("decideDrop", () => {
  it("moves a domain freely on empty canvas", () => {
    const decision = decideDrop({
      dragged: platform,
      cursor: { x: 300, y: 600 },
      nextTopLeft: { x: 0, y: 500 },
      nodes,
      skip: new Set(["platform", "core"]),
      rootParentId: "system",
    });
    expect(decision).toEqual({ kind: "move" });
  });

  it("takes a component out of its domain onto the system", () => {
    const decision = decideDrop({
      dragged: core,
      cursor: { x: 100, y: 400 },
      nextTopLeft: { x: 20, y: 380 },
      nodes,
      skip: new Set(["core"]),
      rootParentId: "system",
    });
    expect(decision).toEqual({ kind: "reparent", parentId: "system", x: 20, y: 380 });
  });

  it("moves a component into another domain", () => {
    const decision = decideDrop({
      dragged: core,
      cursor: { x: 900, y: 80 },
      nextTopLeft: { x: 840, y: 60 },
      nodes,
      skip: new Set(["core"]),
      rootParentId: "system",
    });
    expect(decision).toEqual({ kind: "reparent", parentId: "data", x: 24, y: 16 });
  });

  it("keeps a component home without a system to leave for", () => {
    const decision = decideDrop({
      dragged: core,
      cursor: { x: 100, y: 400 },
      nextTopLeft: { x: 20, y: 380 },
      nodes,
      skip: new Set(["core"]),
    });
    expect(decision).toEqual({ kind: "reject" });
  });

  it("moves a loose component freely", () => {
    const loose = hit("cli", "system", "component", 100, 400, 188, 56);
    const decision = decideDrop({
      dragged: loose,
      cursor: { x: 150, y: 700 },
      nextTopLeft: { x: 100, y: 680 },
      nodes: [...nodes, loose],
      skip: new Set(["cli"]),
      rootParentId: "system",
    });
    expect(decision).toEqual({ kind: "move" });
  });
});

describe("routeAroundCards", () => {
  const card = (x: number, y: number) => ({ x, y, width: 188, height: 56 });
  const crosses = (points: { x: number; y: number }[], rect: { x: number; y: number; width: number; height: number }) =>
    points.slice(1).some((q, index) => {
      const p = points[index]!;
      return Math.max(p.x, q.x) > rect.x && Math.min(p.x, q.x) < rect.x + rect.width
        && Math.max(p.y, q.y) > rect.y && Math.min(p.y, q.y) < rect.y + rect.height;
    });

  it("keeps a direct route when nothing is in the way", () => {
    const route = routeAroundCards(card(0, 0), card(400, 0), []);
    expect(route[0]).toEqual({ x: 188, y: 28 });
    expect(route[route.length - 1]).toEqual({ x: 400, y: 28 });
    expect(route).toHaveLength(2);
  });

  it("goes around a card between the two ends", () => {
    const middle = card(250, 0);
    const route = routeAroundCards(card(0, 0), card(500, 0), [middle]);
    expect(crosses(route, middle)).toBe(false);
  });

  it("never runs through its own cards", () => {
    const a = card(0, 0);
    const b = card(0, 200);
    const route = routeAroundCards(a, b, [card(0, 100)]);
    const inner = (rect: typeof a) => ({ x: rect.x + 1, y: rect.y + 1, width: rect.width - 2, height: rect.height - 2 });
    expect(crosses(route, inner(a))).toBe(false);
    expect(crosses(route, inner(b))).toBe(false);
  });
});

describe("fixed row slots", () => {
  it("orders older free positions by row, then left to right", () => {
    const cards = [{ id: "b", x: 300, y: 0 }, { id: "c", x: 0, y: 120 }, { id: "a", x: 10, y: 4 }];
    expect(nestRowOrder(cards).map((card) => card.id)).toEqual(["a", "b", "c"]);
  });

  it("drops into the slot past the cards left of the drop", () => {
    const row = [{ x: 0 }, { x: 488 }];
    expect(nestDropIndex(row, 250)).toBe(1);
    expect(nestDropIndex(row, -40)).toBe(0);
    expect(nestDropIndex(row, 600)).toBe(2);
  });

  it("sizes a domain from its card count", () => {
    expect(nestRowFrameSize(2).width).toBe(nestSlotX(1) + NEST_LEAF_W + NEST_PAD * 2);
    expect(nestRowFrameSize(0).width).toBe(NEST_LEAF_W);
  });
});

describe("decideDrop over a sibling card", () => {
  it("reorders within the domain instead of rejecting", () => {
    const api = hit("api", "platform", "component", 260, 44, 188, 56);
    const decision = decideDrop({
      dragged: core,
      cursor: { x: 300, y: 70 },
      nextTopLeft: { x: 250, y: 44 },
      nodes: [...nodes, api],
      skip: new Set(["core"]),
      rootParentId: "system",
    });
    expect(decision).toEqual({ kind: "move" });
  });
});
