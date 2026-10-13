import { CARD_IDS, type CardId } from "./card-order";

/** Minimum cell width; the column count is however many of these fit the container. */
export const CELL_MIN_W = 260;
/** One row is one small card; every card is a whole number of rows, so cards stack in step. */
export const ROW_H = 192;
export const GAP = 12;

const STORAGE_KEY = "vitals.cardOrder";
/** Anchor cells are in row units, so the key changes with ROW_H. */
const ANCHORS_KEY = "vitals.cardAnchors.v3";

export type Cell = { x: number; y: number };
export type Rect = Cell & { w: number; h: number };
/** Cells a card was dropped on directly; anchored cards keep that spot instead of flowing. */
export type Anchors = Partial<Record<CardId, Cell>>;

/** Card sizes in cells; width is clamped to the column count at render time. */
const SIZES: Record<CardId, { w: number; h: number }> = {
  cpu: { w: 1, h: 2 },
  memory: { w: 1, h: 1 },
  network: { w: 1, h: 2 },
  diskio: { w: 2, h: 1 },
  pressure: { w: 1, h: 1 },
  disk: { w: 2, h: 1 },
  thermal: { w: 1, h: 1 },
  processes: { w: 2, h: 2 },
};

export function colsFor(contentWidth: number): number {
  return Math.max(1, Math.floor((contentWidth + GAP) / (CELL_MIN_W + GAP)));
}

export function sizeOf(id: CardId, cols: number): { w: number; h: number } {
  const size = SIZES[id];
  return { w: Math.min(size.w, cols), h: size.h };
}

const overlaps = (a: Rect, b: Rect) => a.x < b.x + b.w && b.x < a.x + a.w && a.y < b.y + b.h && b.y < a.y + a.h;

/**
 * Packs cards in sequence order: anchored cards take their cell first (clamped to the column
 * count, and only while it is still free), then every other card takes the first free spot
 * scanning left to right, then top to bottom, like icons on a home screen.
 */
export function packOrder(ids: readonly CardId[], cols: number, anchors: Anchors = {}): Map<CardId, Rect> {
  const placed: Rect[] = [];
  const out = new Map<CardId, Rect>();
  const flowing: CardId[] = [];
  for (const id of ids) {
    const anchor = anchors[id];
    if (!anchor) {
      flowing.push(id);
      continue;
    }
    const { w, h } = sizeOf(id, cols);
    const rect = { x: Math.max(0, Math.min(anchor.x, cols - w)), y: Math.max(0, anchor.y), w, h };
    if (placed.some((other) => overlaps(other, rect))) {
      flowing.push(id);
      continue;
    }
    placed.push(rect);
    out.set(id, rect);
  }
  for (const id of flowing) {
    const { w, h } = sizeOf(id, cols);
    let rect: Rect | undefined;
    for (let y = 0; !rect; y++) {
      for (let x = 0; x + w <= cols; x++) {
        const candidate = { x, y, w, h };
        if (!placed.some((other) => overlaps(other, candidate))) {
          rect = candidate;
          break;
        }
      }
    }
    placed.push(rect);
    out.set(id, rect);
  }
  return out;
}

/**
 * Where `id` would sit if dropped with its top-left on `cell`: the cell clamped so the card fits
 * the columns, or null when another card already covers any part of that spot.
 */
export function freeSpot(rects: ReadonlyMap<CardId, Rect>, id: CardId, cols: number, cell: Cell): Cell | null {
  const { w, h } = sizeOf(id, cols);
  const rect = { x: Math.max(0, Math.min(cell.x, cols - w)), y: Math.max(0, cell.y), w, h };
  for (const [other, placed] of rects) {
    if (other !== id && overlaps(placed, rect)) return null;
  }
  return { x: rect.x, y: rect.y };
}

/**
 * Free cells of the grid while `id` is held, down to the bottom of the packed cards. A cell is one
 * small card, so each free cell is a whole drop spot. The held card's own cells count as free: it
 * can be dropped back there.
 */
export function freeCells(rects: ReadonlyMap<CardId, Rect>, id: CardId, cols: number): Cell[] {
  let bottom = 0;
  for (const rect of rects.values()) bottom = Math.max(bottom, rect.y + rect.h);
  const cells: Cell[] = [];
  for (let y = 0; y < bottom; y++) {
    for (let x = 0; x < cols; x++) {
      const owner = cellAt(rects, { x, y });
      if (owner === null || owner === id) cells.push({ x, y });
    }
  }
  return cells;
}

/** Anchors without `id`, for when a card goes back to flowing with the sequence. */
export function withoutAnchor(anchors: Anchors, id: CardId): Anchors {
  if (!anchors[id]) return anchors;
  const next = { ...anchors };
  delete next[id];
  return next;
}

/** Card whose cells include `cell`, if any. */
export function cellAt(rects: ReadonlyMap<CardId, Rect>, cell: Cell): CardId | null {
  for (const [id, rect] of rects) {
    if (cell.x >= rect.x && cell.x < rect.x + rect.w && cell.y >= rect.y && cell.y < rect.y + rect.h) return id;
  }
  return null;
}

/** Pixel box of a rect inside the grid's content box. */
export function rectBox(rect: Rect, cellW: number): { left: number; top: number; width: number; height: number } {
  return {
    left: rect.x * (cellW + GAP),
    top: rect.y * (ROW_H + GAP),
    width: rect.w * cellW + (rect.w - 1) * GAP,
    height: rect.h * ROW_H + (rect.h - 1) * GAP,
  };
}

