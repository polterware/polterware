import type { Point } from "../document/types";

export type GuideLine = {
  orientation: "h" | "v";
  position: number;
};

export function snapToGrid(p: Point, gridSize: number, enabled: boolean): Point {
  if (!enabled || gridSize <= 0) return p;
  return {
    x: Math.round(p.x / gridSize) * gridSize,
    y: Math.round(p.y / gridSize) * gridSize,
  };
}

export function snapDelta(
  origin: Point,
  current: Point,
  gridSize: number,
  enabled: boolean,
): Point {
  if (!enabled) {
    return { x: current.x - origin.x, y: current.y - origin.y };
  }
  const snapped = snapToGrid(current, gridSize, true);
  const originSnapped = snapToGrid(origin, gridSize, true);
  return { x: snapped.x - originSnapped.x, y: snapped.y - originSnapped.y };
}

/** Align edges of moving rects to others — returns guides + adjusted delta. */
export function objectSnapGuides(
  moving: { x: number; y: number; width: number; height: number },
  others: Array<{ x: number; y: number; width: number; height: number }>,
  threshold = 6,
) {
  let dx = 0;
  let dy = 0;
  const guides: GuideLine[] = [];
  const mx = [moving.x, moving.x + moving.width / 2, moving.x + moving.width];
  const my = [moving.y, moving.y + moving.height / 2, moving.y + moving.height];

  for (const o of others) {
    const ox = [o.x, o.x + o.width / 2, o.x + o.width];
    const oy = [o.y, o.y + o.height / 2, o.y + o.height];
    for (const a of mx) {
      for (const b of ox) {
        if (Math.abs(a + dx - b) < threshold) {
          dx = b - a;
          guides.push({ orientation: "v", position: b });
        }
      }
    }
    for (const a of my) {
      for (const b of oy) {
        if (Math.abs(a + dy - b) < threshold) {
          dy = b - a;
          guides.push({ orientation: "h", position: b });
        }
      }
    }
  }
  return { dx, dy, guides };
}
