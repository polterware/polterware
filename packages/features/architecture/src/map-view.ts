import type { CanvasModel, CanvasNode } from "@polterware/frontend-canvas";
import { assignLayer, layerBandY } from "./layers";
import type { ArchitectureLayer } from "./types";

export function groupNodesByLayer(nodes: CanvasNode[]): Record<ArchitectureLayer, CanvasNode[]> {
  const groups: Record<ArchitectureLayer, CanvasNode[]> = {
    cloud: [],
    applications: [],
    data: [],
    other: [],
  };
  for (const n of nodes) {
    groups[assignLayer(n)].push(n);
  }
  return groups;
}

/** Apply band Y hints when nodes lack spread (view-only; does not mutate evidence). */
export function withLayerHints(model: CanvasModel): CanvasModel {
  const nodes = model.nodes.map((n) => {
    if (n.y !== 0 || n.x !== 0) return n;
    const layer = assignLayer(n);
    return { ...n, y: layerBandY(layer) };
  });
  return { ...model, nodes };
}

export function filterNodes(model: CanvasModel, search: string): CanvasModel {
  const q = search.trim().toLowerCase();
  if (!q) return model;
  const nodes = model.nodes.filter((n) => n.label.toLowerCase().includes(q) || n.id.toLowerCase().includes(q));
  const ids = new Set(nodes.map((n) => n.id));
  const edges = model.edges.filter((e) => ids.has(e.from) && ids.has(e.to));
  return { ...model, nodes, edges };
}
