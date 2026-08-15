import type { ArchitectureDocument } from "./types";
import { touchUpdated } from "./document";

export type AlignMode = "left" | "right" | "top" | "bottom" | "centerX" | "centerY";
export type DistributeMode = "horizontal" | "vertical";

/** Align selected nodes; locked nodes skipped. Returns doc unchanged if < 2 movable. */
export function alignNodes(
  doc: ArchitectureDocument,
  nodeIds: string[],
  mode: AlignMode,
): ArchitectureDocument {
  const nodes = doc.nodes.filter((n) => nodeIds.includes(n.id) && !n.locked);
  if (nodes.length < 2) return doc;

  let target = 0;
  switch (mode) {
    case "left":
      target = Math.min(...nodes.map((n) => n.x));
      break;
    case "right":
      target = Math.max(...nodes.map((n) => n.x + n.width));
      break;
    case "top":
      target = Math.min(...nodes.map((n) => n.y));
      break;
    case "bottom":
      target = Math.max(...nodes.map((n) => n.y + n.height));
      break;
    case "centerX": {
      const min = Math.min(...nodes.map((n) => n.x));
      const max = Math.max(...nodes.map((n) => n.x + n.width));
      target = (min + max) / 2;
      break;
    }
    case "centerY": {
      const min = Math.min(...nodes.map((n) => n.y));
      const max = Math.max(...nodes.map((n) => n.y + n.height));
      target = (min + max) / 2;
      break;
    }
  }

  const idSet = new Set(nodes.map((n) => n.id));
  return touchUpdated({
    ...doc,
    nodes: doc.nodes.map((n) => {
      if (!idSet.has(n.id) || n.locked) return n;
      switch (mode) {
        case "left":
          return { ...n, x: target };
        case "right":
          return { ...n, x: target - n.width };
        case "top":
          return { ...n, y: target };
        case "bottom":
          return { ...n, y: target - n.height };
        case "centerX":
          return { ...n, x: target - n.width / 2 };
        case "centerY":
          return { ...n, y: target - n.height / 2 };
        default:
          return n;
      }
    }),
  });
}

/** Evenly distribute selected nodes along an axis (≥3 movable). */
export function distributeNodes(
  doc: ArchitectureDocument,
  nodeIds: string[],
  mode: DistributeMode,
): ArchitectureDocument {
  const nodes = doc.nodes
    .filter((n) => nodeIds.includes(n.id) && !n.locked)
    .slice()
    .sort((a, b) => (mode === "horizontal" ? a.x - b.x : a.y - b.y));
  if (nodes.length < 3) return doc;

  if (mode === "horizontal") {
    const first = nodes[0]!;
    const last = nodes[nodes.length - 1]!;
    const span = last.x - first.x;
    const step = span / (nodes.length - 1);
    const positions = new Map(nodes.map((n, i) => [n.id, first.x + step * i]));
    return touchUpdated({
      ...doc,
      nodes: doc.nodes.map((n) => {
        const x = positions.get(n.id);
        return x !== undefined ? { ...n, x } : n;
      }),
    });
  }

  const first = nodes[0]!;
  const last = nodes[nodes.length - 1]!;
  const span = last.y - first.y;
  const step = span / (nodes.length - 1);
  const positions = new Map(nodes.map((n, i) => [n.id, first.y + step * i]));
  return touchUpdated({
    ...doc,
    nodes: doc.nodes.map((n) => {
      const y = positions.get(n.id);
      return y !== undefined ? { ...n, y } : n;
    }),
  });
}

/** Swap source/target of a relationship (reverse direction). */
export function reverseRelationship(
  doc: ArchitectureDocument,
  id: string,
): ArchitectureDocument {
  return touchUpdated({
    ...doc,
    relationships: doc.relationships.map((r) => {
      if (r.id !== id) return r;
      return {
        ...r,
        sourceNodeId: r.targetNodeId,
        targetNodeId: r.sourceNodeId,
        sourcePortId: r.targetPortId,
        targetPortId: r.sourcePortId,
      };
    }),
  });
}
