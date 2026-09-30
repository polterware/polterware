import type { Point, Rect } from "@polterware/frontend-canvas";

/** Header row of a map card. Children sit in the well below it. */
export const NEST_HEADER = 56;
/** Label band of a domain frame. The frame is a border, not a card. */
export const NEST_FRAME_HEADER = 28;
export const NEST_PAD = 16;
export const NEST_LEAF_W = 188;
export const NEST_LEAF_H = 56;

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
        ? { width: NEST_LEAF_W, height: header + NEST_PAD * 2 }
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
      height: header + NEST_PAD + maxY + NEST_PAD,
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
  const hit = hitNestNode(input.nodes, input.cursor, input.skip);
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
