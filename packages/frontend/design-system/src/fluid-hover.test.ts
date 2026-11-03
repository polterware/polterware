import { describe, expect, it } from "vitest";
import { pickNearest, type ItemRect, type PickNearestInput } from "./fluid-hover";

const rows: ItemRect[] = [
  { top: 0, left: 0, width: 100, height: 20 },
  { top: 24, left: 0, width: 100, height: 20 },
  { top: 48, left: 0, width: 100, height: 20 },
];

const base = (overrides: Partial<PickNearestInput>): PickNearestInput => ({
  axis: "y",
  point: { x: 0, y: 0 },
  rects: rows,
  containerRect: { left: 10, top: 100, width: 100, height: 68 },
  scroll: { x: 0, y: 0 },
  border: { x: 0, y: 0 },
  layoutSize: { width: 100, height: 68 },
  ...overrides,
});

describe("pickNearest", () => {
  it("picks the item the pointer is inside", () => {
    expect(pickNearest(base({ point: { x: 50, y: 130 } }))).toBe(1);
  });

  it("picks the nearest item from a gap or past the last row", () => {
    expect(pickNearest(base({ point: { x: 50, y: 121.5 } }))).toBe(0);
    expect(pickNearest(base({ point: { x: 50, y: 400 } }))).toBe(2);
  });

  it("keeps the first item on a tie", () => {
    expect(pickNearest(base({ point: { x: 50, y: 122 } }))).toBe(0);
  });

  it("maps scroll offsets into viewport space", () => {
    expect(pickNearest(base({ point: { x: 50, y: 106 }, scroll: { x: 0, y: 24 } }))).toBe(1);
  });

  it("skips disabled and unregistered slots", () => {
    const sparse = [rows[0], undefined, rows[2]];
    expect(pickNearest(base({ point: { x: 50, y: 134 }, rects: sparse }))).toBe(0);
    expect(pickNearest(base({ point: { x: 50, y: 110 }, isDisabled: (index) => index === 0 }))).toBe(1);
  });

  it("uses the distance to centres on the xy axis", () => {
    const grid: ItemRect[] = [
      { top: 0, left: 0, width: 40, height: 40 },
      { top: 0, left: 60, width: 40, height: 40 },
    ];
    expect(pickNearest(base({ axis: "xy", rects: grid, point: { x: 70, y: 150 } }))).toBe(1);
  });

  it("returns null without rects", () => {
    expect(pickNearest(base({ rects: [] }))).toBeNull();
  });
});
