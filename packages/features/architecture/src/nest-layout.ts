import type { Point, Rect } from "@polterware/frontend-canvas";

/** Header row of a map card. Children sit in the well below it. */
export const NEST_HEADER = 56;
/** Label band of a domain frame. The frame is a border, not a card. */
export const NEST_FRAME_HEADER = 44;
export const NEST_PAD = 16;
/**
 * Title pill of a domain frame: inset from the frame, fixed height, width by name.
 * Every gap around it is NEST_PAD: PILL_TOP + PILL_H = FRAME_HEADER, cards start PAD below.
 */
export const NEST_PILL_TOP = NEST_PAD;
export const NEST_PILL_H = 28;
const NEST_PILL_CHROME = 1 + 5 + 16 + 8 + 12 + 1;
const NEST_PILL_CHAR_W = 7.5;
export const NEST_LEAF_W = 188;
export const NEST_LEAF_H = 56;
/** A domain frame is never shorter than one row of cards, empty or not. */
export const NEST_FRAME_MIN_H = NEST_FRAME_HEADER + NEST_PAD * 2 + NEST_LEAF_H;
/** Distance between sibling cards on a row, so the edges between them read. */
export const NEST_CHILD_GAP = NEST_LEAF_W + 56;

export const ATLAS_LEVEL_KEY = "atlasLevel";
export const ATLAS_PARENT_KEY = "atlasParent";
export const NEST_CONTENT_X = "nestContentX";
export const NEST_CONTENT_Y = "nestContentY";
export const NEST_CONTENT_W = "nestContentW";
export const NEST_CONTENT_H = "nestContentH";

export type NestLevel = "system" | "domain" | "component";

export function isNestLevel(value: string): value is NestLevel {
  return value === "system" || value === "domain" || value === "component";
}

/** The level that may contain `child`. A system has no parent. */
export function requiredParentLevel(child: NestLevel): NestLevel | null {
  if (child === "domain") return "system";
  if (child === "component") return "domain";
  return null;
}

/** A component also sits loose under its system, outside any domain. */
export function canNest(child: NestLevel, parent: NestLevel): boolean {
  return requiredParentLevel(child) === parent || (child === "component" && parent === "system");
}

/** The level created inside a selected card. A component has no child card. */
export function childLevel(parent: NestLevel | null): NestLevel | null {
  if (parent === null) return "system";
  if (parent === "system") return "domain";
  if (parent === "domain") return "component";
  return null;
}

export type NestSlot = {
  id: string;
  parentId: string;
  level: NestLevel;
  /** Roots: world origin. Children: offset inside the parent well. */
  x: number;
  y: number;
};

export type NestRect = Rect & {
  depth: number;
  content: Rect;
};

type Size = { width: number; height: number };

function frameHeader(level: NestLevel): number {
  return level === "domain" ? NEST_FRAME_HEADER : NEST_HEADER;
}

function contentOf(origin: Point, size: Size, header: number): Rect {
  return {
    x: origin.x + NEST_PAD,
    y: origin.y + header + NEST_PAD,
    width: Math.max(0, size.width - NEST_PAD * 2),
    height: Math.max(0, size.height - header - NEST_PAD * 2),
  };
}

/**
 * World rects for the map. A system is not drawn. Each domain is a root frame
 * and its components sit in the well. Stored slots stay local to that frame.
 */
