import type { ArchitectureDocument, ArchNode, Relationship } from "@polterware/frontend-canvas";
import { nestChildren, placeNestRow } from "./atlas-map";
import {
  ATLAS_LEVEL_KEY,
  ATLAS_PARENT_KEY,
  NEST_LEAF_H,
  NEST_LEAF_W,
  NEST_PAD,
  nestRowFrameSize,
  nestSlotX,
} from "./nest-layout";

const ORIGIN = 48;
// Room between rows for the router's lanes.
const ROW_GAP = 120;
const COL_GAP = 96;
const UNIT_SWEEPS = 4;
const CARD_SWEEPS = 2;

type Unit = {
  id: string;
  width: number;
  height: number;
  x: number;
  y: number;
  /** A domain's cards in row order; empty for a loose component. */
  cards: string[];
};

type Weights = Map<string, Map<string, number>>;

function addWeight(weights: Weights, from: string, to: string, weight = 1): void {
  const row = weights.get(from) ?? new Map<string, number>();
  row.set(to, (row.get(to) ?? 0) + weight);
  weights.set(from, row);
}

function readingOrder(a: { x: number; y: number }, b: { x: number; y: number }): number {
  return a.y - b.y || a.x - b.x;
}

/**
 * Arranges the Atlas map for reading: domains and loose components fall into rows by
 * dependency (a card sits above what it uses), and each domain orders its cards
 * toward their outside neighbours. Only local slots change. With `visible`, only those
 * nodes are arranged (from the origin) and every other node keeps its slot.
 */
