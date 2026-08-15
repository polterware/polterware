import type { Point, PortSide, Rect } from "../document/types";

export type ConnectionSide = Exclude<PortSide, "auto">;

export type ConnectionAnchor = {
  point: Point;
  side: ConnectionSide;
};

export type OrthogonalRoute = {
  source: ConnectionAnchor;
  target: ConnectionAnchor;
  points: Point[];
};

export function edgeHitPadding(): number {
  return 14;
}

export function pointNearSegment(
  p: Point,
  a: Point,
  b: Point,
  pad: number,
): boolean {
  return segmentDistanceSq(p, a, b) <= pad * pad;
}

/** Simple orthogonal-ish path: mid elbow. */
export function orthogonalPath(
  x1: number,
  y1: number,
  x2: number,
  y2: number,
): string {
  const mx = (x1 + x2) / 2;
  return `M ${x1} ${y1} L ${mx} ${y1} L ${mx} ${y2} L ${x2} ${y2}`;
}

function anchorPoint(rect: Rect, side: ConnectionSide): Point {
  switch (side) {
    case "left":
      return { x: rect.x, y: rect.y + rect.height / 2 };
    case "right":
      return { x: rect.x + rect.width, y: rect.y + rect.height / 2 };
    case "top":
      return { x: rect.x + rect.width / 2, y: rect.y };
    case "bottom":
      return { x: rect.x + rect.width / 2, y: rect.y + rect.height };
  }
}

export function resolveAutomaticConnection(a: Rect, b: Rect): OrthogonalRoute {
  const cx1 = a.x + a.width / 2;
  const cy1 = a.y + a.height / 2;
  const cx2 = b.x + b.width / 2;
  const cy2 = b.y + b.height / 2;
  const dx = cx2 - cx1;
  const dy = cy2 - cy1;

  let sourceSide: ConnectionSide;
  let targetSide: ConnectionSide;
  if (Math.abs(dx) >= Math.abs(dy)) {
    sourceSide = dx >= 0 ? "right" : "left";
    targetSide = dx >= 0 ? "left" : "right";
  } else {
    sourceSide = dy >= 0 ? "bottom" : "top";
    targetSide = dy >= 0 ? "top" : "bottom";
  }

  const source = { point: anchorPoint(a, sourceSide), side: sourceSide };
  const target = { point: anchorPoint(b, targetSide), side: targetSide };
  const points: Point[] = [source.point];

  if (sourceSide === "left" || sourceSide === "right") {
    const middleX = (source.point.x + target.point.x) / 2;
    points.push(
      { x: middleX, y: source.point.y },
      { x: middleX, y: target.point.y },
    );
  } else {
    const middleY = (source.point.y + target.point.y) / 2;
    points.push(
      { x: source.point.x, y: middleY },
      { x: target.point.x, y: middleY },
    );
  }

  points.push(target.point);
  return { source, target, points };
}

export function routeOrthogonalLine(a: Rect, b: Rect): Point[] {
  return resolveAutomaticConnection(a, b).points;
}

function sideToward(rect: Rect, p: Point): ConnectionSide {
  const cx = rect.x + rect.width / 2;
  const cy = rect.y + rect.height / 2;
  const dx = p.x - cx;
  const dy = p.y - cy;
  if (Math.abs(dx) >= Math.abs(dy)) return dx >= 0 ? "right" : "left";
  return dy >= 0 ? "bottom" : "top";
}

function isHorizontalSide(side: ConnectionSide): boolean {
  return side === "left" || side === "right";
}

function appendRoutePoint(pts: Point[], target: Point, first: "horizontal" | "vertical") {
  const last = pts[pts.length - 1]!;
  if (last.x === target.x || last.y === target.y) {
    pts.push({ ...target });
    return;
  }
  if (first === "horizontal") pts.push({ x: target.x, y: last.y });
  else pts.push({ x: last.x, y: target.y });
  pts.push({ ...target });
}

export function cleanupRoutePoints(points: readonly Point[]): Point[] {
  const pts: Point[] = [];
  for (const p of points) {
    const last = pts[pts.length - 1];
    if (!last || last.x !== p.x || last.y !== p.y) pts.push({ ...p });
  }
  const out: Point[] = [];
  for (let i = 0; i < pts.length; i++) {
    const prev = pts[i - 1];
    const cur = pts[i]!;
    const next = pts[i + 1];
    if (prev && next) {
      const cross =
        (cur.x - prev.x) * (next.y - cur.y) - (cur.y - prev.y) * (next.x - cur.x);
      if (cross === 0) continue;
    }
    out.push(cur);
  }
  return out;
}

