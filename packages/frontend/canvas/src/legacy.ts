/** Legacy CanvasModel API kept for transitional consumers and tests. */

export type CanvasNode = {
  id: string;
  x: number;
  y: number;
  label: string;
  kind?: "system" | "resource" | "environment";
  source?: string;
  externalKey?: string;
};

export type CanvasEdge = {
  id: string;
  from: string;
  to: string;
  kind: "contains" | "related";
  evidence: boolean;
};

export type CanvasModel = {
  nodes: CanvasNode[];
  edges: CanvasEdge[];
  revision: number;
  selection: string[];
};

export function createEmptyCanvas(): CanvasModel {
  return { nodes: [], edges: [], revision: 0, selection: [] };
}

export function addNode(model: CanvasModel, node: CanvasNode): CanvasModel {
  return { ...model, nodes: [...model.nodes, node], revision: model.revision + 1 };
}

export function addEdge(model: CanvasModel, edge: CanvasEdge): CanvasModel {
  return { ...model, edges: [...model.edges, edge], revision: model.revision + 1 };
}

export function selectNodes(model: CanvasModel, ids: string[]): CanvasModel {
  return { ...model, selection: ids };
}

export function selectedNode(model: CanvasModel): CanvasNode | undefined {
  const id = model.selection[0];
  if (!id) return undefined;
  return model.nodes.find((n) => n.id === id);
}

export function serialize(model: CanvasModel): string {
  return JSON.stringify({ nodes: model.nodes, edges: model.edges, revision: model.revision });
}

export function deserialize(json: string): CanvasModel {
  if (!json || json === "{}") return createEmptyCanvas();
  try {
    const parsed = JSON.parse(json) as {
      nodes?: CanvasNode[];
      edges?: CanvasEdge[];
      revision?: number;
    };
    return {
      nodes: parsed.nodes ?? [],
      edges: parsed.edges ?? [],
      revision: parsed.revision ?? 0,
      selection: [],
    };
  } catch {
    return createEmptyCanvas();
  }
}

export function buildOperationalMap(
  systems: Array<{ id: string; name: string }>,
  resources: Array<{
    id: string;
    name: string;
    type?: string;
    source?: string;
    externalKey?: string;
    systemId?: string;
  }>,
): CanvasModel {
  const nodes: CanvasNode[] = [];
  const edges: CanvasEdge[] = [];
  systems.forEach((s, i) => {
    nodes.push({
      id: s.id,
      x: 40 + i * 220,
      y: 40,
      label: s.name,
      kind: "system",
    });
  });
  resources.forEach((r, i) => {
    const sysId = r.systemId ?? systems[0]?.id ?? "sys";
    nodes.push({
      id: r.id,
      x: 60 + (i % 4) * 160,
      y: 140 + Math.floor(i / 4) * 80,
      label: r.name,
      kind: "resource",
      source: r.source,
      externalKey: r.externalKey,
    });
    edges.push({
      id: `e-${sysId}-${r.id}`,
      from: sysId,
      to: r.id,
      kind: "contains",
      evidence: Boolean(r.source && r.source !== ""),
    });
  });
  return { nodes, edges, revision: 0, selection: [] };
}

export type HistoryStack = { past: CanvasModel[]; present: CanvasModel; future: CanvasModel[] };

export function pushHistory(h: HistoryStack, next: CanvasModel): HistoryStack {
  return { past: [...h.past, h.present], present: next, future: [] };
}

export function undo(h: HistoryStack): HistoryStack {
  if (h.past.length === 0) return h;
  const present = h.past[h.past.length - 1]!;
  return { past: h.past.slice(0, -1), present, future: [h.present, ...h.future] };
}
