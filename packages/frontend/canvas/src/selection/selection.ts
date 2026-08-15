import type { ArchitectureDocument, Rect, SelectionState } from "../document/types";
import { rectContains, rectsIntersect } from "../viewport/viewport";

export function emptySelection(): SelectionState {
  return { nodeIds: [], relationshipIds: [], boundaryIds: [] };
}

export function selectClick(
  sel: SelectionState,
  hit: { kind: string; id: string },
  shift: boolean,
): SelectionState {
  if (hit.kind === "empty") {
    return shift ? sel : emptySelection();
  }
  const key =
    hit.kind === "node"
      ? "nodeIds"
      : hit.kind === "relationship"
        ? "relationshipIds"
        : hit.kind === "boundary"
          ? "boundaryIds"
          : null;
  if (!key) return sel;

  if (!shift) {
    return {
      nodeIds: key === "nodeIds" ? [hit.id] : [],
      relationshipIds: key === "relationshipIds" ? [hit.id] : [],
      boundaryIds: key === "boundaryIds" ? [hit.id] : [],
    };
  }

  const list = [...sel[key]];
  const idx = list.indexOf(hit.id);
  if (idx >= 0) list.splice(idx, 1);
  else list.push(hit.id);
  return { ...sel, [key]: list };
}

export type MarqueeMode = "contain" | "intersect";

export function selectMarquee(
  doc: ArchitectureDocument,
  marquee: Rect,
  mode: MarqueeMode,
): SelectionState {
  // normalize rect
  const r: Rect = {
    x: Math.min(marquee.x, marquee.x + marquee.width),
    y: Math.min(marquee.y, marquee.y + marquee.height),
    width: Math.abs(marquee.width),
    height: Math.abs(marquee.height),
  };

  const nodeIds: string[] = [];
  for (const n of doc.nodes) {
    const nr = { x: n.x, y: n.y, width: n.width, height: n.height };
    const ok = mode === "contain" ? rectContains(r, nr) : rectsIntersect(r, nr);
    if (ok) nodeIds.push(n.id);
  }

  const boundaryIds: string[] = [];
  for (const b of doc.boundaries) {
    const br = { x: b.x, y: b.y, width: b.width, height: b.height };
    const ok = mode === "contain" ? rectContains(r, br) : rectsIntersect(r, br);
    if (ok) boundaryIds.push(b.id);
  }

  return { nodeIds, relationshipIds: [], boundaryIds };
}

/** Left-to-right (positive width from drag) → contain; right-to-left → intersect */
export function marqueeModeFromDrag(startX: number, endX: number): MarqueeMode {
  return endX >= startX ? "contain" : "intersect";
}

export function selectAll(doc: ArchitectureDocument): SelectionState {
  return {
    nodeIds: doc.nodes.map((n) => n.id),
    relationshipIds: doc.relationships.map((r) => r.id),
    boundaryIds: doc.boundaries.map((b) => b.id),
  };
}

export function selectionCount(s: SelectionState): number {
  return s.nodeIds.length + s.relationshipIds.length + s.boundaryIds.length;
}
