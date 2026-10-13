import { describe, expect, it } from "vitest";
import type { CardId } from "./card-order";
import { dropSide, freeCells, freeSpot, normalizeAnchors, packOrder, resolveDrop } from "./card-layout";

describe("dropSide", () => {
  const box = { left: 0, top: 0, width: 200, height: 100 };

  it("picks the side the point is furthest toward", () => {
    expect(dropSide(box, 100, 10)).toBe("top");
    expect(dropSide(box, 100, 90)).toBe("bottom");
    expect(dropSide(box, 10, 50)).toBe("left");
    expect(dropSide(box, 190, 50)).toBe("right");
  });

  it("normalizes by the box size so wide cards still have top and bottom halves", () => {
    expect(dropSide(box, 40, 5)).toBe("top");
    expect(dropSide(box, 160, 95)).toBe("bottom");
  });
});

describe("resolveDrop", () => {
  it("inserts before the target on top and left, pushing it later", () => {
    const order: CardId[] = ["cpu", "memory", "network"];
    expect(resolveDrop(order, order, 1, "network", "cpu", "top")).toEqual(["network", "cpu", "memory"]);
    expect(resolveDrop(order, order, 3, "network", "memory", "left")).toEqual(["cpu", "network", "memory"]);
  });

  it("inserts after the target on bottom and right, letting it move earlier", () => {
    const order: CardId[] = ["cpu", "memory", "network"];
    expect(resolveDrop(order, order, 1, "cpu", "memory", "bottom")).toEqual(["memory", "cpu", "network"]);
    expect(resolveDrop(order, order, 3, "cpu", "network", "right")).toEqual(["memory", "network", "cpu"]);
  });

  it("looks further along until the card really lands below the target", () => {
    // cols 2: memory and thermal sit side by side, so pressure only lands below memory after thermal.
    const order: CardId[] = ["memory", "thermal", "pressure"];
    expect(resolveDrop(order, order, 2, "pressure", "memory", "bottom")).toEqual(["memory", "thermal", "pressure"]);
    const moved: CardId[] = ["pressure", "memory", "thermal"];
    expect(resolveDrop(moved, moved, 2, "pressure", "memory", "bottom")).toEqual(["memory", "thermal", "pressure"]);
  });

  it("falls back to inserting right after the target when the packing cannot put it below", () => {
    const order: CardId[] = ["cpu", "pressure", "memory"];
    expect(resolveDrop(order, order, 2, "memory", "cpu", "bottom")).toEqual(["cpu", "memory", "pressure"]);
  });

  it("keeps hidden cards in their slots", () => {
    const order: CardId[] = ["cpu", "network", "memory", "thermal"];
    const visible: CardId[] = ["cpu", "memory", "thermal"];
    expect(resolveDrop(order, visible, 3, "thermal", "memory", "left")).toEqual(["cpu", "network", "thermal", "memory"]);
    expect(resolveDrop(order, visible, 3, "cpu", "thermal", "right")).toEqual(["network", "memory", "thermal", "cpu"]);
  });
});

describe("packOrder with anchors", () => {
  it("keeps an anchored card on its cell and flows the rest around it", () => {
    const rects = packOrder(["memory", "pressure", "thermal"], 3, { pressure: { x: 2, y: 3 } });
    expect(rects.get("pressure")).toEqual({ x: 2, y: 3, w: 1, h: 1 });
    expect(rects.get("memory")).toEqual({ x: 0, y: 0, w: 1, h: 1 });
    expect(rects.get("thermal")).toEqual({ x: 1, y: 0, w: 1, h: 1 });
  });

  it("clamps an anchor to the column count", () => {
    const rects = packOrder(["disk"], 2, { disk: { x: 1, y: 0 } });
    expect(rects.get("disk")).toEqual({ x: 0, y: 0, w: 2, h: 1 });
  });

  it("flows an anchored card whose cell is already taken", () => {
    const rects = packOrder(["memory", "pressure"], 2, { memory: { x: 0, y: 0 }, pressure: { x: 0, y: 0 } });
    expect(rects.get("pressure")).toEqual({ x: 1, y: 0, w: 1, h: 1 });
  });
});

describe("freeSpot", () => {
  const rects = packOrder(["memory", "pressure"], 3);

  it("accepts a free cell and ignores the card being moved", () => {
    expect(freeSpot(rects, "pressure", 3, { x: 2, y: 0 })).toEqual({ x: 2, y: 0 });
    expect(freeSpot(rects, "pressure", 3, { x: 1, y: 1 })).toEqual({ x: 1, y: 1 });
  });

  it("rejects a cell another card covers", () => {
    expect(freeSpot(rects, "pressure", 3, { x: 0, y: 0 })).toBeNull();
  });
});

describe("freeCells", () => {
  it("lists empty cells down to the packed bottom, counting the held card as free", () => {
    const rects = new Map<CardId, { x: number; y: number; w: number; h: number }>([
      ["cpu", { x: 0, y: 0, w: 1, h: 2 }],
      ["memory", { x: 1, y: 0, w: 1, h: 1 }],
      ["pressure", { x: 2, y: 0, w: 1, h: 1 }],
    ]);
    expect(freeCells(rects, "pressure", 3)).toEqual([
      { x: 2, y: 0 },
      { x: 1, y: 1 },
      { x: 2, y: 1 },
    ]);
  });
});

describe("normalizeAnchors", () => {
  it("drops unknown cards and malformed cells", () => {
    expect(normalizeAnchors({ pressure: { x: 2, y: 3 }, nope: { x: 0, y: 0 }, cpu: { x: -1, y: 0 }, disk: "x" })).toEqual({
      pressure: { x: 2, y: 3 },
    });
  });
});