export function arrangeAtlas(
  local: ArchitectureDocument,
  edges: readonly Pick<Relationship, "sourceNodeId" | "targetNodeId">[],
  visible?: ReadonlySet<string> | null,
): ArchitectureDocument {
  const levelOf = new Map(local.nodes.map((node) => [node.id, String(node.metadata?.[ATLAS_LEVEL_KEY] ?? "")]));
  const parentOf = (node: ArchNode) => String(node.metadata?.[ATLAS_PARENT_KEY] ?? "");

  const units = new Map<string, Unit>();
  for (const node of local.nodes) {
    if (visible && !visible.has(node.id)) continue;
    const level = levelOf.get(node.id);
    if (level === "domain") {
      const cards = nestChildren(local, node.id).map((card) => card.id);
      units.set(node.id, { id: node.id, ...nestRowFrameSize(cards.length), x: node.x, y: node.y, cards });
    } else if (level === "component" && levelOf.get(parentOf(node)) !== "domain") {
      units.set(node.id, { id: node.id, width: NEST_LEAF_W, height: NEST_LEAF_H, x: node.x, y: node.y, cards: [] });
    }
  }
  if (units.size === 0) return local;

  const unitOf = new Map<string, string>();
  for (const unit of units.values()) {
    unitOf.set(unit.id, unit.id);
    for (const card of unit.cards) unitOf.set(card, unit.id);
  }

  // Unit graph, weighted by how many card edges it stands for.
  const out: Weights = new Map();
  const linked = new Map<string, Map<string, number>>();
  for (const edge of edges) {
    const from = unitOf.get(edge.sourceNodeId);
    const to = unitOf.get(edge.targetNodeId);
    if (!from || !to || from === to) continue;
    addWeight(out, from, to);
    addWeight(linked, from, to);
    addWeight(linked, to, from);
  }

  const byReading = [...units.values()].sort(readingOrder);
  const connected = byReading.filter((unit) => linked.has(unit.id));
  const isolated = byReading.filter((unit) => !linked.has(unit.id));

  // Break cycles: a DFS in reading order turns back edges around.
  const dagIn: Weights = new Map();
  const state = new Map<string, "open" | "done">();
  const visit = (id: string) => {
    state.set(id, "open");
    for (const [to, weight] of out.get(id) ?? []) {
      if (state.get(to) === "open") {
        addWeight(dagIn, id, to, weight);
        continue;
      }
      addWeight(dagIn, to, id, weight);
      if (!state.has(to)) visit(to);
    }
    state.set(id, "done");
  };
  for (const unit of connected) if (!state.has(unit.id)) visit(unit.id);

  // Longest-path layers: what a card uses sits below it.
  const layerOf = new Map<string, number>();
  const layer = (id: string): number => {
    const known = layerOf.get(id);
    if (known !== undefined) return known;
    layerOf.set(id, 0);
    let depth = 0;
    for (const from of dagIn.get(id)?.keys() ?? []) depth = Math.max(depth, layer(from) + 1);
    layerOf.set(id, depth);
    return depth;
  };
  const rows: Unit[][] = [];
  for (const unit of connected) {
    const index = layer(unit.id);
    (rows[index] ??= []).push(unit);
  }
  for (const row of rows) row?.sort((a, b) => a.x - b.x);
  const layered = rows.filter((row): row is Unit[] => Boolean(row?.length));
  const allRows = isolated.length ? [...layered, isolated] : layered;

  const place = () => {
    const widthOf = (row: Unit[]) => row.reduce((sum, unit) => sum + unit.width, 0) + COL_GAP * (row.length - 1);
    const widest = Math.max(...allRows.map(widthOf));
    let y = ORIGIN;
    for (const row of allRows) {
      let x = ORIGIN + (widest - widthOf(row)) / 2;
      for (const unit of row) {
        unit.x = Math.round(x);
        unit.y = y;
        x += unit.width + COL_GAP;
      }
      y += Math.max(...row.map((unit) => unit.height)) + ROW_GAP;
    }
  };

  const centreX = (unit: Unit) => unit.x + unit.width / 2;
  const barycenter = (unit: Unit): number => {
    let sum = 0;
    let weight = 0;
    for (const [id, count] of linked.get(unit.id) ?? []) {
      const other = units.get(id);
      if (!other) continue;
      sum += centreX(other) * count;
      weight += count;
    }
    return weight ? sum / weight : centreX(unit);
  };

  place();
  for (let sweep = 0; sweep < UNIT_SWEEPS; sweep += 1) {
    const order = sweep % 2 === 0 ? layered : [...layered].reverse();
    for (const row of order) {
      const keys = new Map(row.map((unit) => [unit.id, barycenter(unit)]));
      row.sort((a, b) => keys.get(a.id)! - keys.get(b.id)!);
      place();
    }
  }

  // Cards in a domain face their outside neighbours.
  const neighbours = new Map<string, string[]>();
  for (const edge of edges) {
    if (unitOf.get(edge.sourceNodeId) === unitOf.get(edge.targetNodeId)) continue;
    neighbours.set(edge.sourceNodeId, [...(neighbours.get(edge.sourceNodeId) ?? []), edge.targetNodeId]);
    neighbours.set(edge.targetNodeId, [...(neighbours.get(edge.targetNodeId) ?? []), edge.sourceNodeId]);
  }
  const worldX = (id: string): number | undefined => {
    const unit = units.get(unitOf.get(id) ?? "");
    if (!unit) return undefined;
    const index = unit.cards.indexOf(id);
    if (index < 0) return centreX(unit);
    return unit.x + NEST_PAD + nestSlotX(index) + NEST_LEAF_W / 2;
  };
  for (let sweep = 0; sweep < CARD_SWEEPS; sweep += 1) {
    for (const unit of units.values()) {
      if (unit.cards.length < 2) continue;
      const keys = new Map(unit.cards.map((card) => {
        const xs = (neighbours.get(card) ?? []).map(worldX).filter((x): x is number => x !== undefined);
        const own = worldX(card)!;
        return [card, xs.length ? xs.reduce((sum, x) => sum + x, 0) / xs.length : own];
      }));
      unit.cards.sort((a, b) => keys.get(a)! - keys.get(b)!);
    }
  }

  let next: ArchitectureDocument = {
    ...local,
    nodes: local.nodes.map((node) => {
      const unit = units.get(node.id);
      if (!unit || (node.x === unit.x && node.y === unit.y)) return node;
      return { ...node, x: unit.x, y: unit.y };
    }),
  };
  for (const unit of units.values()) {
    if (unit.cards.length) next = placeNestRow(next, unit.cards);
  }
  return next;
}