/**
 * Resolve the route for a relationship. Without manual points it falls back to
 * the automatic orthogonal route; with manual points the interior chain is kept
 * and endpoints re-anchor to the closest side of each rect.
 */
export function resolveRelationshipRoute(
  a: Rect,
  b: Rect,
  manualPoints?: readonly Point[],
): Point[] {
  if (!manualPoints || manualPoints.length === 0) return routeOrthogonalLine(a, b);
  const first = manualPoints[0]!;
  const last = manualPoints[manualPoints.length - 1]!;
  const sSide = sideToward(a, first);
  const tSide = sideToward(b, last);
  const pts: Point[] = [anchorPoint(a, sSide)];
  appendRoutePoint(pts, first, isHorizontalSide(sSide) ? "horizontal" : "vertical");
  for (let i = 1; i < manualPoints.length; i++) pts.push({ ...manualPoints[i]! });
  appendRoutePoint(pts, anchorPoint(b, tSide), isHorizontalSide(tSide) ? "vertical" : "horizontal");
  return cleanupRoutePoints(pts);
}

function segmentDistanceSq(p: Point, a: Point, b: Point): number {
  const dx = b.x - a.x;
  const dy = b.y - a.y;
  const len2 = dx * dx + dy * dy;
  if (len2 === 0) {
    const ddx = p.x - a.x;
    const ddy = p.y - a.y;
    return ddx * ddx + ddy * ddy;
  }
  let t = ((p.x - a.x) * dx + (p.y - a.y) * dy) / len2;
  t = Math.max(0, Math.min(1, t));
  const cx = a.x + t * dx;
  const cy = a.y + t * dy;
  const ddx = p.x - cx;
  const ddy = p.y - cy;
  return ddx * ddx + ddy * ddy;
}

export function nearestSegmentIndex(p: Point, pts: readonly Point[]): number {
  let best = 0;
  let bestD = Infinity;
  for (let i = 0; i < pts.length - 1; i++) {
    const d = segmentDistanceSq(p, pts[i]!, pts[i + 1]!);
    if (d < bestD) {
      bestD = d;
      best = i;
    }
  }
  return best;
}

/**
 * Move one segment of an orthogonal route perpendicular to its orientation.
 * Endpoints (anchors) never move; dragging a segment that touches an anchor
 * inserts a new vertex next to that anchor. Returns the full route.
 */
export function moveRouteSegment(
  route: readonly Point[],
  segmentIndex: number,
  delta: Point,
): Point[] {
  const pts = route.map((p) => ({ ...p }));
  const a = pts[segmentIndex];
  const b = pts[segmentIndex + 1];
  if (!a || !b) return pts;
  const horizontal = a.y === b.y;
  if (!horizontal && a.x !== b.x) return pts;
  const mx = horizontal ? 0 : delta.x;
  const my = horizontal ? delta.y : 0;
  for (const idx of [segmentIndex + 1, segmentIndex]) {
    const p = pts[idx]!;
    const moved = { x: p.x + mx, y: p.y + my };
    if (idx === 0) pts.splice(1, 0, moved);
    else if (idx === pts.length - 1) pts.splice(pts.length - 1, 0, moved);
    else pts[idx] = moved;
  }
  return cleanupRoutePoints(pts);
}

/** Remove an interior vertex, reconnecting neighbors orthogonally. */
export function removeRouteVertex(route: readonly Point[], index: number): Point[] {
  if (index <= 0 || index >= route.length - 1) return route.map((p) => ({ ...p }));
  const prev = route[index - 1]!;
  const next = route[index + 1]!;
  const out = route.slice(0, index).concat(route.slice(index + 1));
  if (prev.x !== next.x && prev.y !== next.y) {
    const beforePrev = route[index - 2];
    const incomingHorizontal = beforePrev ? beforePrev.y === prev.y : true;
    const elbow = incomingHorizontal
      ? { x: prev.x, y: next.y }
      : { x: next.x, y: prev.y };
    out.splice(index, 0, elbow);
  }
  return cleanupRoutePoints(out);
}

export function pointsToSvgPath(points: readonly Point[]): string {
  const first = points[0];
  if (!first) return "";
  let d = `M ${first.x} ${first.y}`;
  for (let i = 1; i < points.length; i++) {
    const point = points[i];
    if (point) d += ` L ${point.x} ${point.y}`;
  }
  return d;
}

