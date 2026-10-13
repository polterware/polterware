import { describe, expect, it } from "vitest";
import type { CardId } from "./card-order";
import { dropSide, resolveDrop } from "./card-layout";

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
    // cols 2: cpu fills the left column (h6), memory and thermal stack on the right (h3 each).
    const order: CardId[] = ["cpu", "memory", "thermal", "pressure"];
    expect(resolveDrop(order, order, 2, "pressure", "cpu", "bottom")).toEqual(["cpu", "memory", "thermal", "pressure"]);
    const moved: CardId[] = ["pressure", "cpu", "memory", "thermal"];
    expect(resolveDrop(moved, moved, 2, "pressure", "cpu", "bottom")).toEqual(["cpu", "memory", "thermal", "pressure"]);
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