export function layoutNest(nodes: readonly NestSlot[]): Map<string, NestRect> {
  const byId = new Map(nodes.map((node) => [node.id, node]));
  const children = new Map<string, NestSlot[]>();
  const roots: NestSlot[] = [];
  for (const node of nodes) {
    if (node.level === "system") continue;
    const parentNode = node.parentId ? byId.get(node.parentId) : undefined;
    const parent = parentNode && parentNode.level !== "system" ? parentNode.id : "";
    if (!parent) {
      roots.push(node);
      continue;
    }
    children.set(parent, [...(children.get(parent) ?? []), node]);
  }

  const sizes = new Map<string, Size>();
  const sizeOf = (id: string, stack: Set<string>): Size => {
    const cached = sizes.get(id);
    if (cached) return cached;
    if (stack.has(id)) return { width: NEST_LEAF_W, height: NEST_LEAF_H };
    stack.add(id);
    const node = byId.get(id);
    const header = frameHeader(node?.level ?? "component");
    const kids = children.get(id) ?? [];
    if (kids.length === 0) {
      const leaf = node?.level === "domain"
        ? { width: NEST_LEAF_W, height: NEST_FRAME_MIN_H }
        : { width: NEST_LEAF_W, height: NEST_LEAF_H };
      sizes.set(id, leaf);
      return leaf;
    }
    let maxX = 0;
    let maxY = 0;
    for (const kid of kids) {
      const kidSize = sizeOf(kid.id, stack);
      maxX = Math.max(maxX, Math.max(0, kid.x) + kidSize.width);
      maxY = Math.max(maxY, Math.max(0, kid.y) + kidSize.height);
    }
    const size = {
      width: Math.max(NEST_LEAF_W, maxX + NEST_PAD * 2),
      height: Math.max(node?.level === "domain" ? NEST_FRAME_MIN_H : 0, header + NEST_PAD + maxY + NEST_PAD),
    };
    sizes.set(id, size);
    return size;
  };

  const rects = new Map<string, NestRect>();
  const place = (node: NestSlot, origin: Point, depth: number, stack: Set<string>) => {
    if (stack.has(node.id)) return;
    stack.add(node.id);
    const size = sizeOf(node.id, new Set());
    const rect: NestRect = {
      ...origin,
      ...size,
      depth,
      content: contentOf(origin, size, frameHeader(node.level)),
    };
    rects.set(node.id, rect);
    for (const kid of children.get(node.id) ?? []) {
      place(
        kid,
        {
          x: rect.content.x + Math.max(0, kid.x),
          y: rect.content.y + Math.max(0, kid.y),
        },
        depth + 1,
        stack,
      );
    }
  };

  for (const root of roots) place(root, { x: root.x, y: root.y }, 0, new Set());
  return rects;
}

export function descendantIds(id: string, parentOf: (id: string) => string): (candidate: string) => boolean {
  return (candidate: string) => {
    let current = parentOf(candidate);
    const seen = new Set<string>();
    while (current && !seen.has(current)) {
      if (current === id) return true;
      seen.add(current);
      current = parentOf(current);
    }
    return false;
  };
}

export type NestHitNode = {
  id: string;
  parentId: string;
  level: NestLevel;
  depth: number;
  x: number;
  y: number;
  width: number;
  height: number;
  content: Rect;
};

export function hitNestNode(
  nodes: readonly NestHitNode[],
  point: Point,
  skip: ReadonlySet<string>,
): NestHitNode | null {
  let best: NestHitNode | null = null;
  for (const node of nodes) {
    if (skip.has(node.id)) continue;
    if (point.x < node.x || point.y < node.y || point.x > node.x + node.width || point.y > node.y + node.height) {
      continue;
    }
    if (!best || node.depth >= best.depth) best = node;
  }
  return best;
}

export type DropDecision =
  | { kind: "move" }
  | { kind: "reject" }
  | { kind: "reparent"; parentId: string; x: number; y: number };

/**
 * Where a released card lands. A root card (domain, or a component loose under
 * the system) moves freely. A component dropped outside its domain leaves it for
 * `rootParentId` (the system); dropped on another domain, it moves into that one.
 */
