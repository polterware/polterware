import { createDragMotion, type DragMotion } from "@polterware/frontend-design-system";
import { batch, createSignal } from "solid-js";
import type { CardId } from "./card-order";
import {
  GAP,
  ROW_H,
  cellAt,
  dropSide,
  freeSpot,
  gridMetrics,
  moveId,
  rectBox,
  resolveDrop,
  withoutAnchor,
  type Anchors,
  type Cell,
  type DropSide,
  type Rect,
} from "./card-layout";

/** Pointer travel before a press on a header becomes a drag. */
const DEAD_ZONE = 4;
const EDGE = 48;
const EDGE_STEP = 14;
/** A held card floats a touch higher than the shared default so it reads as above the surface. */
const LIFT_SCALE = 1.03;
/** How long the pointer must rest over a card side before the sequence reflows (home-screen feel). */
const DWELL_MS = 120;

const INTERACTIVE = "button:not([data-card-grip]), a, input, select, textarea, [role='combobox'], [role='button']";

export type CardReorder = {
  register: (id: CardId, element: HTMLElement) => () => void;
  draggingId: () => CardId | null;
  /** The card raised above the others: held, or still gliding back to its cell after release. */
  liftedId: () => CardId | null;
  onHeaderPointerDown: (id: CardId, event: PointerEvent) => void;
  onGripKeyDown: (id: CardId, event: KeyboardEvent) => void;
};

/** Viewport box a floating card is pinned to while it is in the top layer. */
type Box = { left: number; top: number; width: number; height: number };
type Entry = { element: HTMLElement; motion: DragMotion; floating: Box | null };
/** A side of a card to drop on, a free cell to anchor to, or the end of the sequence. */
type Target = { card: CardId; side: DropSide } | { cell: Cell } | "end";
type Layout = { order: CardId[]; anchors: Anchors };

const targetKey = (target: Target | null): string | null => {
  if (target === null || target === "end") return target;
  if ("cell" in target) return `cell:${target.cell.x},${target.cell.y}`;
  return `${target.card}:${target.side}`;
};

function scrollParent(element: HTMLElement): HTMLElement {
  for (let node = element.parentElement; node; node = node.parentElement) {
    const overflow = getComputedStyle(node).overflowY;
    if (overflow === "auto" || overflow === "scroll") return node;
  }
  return document.documentElement;
}

/**
 * Home-screen style reordering. Cards live in one sequence and their cells are derived from it.
 * Resting the pointer on a side of a card drops the dragged card on that side: top/left push the
 * target down/right, bottom/right let it move up/left (when the packing allows). The displaced
 * cards glide to their new cells (FLIP). Resting the card over a free cell anchors it there, so
 * gaps in the grid are drop targets too.
 */