export function hitTestPolyline(p: Point, pts: readonly Point[], pad: number): boolean {
  for (let i = 0; i < pts.length - 1; i++) {
    if (pointNearSegment(p, pts[i]!, pts[i + 1]!, pad)) return true;
  }
  return false;
}

export function straightPath(x1: number, y1: number, x2: number, y2: number): string {
  return `M ${x1} ${y1} L ${x2} ${y2}`;
}

export function polylineMidpoint(pts: readonly Point[]): Point | null {
  if (pts.length === 0) return null;
  if (pts.length === 1) return { ...pts[0]! };

  let totalLen = 0;
  const segments: { p1: Point; p2: Point; len: number }[] = [];

  for (let i = 0; i < pts.length - 1; i++) {
    const p1 = pts[i]!;
    const p2 = pts[i + 1]!;
    const dx = p2.x - p1.x;
    const dy = p2.y - p1.y;
    const len = Math.sqrt(dx * dx + dy * dy);
    segments.push({ p1, p2, len });
    totalLen += len;
  }

  if (totalLen === 0) {
    return { ...pts[0]! };
  }

  const midLen = totalLen / 2;
  let currentLen = 0;

  for (const seg of segments) {
    if (currentLen + seg.len >= midLen) {
      const remaining = midLen - currentLen;
      const ratio = seg.len === 0 ? 0 : remaining / seg.len;
      return {
        x: seg.p1.x + (seg.p2.x - seg.p1.x) * ratio,
        y: seg.p1.y + (seg.p2.y - seg.p1.y) * ratio,
      };
    }
    currentLen += seg.len;
  }

  const last = pts[pts.length - 1]!;
  return { ...last };
}

export function bezierPath(x1: number, y1: number, x2: number, y2: number): string {
  const dx = Math.abs(x2 - x1) * 0.5;
  return `M ${x1} ${y1} C ${x1 + dx} ${y1}, ${x2 - dx} ${y2}, ${x2} ${y2}`;
}

export function pointsToRoundedSvgPath(
  points: readonly Point[],
  radius: number = 8,
): string {
  if (points.length === 0) return "";

  const safeRadius = Number.isFinite(radius) ? Math.max(0, radius) : 8;

  const pts: Point[] = [];
  for (const p of points) {
    if (
      pts.length === 0 ||
      pts[pts.length - 1]!.x !== p.x ||
      pts[pts.length - 1]!.y !== p.y
    ) {
      pts.push(p);
    }
  }

  if (pts.length === 0) return "";
  if (pts.length === 1) return `M ${pts[0]!.x} ${pts[0]!.y}`;
  if (pts.length === 2) return `M ${pts[0]!.x} ${pts[0]!.y} L ${pts[1]!.x} ${pts[1]!.y}`;

  let d = `M ${pts[0]!.x} ${pts[0]!.y}`;

  for (let i = 1; i < pts.length - 1; i++) {
    const p0 = pts[i - 1]!;
    const p1 = pts[i]!;
    const p2 = pts[i + 1]!;

    const d01 = Math.sqrt((p1.x - p0.x) ** 2 + (p1.y - p0.y) ** 2);
    const d12 = Math.sqrt((p2.x - p1.x) ** 2 + (p2.y - p1.y) ** 2);

    const r = Math.min(safeRadius, d01 / 2, d12 / 2);

    const v01x = d01 === 0 ? 0 : (p1.x - p0.x) / d01;
    const v01y = d01 === 0 ? 0 : (p1.y - p0.y) / d01;

    const v12x = d12 === 0 ? 0 : (p2.x - p1.x) / d12;
    const v12y = d12 === 0 ? 0 : (p2.y - p1.y) / d12;

    const cross = v01x * v12y - v01y * v12x;

    if (r === 0 || Math.abs(cross) < 1e-6) {
      d += ` L ${p1.x} ${p1.y}`;
    } else {
      const p1x_start = p1.x - v01x * r;
      const p1y_start = p1.y - v01y * r;

      const p1x_end = p1.x + v12x * r;
      const p1y_end = p1.y + v12y * r;

      d += ` L ${p1x_start} ${p1y_start} Q ${p1.x} ${p1.y} ${p1x_end} ${p1y_end}`;
    }
  }

  const last = pts[pts.length - 1]!;
  d += ` L ${last.x} ${last.y}`;

  return d;
}