export function decideDrop(input: {
  dragged: NestHitNode;
  cursor: Point;
  nextTopLeft: Point;
  nodes: readonly NestHitNode[];
  skip: ReadonlySet<string>;
  /** The system id. Without it a card cannot leave its parent. */
  rootParentId?: string;
}): DropDecision {
  let hit = hitNestNode(input.nodes, input.cursor, input.skip);
  // Cards in a domain hold fixed slots: over a card means over its domain's row.
  if (hit && hit.level === "component" && input.dragged.level === "component") {
    const holder = input.nodes.find((node) => node.id === hit?.parentId);
    hit = holder ?? null;
  }
  const isDescendant = descendantIds(input.dragged.id, (id) => input.nodes.find((node) => node.id === id)?.parentId ?? "");
  if (
    hit
    && hit.id !== input.dragged.parentId
    && canNest(input.dragged.level, hit.level)
    && !isDescendant(hit.id)
  ) {
    return {
      kind: "reparent",
      parentId: hit.id,
      x: Math.max(0, input.nextTopLeft.x - hit.content.x),
      y: Math.max(0, input.nextTopLeft.y - hit.content.y),
    };
  }
  if (hit && hit.id !== input.dragged.id && hit.id !== input.dragged.parentId) return { kind: "reject" };
  const parent = input.nodes.find((node) => node.id === input.dragged.parentId);
  // The parent is not drawn (a system, or none): the card is a root and moves freely.
  if (!parent) return { kind: "move" };
  if (!hit) {
    const inside = input.cursor.x >= parent.x
      && input.cursor.y >= parent.y
      && input.cursor.x <= parent.x + parent.width
      && input.cursor.y <= parent.y + parent.height;
    if (inside) return { kind: "move" };
    if (input.rootParentId && canNest(input.dragged.level, "system")) {
      return { kind: "reparent", parentId: input.rootParentId, x: input.nextTopLeft.x, y: input.nextTopLeft.y };
    }
    return { kind: "reject" };
  }
  return { kind: "move" };
}

/** Keep elbows inside the parent well. Anchors stay on the cards. */
export function clampRouteToContent(points: readonly Point[], content: Rect, inset = 8): Point[] {
  if (content.width <= inset * 2 || content.height <= inset * 2) return points.map((point) => ({ ...point }));
  const minX = content.x + inset;
  const maxX = content.x + content.width - inset;
  const minY = content.y + inset;
  const maxY = content.y + content.height - inset;
  return points.map((point, index) => {
    if (index === 0 || index === points.length - 1) return { ...point };
    return {
      x: Math.min(maxX, Math.max(minX, point.x)),
      y: Math.min(maxY, Math.max(minY, point.y)),
    };
  });
}

/** Domains the map filter can focus. A system is the whole canvas. */
export function hierarchyGroups<T extends { id: string; parentId: string; level: NestLevel; title: string }>(
  nodes: readonly T[],
): T[] {
  return nodes.filter((node) => node.level === "domain");
}

export function subtreeIds(rootId: string, parentOf: ReadonlyMap<string, string>): Set<string> {
  const children = new Map<string, string[]>();
  for (const [id, parentId] of parentOf) {
    if (!parentId) continue;
    children.set(parentId, [...(children.get(parentId) ?? []), id]);
  }
  const ids = new Set<string>();
  const walk = (id: string) => {
    if (ids.has(id)) return;
    ids.add(id);
    for (const child of children.get(id) ?? []) walk(child);
  };
  walk(rootId);
  return ids;
}

/** Members plus the ancestors that still have to draw around them. */
export function viewMemberIds(memberIds: readonly string[], parentOf: ReadonlyMap<string, string>): Set<string> {
  const ids = new Set<string>();
  for (const id of memberIds) {
    let current = id;
    const seen = new Set<string>();
    while (current && !seen.has(current)) {
      seen.add(current);
      ids.add(current);
      current = parentOf.get(current) ?? "";
    }
  }
  return ids;
}

type RouteSide = "left" | "right" | "top" | "bottom";

const ROUTE_SIDES: readonly RouteSide[] = ["left", "right", "top", "bottom"];
/** Straight run out of a card before the first bend. */
const ROUTE_STUB = 16;
/** Gap between a detour lane and the cards it clears. */
const ROUTE_LANE_GAP = 20;
/** Gap between a detour lane and the domain frames it clears. */
const ROUTE_FRAME_GAP = 36;
/** Run an edge may make inside an end's own frame before it costs extra. */
const ROUTE_OWN_FRAME_RUN = 96;
/** Clearance kept around cards an edge does not belong to. */
const ROUTE_CLEARANCE = 8;