export function createCardReorder(options: {
  grid: () => HTMLElement | undefined;
  cols: () => number;
  rects: () => Map<CardId, Rect>;
  order: () => CardId[];
  visible: () => CardId[];
  setOrder: (next: CardId[]) => void;
  anchors: () => Anchors;
  setAnchors: (next: Anchors) => void;
}): CardReorder {
  const entries = new Map<CardId, Entry>();
  const [draggingId, setDraggingId] = createSignal<CardId | null>(null);
  const [liftedId, setLiftedId] = createSignal<CardId | null>(null);

  /** Runs a layout change; returns how far `held` moved on screen (its transform is untouched). */
  const apply = (next: Layout, held: CardId | null): { dx: number; dy: number } => {
    const before = new Map<CardId, DOMRect>();
    for (const [id, entry] of entries) before.set(id, entry.element.getBoundingClientRect());
    batch(() => {
      options.setOrder(next.order);
      options.setAnchors(next.anchors);
    });
    let shift = { dx: 0, dy: 0 };
    for (const [id, entry] of entries) {
      const from = before.get(id);
      if (!from) continue;
      const to = entry.element.getBoundingClientRect();
      const dx = from.left - to.left;
      const dy = from.top - to.top;
      if (id === held) {
        shift = { dx, dy };
        continue;
      }
      if (dx === 0 && dy === 0) continue;
      entry.motion.follow(dx, dy);
      void entry.motion.settle();
    }
    return shift;
  };

  /** Viewport box of the cell the card occupies in the current order. */
  const homeBox = (id: CardId): Box | null => {
    const grid = options.grid();
    const rect = options.rects().get(id);
    if (!grid || !rect) return null;
    const { left, top, cellW } = gridMetrics(grid, options.cols());
    const box = rectBox(rect, cellW);
    return { left: left + box.left, top: top + box.top, width: box.width, height: box.height };
  };

  /**
   * Lifts the card into the top layer, pinned over its cell, so it floats above every surface
   * (the main surface clips and isolates its content, so no z-index can get it out).
   */
  const float = (id: CardId, entry: Entry) => {
    land(entry);
    const box = homeBox(id);
    const { element } = entry;
    if (!box || typeof element.showPopover !== "function") return;
    element.setAttribute("popover", "manual");
    element.style.left = `${box.left}px`;
    element.style.top = `${box.top}px`;
    element.style.width = `${box.width}px`;
    element.style.height = `${box.height}px`;
    element.showPopover();
    entry.floating = box;
  };

  /** Puts a floating card back into the grid; its cell must already sit where it is drawn. */
  const land = (entry: Entry) => {
    if (!entry.floating) return;
    const { element } = entry;
    entry.floating = null;
    if (element.matches(":popover-open")) element.hidePopover();
    element.removeAttribute("popover");
    element.style.left = element.style.top = element.style.width = element.style.height = "";
    entry.motion.follow(0, 0);
  };

  const insertAt = (id: CardId, target: Target): Layout => {
    const order = options.order();
    const anchors = options.anchors();
    if (target === "end") return { order: moveId(order, id, order.length), anchors: withoutAnchor(anchors, id) };
    if ("cell" in target) return { order, anchors: { ...anchors, [id]: target.cell } };
    return {
      order: resolveDrop(order, options.visible(), options.cols(), id, target.card, target.side, anchors),
      anchors: withoutAnchor(anchors, id),
    };
  };
  const currentLayout = (): Layout => ({ order: options.order(), anchors: options.anchors() });
  const sameLayout = (a: Layout, b: Layout) => {
    if (a.order.length !== b.order.length || a.order.some((value, index) => value !== b.order[index])) return false;
    const keys = new Set([...Object.keys(a.anchors), ...Object.keys(b.anchors)] as CardId[]);
    for (const key of keys) {
      const left = a.anchors[key];
      const right = b.anchors[key];
      if (left?.x !== right?.x || left?.y !== right?.y) return false;
    }
    return true;
  };

  const register = (id: CardId, element: HTMLElement) => {
    const entry: Entry = { element, motion: createDragMotion(element, { liftScale: LIFT_SCALE }), floating: null };
    entries.set(id, entry);
    return () => {
      entry.motion.stop();
      if (entries.get(id) === entry) entries.delete(id);
    };
  };

  const onHeaderPointerDown = (id: CardId, event: PointerEvent) => {
    if (event.button !== 0) return;
    if ((event.target as HTMLElement).closest(INTERACTIVE)) return;
    const entry = entries.get(id);
    if (!entry) return;
    const header = event.currentTarget as HTMLElement;
    const scroller = scrollParent(entry.element);
    const startLayout: Layout = { order: [...options.order()], anchors: { ...options.anchors() } };
    const startX = event.clientX;
    const startY = event.clientY;
    const startScroll = scroller.scrollTop;
    let x = startX;
    let y = startY;
    let shiftX = 0;
    let shiftY = 0;
    let active = false;
    let edgeFrame = 0;
    let dwellTimer = 0;
    let pending: string | null = null;
    /**
     * The card side just dropped on is ignored until the pointer leaves that card, so cards never
     * flip-flop; its opposite side stays live so the move can be undone in place.
     */
    let blocked: { card: CardId; key: string } | null = null;

    const follow = () => {
      // A floating card is pinned to the viewport, so scrolling the grid does not carry it.
      const scrolled = entry.floating ? 0 : scroller.scrollTop - startScroll;
      entry.motion.follow(x - startX + shiftX, y - startY + shiftY + scrolled);
    };

    /**
     * Pointer position in the grid's content box, plus the card (and which side of it) it is over.
     * Off every card, the free cell under the held card's top-left becomes the target.
     */
    const probe = (): { target: Target | null; over: CardId | null } => {
      const grid = options.grid();
      if (!grid) return { target: null, over: null };
      const cols = options.cols();
      const { left, top, cellW } = gridMetrics(grid, cols);
      const px = x - left;
      const py = y - top;
      const rects = options.rects();
      let bottom = 0;
      for (const [other, rect] of rects) {
        const box = rectBox(rect, cellW);
        bottom = Math.max(bottom, box.top + box.height);
        if (px < box.left || px > box.left + box.width || py < box.top || py > box.top + box.height) continue;
        return { target: other !== id ? { card: other, side: dropSide(box, px, py) } : null, over: other };
      }
      if (py > bottom + GAP) return { target: "end", over: null };
      const drawn = entry.element.getBoundingClientRect();
      const cell = {
        x: Math.floor((drawn.left - left + cellW / 2) / (cellW + GAP)),
        y: Math.floor((drawn.top - top + ROW_H / 2) / (ROW_H + GAP)),
      };
      const spot = cell.x >= 0 && cell.x < cols && cell.y >= 0 ? freeSpot(rects, id, cols, cell) : null;
      return { target: spot ? { cell: spot } : null, over: null };
    };

    const commit = (target: Target) => {
      dwellTimer = 0;
      pending = null;
      blocked = target === "end" || "cell" in target ? null : { card: target.card, key: `${target.card}:${target.side}` };
      const next = insertAt(id, target);
      if (sameLayout(next, currentLayout())) return;
      const shift = apply(next, id);
      shiftX += shift.dx;
      shiftY += shift.dy;
      follow();
    };

    const retarget = () => {
      const { target, over } = probe();
      if (blocked && over !== blocked.card) blocked = null;
      const next = target && targetKey(target) === blocked?.key ? null : target;
      const key = targetKey(next);
      if (key === pending) return;
      window.clearTimeout(dwellTimer);
      dwellTimer = 0;
      pending = key;
      if (!next) return;
      dwellTimer = window.setTimeout(() => {
        if (active) commit(next);
      }, DWELL_MS);
    };

    const pump = () => {
      edgeFrame = 0;
      if (!active) return;
      const bounds = scroller.getBoundingClientRect();
      const direction = y < bounds.top + EDGE ? -1 : y > bounds.bottom - EDGE ? 1 : 0;
      if (direction === 0) return;
      scroller.scrollTop += direction * EDGE_STEP;
      follow();
      retarget();
      edgeFrame = requestAnimationFrame(pump);
    };
    const onMove = (move: PointerEvent) => {
      if (move.pointerId !== event.pointerId) return;
      x = move.clientX;
      y = move.clientY;
      if (!active) {
        if (Math.hypot(x - startX, y - startY) < DEAD_ZONE) return;
        active = true;
        setDraggingId(id);
        setLiftedId(id);
        float(id, entry);
        entry.motion.lift(true);
        window.addEventListener("keydown", onKey);
      }
      follow();
      retarget();
      if (!edgeFrame) edgeFrame = requestAnimationFrame(pump);
    };
    const finish = (cancelled: boolean) => {
      header.removeEventListener("pointermove", onMove);
      header.removeEventListener("pointerup", onUp);
      header.removeEventListener("pointercancel", onCancel);
      window.removeEventListener("keydown", onKey);
      cancelAnimationFrame(edgeFrame);
      edgeFrame = 0;
      window.clearTimeout(dwellTimer);
      dwellTimer = 0;
      if (header.hasPointerCapture(event.pointerId)) header.releasePointerCapture(event.pointerId);
      if (!active) return;
      active = false;
      if (cancelled) {
        const shift = apply(startLayout, id);
        shiftX += shift.dx;
        shiftY += shift.dy;
        follow();
      }
      setDraggingId(null);
      entry.motion.lift(false);
      // A floating card glides from where it is drawn to its (possibly new) cell, then lands.
      const pinned = entry.floating;
      const home = pinned ? homeBox(id) : null;
      const toX = pinned && home ? home.left - pinned.left : 0;
      const toY = pinned && home ? home.top - pinned.top : 0;
      void entry.motion.settle(toX, toY).then(() => {
        if (draggingId() === id) return;
        land(entry);
        if (liftedId() === id) setLiftedId(null);
      });
    };
    const onUp = (up: PointerEvent) => {
      if (up.pointerId === event.pointerId) finish(false);
    };
    const onCancel = (cancel: PointerEvent) => {
      if (cancel.pointerId === event.pointerId) finish(true);
    };
    function onKey(key: KeyboardEvent) {
      if (key.key === "Escape") finish(true);
    }

    header.setPointerCapture(event.pointerId);
    header.addEventListener("pointermove", onMove);
    header.addEventListener("pointerup", onUp);
    header.addEventListener("pointercancel", onCancel);
  };

  const onGripKeyDown = (id: CardId, event: KeyboardEvent) => {
    const rects = options.rects();
    const rect = rects.get(id);
    if (!rect) return;
    const visible = options.visible();
    let card: CardId | null = null;
    let side: DropSide;
    if (event.key === "ArrowLeft" || event.key === "ArrowRight") {
      side = event.key === "ArrowLeft" ? "left" : "right";
      card = visible[visible.indexOf(id) + (side === "left" ? -1 : 1)] ?? null;
    } else if (event.key === "ArrowUp") {
      side = "top";
      card = cellAt(rects, { x: rect.x, y: rect.y - 1 });
    } else if (event.key === "ArrowDown") {
      side = "bottom";
      card = cellAt(rects, { x: rect.x, y: rect.y + rect.h });
    } else {
      return;
    }
    event.preventDefault();
    if (!card) return;
    const next = insertAt(id, { card, side });
    if (sameLayout(next, currentLayout())) return;
    apply(next, null);
    entries.get(id)?.element.scrollIntoView({ block: "nearest" });
  };

  return { register, draggingId, liftedId, onHeaderPointerDown, onGripKeyDown };
}
