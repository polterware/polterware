import { describe, expect, it } from "vitest";
import { decideDrop, type NestHitNode } from "./nest-layout";

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