function routeAnchor(rect: Rect, side: RouteSide, stub = 0): Point {
  if (side === "left") return { x: rect.x - stub, y: rect.y + rect.height / 2 };
  if (side === "right") return { x: rect.x + rect.width + stub, y: rect.y + rect.height / 2 };
  if (side === "top") return { x: rect.x + rect.width / 2, y: rect.y - stub };
  return { x: rect.x + rect.width / 2, y: rect.y + rect.height + stub };
}

function segmentHitsRect(p: Point, q: Point, rect: Rect, inflate: number): boolean {
  return Math.max(p.x, q.x) > rect.x - inflate
    && Math.min(p.x, q.x) < rect.x + rect.width + inflate
    && Math.max(p.y, q.y) > rect.y - inflate
    && Math.min(p.y, q.y) < rect.y + rect.height + inflate;
}

/** Length of an axis-aligned segment inside a rect. */
function lengthInside(p: Point, q: Point, rect: Rect): number {
  const x0 = Math.max(Math.min(p.x, q.x), rect.x);
  const x1 = Math.min(Math.max(p.x, q.x), rect.x + rect.width);
  const y0 = Math.max(Math.min(p.y, q.y), rect.y);
  const y1 = Math.min(Math.max(p.y, q.y), rect.y + rect.height);
  if (x1 < x0 || y1 < y0) return 0;
  return (x1 - x0) + (y1 - y0);
}

function tidyRoute(points: readonly Point[]): Point[] {
  const out: Point[] = [];
  for (const point of points) {
    const last = out[out.length - 1];
    if (last && last.x === point.x && last.y === point.y) continue;
    const prev = out[out.length - 2];
    if (prev && last && ((prev.x === last.x && last.x === point.x) || (prev.y === last.y && last.y === point.y))) {
      out[out.length - 1] = { ...point };
      continue;
    }
    out.push({ ...point });
  }
  return out;
}

/**
 * Orthogonal route from `a` to `b` that goes around the other cards instead of
 * under them. Tries every side pair with direct elbows and detour lanes, then
 * keeps the cheapest: crossings cost most, then length, then bends. `bounds`
 * keeps an edge between siblings inside their frame.
 */
