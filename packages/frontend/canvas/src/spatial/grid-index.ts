import type { Rect } from "../document/types";
import { rectsIntersect } from "../viewport/viewport";

export type IndexEntry = { id: string; kind: "node" | "boundary" | "relationship"; bounds: Rect };

export type GridIndex = {
  cellSize: number;
  cells: Map<string, IndexEntry[]>;
  entries: Map<string, IndexEntry>;
};

function cellKey(cx: number, cy: number): string {
  return `${cx},${cy}`;
}

export function createGridIndex(cellSize = 200): GridIndex {
  return { cellSize, cells: new Map(), entries: new Map() };
}

export function clearIndex(index: GridIndex): void {
  index.cells.clear();
  index.entries.clear();
}

export function upsertEntry(index: GridIndex, entry: IndexEntry): void {
  removeEntry(index, entry.id);
  index.entries.set(entry.id, entry);
  const { cellSize } = index;
  const x0 = Math.floor(entry.bounds.x / cellSize);
  const y0 = Math.floor(entry.bounds.y / cellSize);
  const x1 = Math.floor((entry.bounds.x + entry.bounds.width) / cellSize);
  const y1 = Math.floor((entry.bounds.y + entry.bounds.height) / cellSize);
  for (let cx = x0; cx <= x1; cx++) {
    for (let cy = y0; cy <= y1; cy++) {
      const k = cellKey(cx, cy);
      const list = index.cells.get(k) ?? [];
      list.push(entry);
      index.cells.set(k, list);
    }
  }
}

export function removeEntry(index: GridIndex, id: string): void {
  const prev = index.entries.get(id);
  if (!prev) return;
  index.entries.delete(id);
  for (const [k, list] of index.cells) {
    const next = list.filter((e) => e.id !== id);
    if (next.length) index.cells.set(k, next);
    else index.cells.delete(k);
  }
}

export function queryRect(index: GridIndex, rect: Rect): IndexEntry[] {
  const { cellSize } = index;
  const x0 = Math.floor(rect.x / cellSize);
  const y0 = Math.floor(rect.y / cellSize);
  const x1 = Math.floor((rect.x + rect.width) / cellSize);
  const y1 = Math.floor((rect.y + rect.height) / cellSize);
  const seen = new Set<string>();
  const out: IndexEntry[] = [];
  for (let cx = x0; cx <= x1; cx++) {
    for (let cy = y0; cy <= y1; cy++) {
      const list = index.cells.get(cellKey(cx, cy)) ?? [];
      for (const e of list) {
        if (seen.has(e.id)) continue;
        if (rectsIntersect(e.bounds, rect)) {
          seen.add(e.id);
          out.push(e);
        }
      }
    }
  }
  return out;
}