/** Screen geometry of the grid's content box, used to turn a pointer position into a cell. */
export function gridMetrics(grid: HTMLElement, cols: number): { left: number; top: number; cellW: number } {
  const rect = grid.getBoundingClientRect();
  const style = getComputedStyle(grid);
  const padL = parseFloat(style.paddingLeft) || 0;
  const padR = parseFloat(style.paddingRight) || 0;
  const padT = parseFloat(style.paddingTop) || 0;
  const content = rect.width - padL - padR;
  return { left: rect.left + padL, top: rect.top + padT, cellW: (content - GAP * (cols - 1)) / cols };
}

/** Moves `id` so it sits at `toIndex` of the returned list. */
export function moveId<T>(ids: readonly T[], id: T, toIndex: number): T[] {
  const from = ids.indexOf(id);
  if (from < 0) return [...ids];
  const next = ids.filter((candidate) => candidate !== id);
  next.splice(Math.max(0, Math.min(toIndex, next.length)), 0, id);
  return next;
}

export type DropSide = "top" | "bottom" | "left" | "right";

/** Which side of `box` the point falls on: the axis where it sits furthest from the center wins. */
export function dropSide(box: { left: number; top: number; width: number; height: number }, px: number, py: number): DropSide {
  const u = (px - (box.left + box.width / 2)) / (box.width / 2 || 1);
  const v = (py - (box.top + box.height / 2)) / (box.height / 2 || 1);
  if (Math.abs(u) > Math.abs(v)) return u < 0 ? "left" : "right";
  return v < 0 ? "top" : "bottom";
}

/**
 * Next full order after dropping `id` on `side` of `target`. Left/top insert before the target
 * (it shifts right/down); right/bottom insert after it. For top/bottom, nearby insertion points are
 * tried until the packed layout really puts the card above/below the target, falling back to the
 * plain before/after insertion when the packing cannot. Hidden cards keep their slots in `order`.
 */
export function resolveDrop(
  order: readonly CardId[],
  visible: readonly CardId[],
  cols: number,
  id: CardId,
  target: CardId,
  side: DropSide,
  anchors: Anchors = {},
): CardId[] {
  const rest = visible.filter((other) => other !== id);
  const at = rest.indexOf(target);
  if (at < 0) return [...order];
  const after = side === "right" || side === "bottom";
  let index = after ? at + 1 : at;
  if (side === "top" || side === "bottom") {
    const candidates: number[] = [];
    if (after) for (let i = at + 1; i <= rest.length; i++) candidates.push(i);
    else for (let i = at; i >= 0; i--) candidates.push(i);
    for (const candidate of candidates) {
      const sequence = [...rest];
      sequence.splice(candidate, 0, id);
      const packed = packOrder(sequence, cols, withoutAnchor(anchors, id));
      const moved = packed.get(id);
      const anchor = packed.get(target);
      if (!moved || !anchor) continue;
      if (after ? moved.y >= anchor.y + anchor.h : moved.y + moved.h <= anchor.y) {
        index = candidate;
        break;
      }
    }
  }
  const full = order.filter((other) => other !== id);
  const toIndex = index < rest.length ? full.indexOf(rest[index]) : rest.length ? full.indexOf(rest[rest.length - 1]) + 1 : full.length;
  return moveId(order, id, toIndex);
}

/** Keeps known ids in saved order and appends any card the saved list does not know yet. */
export function normalizeOrder(saved: unknown): CardId[] {
  const known = new Set<string>(CARD_IDS);
  const seen = new Set<string>();
  const order: CardId[] = [];
  if (Array.isArray(saved)) {
    for (const id of saved) {
      if (typeof id === "string" && known.has(id) && !seen.has(id)) {
        seen.add(id);
        order.push(id as CardId);
      }
    }
  }
  for (const id of CARD_IDS) if (!seen.has(id)) order.push(id);
  return order;
}

export function loadOrder(): CardId[] {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    return normalizeOrder(raw ? JSON.parse(raw) : null);
  } catch {
    return normalizeOrder(null);
  }
}

export function saveOrder(order: readonly CardId[]): void {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(order));
  } catch {
    // Persistence is a convenience; the order still works for this session.
  }
}

/** Keeps anchors of known cards whose cell is a pair of non-negative integers. */
export function normalizeAnchors(saved: unknown): Anchors {
  const anchors: Anchors = {};
  if (!saved || typeof saved !== "object") return anchors;
  const known = new Set<string>(CARD_IDS);
  for (const [id, cell] of Object.entries(saved as Record<string, unknown>)) {
    if (!known.has(id) || !cell || typeof cell !== "object") continue;
    const { x, y } = cell as Record<string, unknown>;
    if (Number.isInteger(x) && Number.isInteger(y) && (x as number) >= 0 && (y as number) >= 0) {
      anchors[id as CardId] = { x: x as number, y: y as number };
    }
  }
  return anchors;
}

export function loadAnchors(): Anchors {
  try {
    const raw = localStorage.getItem(ANCHORS_KEY);
    return normalizeAnchors(raw ? JSON.parse(raw) : null);
  } catch {
    return {};
  }
}

export function saveAnchors(anchors: Anchors): void {
  try {
    localStorage.setItem(ANCHORS_KEY, JSON.stringify(anchors));
  } catch {
    // Persistence is a convenience; the layout still works for this session.
  }
}