export function routeAroundCards(
  a: Rect,
  b: Rect,
  obstacles: readonly Rect[],
  bounds?: Rect,
  /** Frames a detour lane should clear too (domains, for edges between them). */
  frames: readonly Rect[] = [],
  /** Extra distance for this edge's lanes, so parallel detours do not merge into one line. */
  laneOffset = 0,
): Point[] {
  const minX = Math.min(a.x, b.x);
  const maxX = Math.max(a.x + a.width, b.x + b.width);
  const minY = Math.min(a.y, b.y);
  const maxY = Math.max(a.y + a.height, b.y + b.height);
  const lanes = [...obstacles, ...frames];
  const acrossX = lanes.filter((rect) => rect.x < maxX && rect.x + rect.width > minX);
  const acrossY = lanes.filter((rect) => rect.y < maxY && rect.y + rect.height > minY);
  const gap = (frames.length ? ROUTE_FRAME_GAP : ROUTE_LANE_GAP) + laneOffset;
  const laneTop = Math.min(minY, ...acrossX.map((rect) => rect.y)) - gap;
  const laneBottom = Math.max(maxY, ...acrossX.map((rect) => rect.y + rect.height)) + gap;
  const laneLeft = Math.min(minX, ...acrossY.map((rect) => rect.x)) - gap;
  const laneRight = Math.max(maxX, ...acrossY.map((rect) => rect.x + rect.width)) + gap;

  const holds = (frame: Rect, card: Rect) =>
    card.x >= frame.x && card.y >= frame.y
    && card.x + card.width <= frame.x + frame.width && card.y + card.height <= frame.y + frame.height;
  // Frames that hold neither end: running through them reads as belonging there.
  const foreign = frames.filter((frame) => !holds(frame, a) && !holds(frame, b));
  const own = frames.filter((frame) => holds(frame, a) || holds(frame, b));

  let best: Point[] | null = null;
  let bestCost = Number.POSITIVE_INFINITY;
  for (const sourceSide of ROUTE_SIDES) {
    for (const targetSide of ROUTE_SIDES) {
      const start = routeAnchor(a, sourceSide);
      const end = routeAnchor(b, targetSide);
      const s = routeAnchor(a, sourceSide, ROUTE_STUB);
      const t = routeAnchor(b, targetSide, ROUTE_STUB);
      const midX = (s.x + t.x) / 2;
      const midY = (s.y + t.y) / 2;
      const middles: Point[][] = [
        [{ x: t.x, y: s.y }],
        [{ x: s.x, y: t.y }],
        [{ x: midX, y: s.y }, { x: midX, y: t.y }],
        [{ x: s.x, y: midY }, { x: t.x, y: midY }],
        [{ x: s.x, y: laneTop }, { x: t.x, y: laneTop }],
        [{ x: s.x, y: laneBottom }, { x: t.x, y: laneBottom }],
        [{ x: laneLeft, y: s.y }, { x: laneLeft, y: t.y }],
        [{ x: laneRight, y: s.y }, { x: laneRight, y: t.y }],
      ];
      for (const middle of middles) {
        const route = tidyRoute([start, s, ...middle, t, end]);
        let cost = 0;
        let blocked = false;
        for (let index = 1; index < route.length; index += 1) {
          const p = route[index - 1]!;
          const q = route[index]!;
          // Never run back through either end card.
          if (segmentHitsRect(p, q, a, -1) || segmentHitsRect(p, q, b, -1)) {
            blocked = true;
            break;
          }
          for (const rect of obstacles) {
            if (segmentHitsRect(p, q, rect, ROUTE_CLEARANCE)) cost += 10_000;
          }
          for (const rect of foreign) {
            if (segmentHitsRect(p, q, rect, 0)) cost += 3_000;
          }
          // Inside an end's own frame, a short run to leave it is fine; travelling there is not.
          for (const rect of own) {
            cost += Math.max(0, lengthInside(p, q, rect) - ROUTE_OWN_FRAME_RUN) * 20;
          }
          cost += Math.abs(q.x - p.x) + Math.abs(q.y - p.y);
        }
        if (blocked) continue;
        cost += Math.max(0, route.length - 2) * 24;
        if (bounds) {
          for (const point of route) {
            if (
              point.x < bounds.x || point.x > bounds.x + bounds.width
              || point.y < bounds.y || point.y > bounds.y + bounds.height
            ) {
              cost += 2_000;
            }
          }
        }
        if (cost < bestCost) {
          bestCost = cost;
          best = route;
        }
      }
    }
  }
  return best ?? [routeAnchor(a, "right"), routeAnchor(b, "left")];
}

/** Where a domain's title pill sits inside its frame (width estimated from the name). */
export function domainPillRect(frame: Rect, name: string): Rect {
  const width = Math.min(NEST_PILL_CHROME + name.length * NEST_PILL_CHAR_W, frame.width - NEST_PAD * 2);
  return { x: frame.x + NEST_PAD, y: frame.y + NEST_PILL_TOP, width: Math.max(0, width), height: NEST_PILL_H };
}

/** Left edge of the card in slot `index` of a domain's row. */
export function nestSlotX(index: number): number {
  return index * NEST_CHILD_GAP;
}

/** A domain's cards in row order. Older free positions read top to bottom, then left to right. */
export function nestRowOrder<T extends { x: number; y: number }>(cards: readonly T[]): T[] {
  const row = (card: T) => Math.round(card.y / NEST_LEAF_H);
  return [...cards].sort((a, b) => row(a) - row(b) || a.x - b.x);
}

/** The slot a card dropped at local `dropX` takes among its siblings (row order, card excluded). */
export function nestDropIndex(siblings: readonly { x: number }[], dropX: number): number {
  return siblings.filter((sibling) => sibling.x < dropX).length;
}

/** Size of a domain frame holding `count` cards on its row (matches `layoutNest`). */
export function nestRowFrameSize(count: number): { width: number; height: number } {
  if (count === 0) return { width: NEST_LEAF_W, height: NEST_FRAME_MIN_H };
  return {
    width: nestSlotX(count - 1) + NEST_LEAF_W + NEST_PAD * 2,
    height: NEST_FRAME_MIN_H,
  };
}
