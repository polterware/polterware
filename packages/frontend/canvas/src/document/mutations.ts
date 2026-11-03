import type {
  ArchitectureDocument,
  ArchNode,
  Boundary,
  NodeType,
  Relationship,
  RelationshipOrigin,
  RelationshipType,
  SelectionState,
} from "./types";
import { DEFAULT_NODE_H, DEFAULT_NODE_W } from "./types";
import { newId, touchUpdated } from "./document";
import { sanitizeText } from "./serialize";

export function emptySelection(): SelectionState {
  return { nodeIds: [], relationshipIds: [], boundaryIds: [] };
}

export function addNode(
  doc: ArchitectureDocument,
  partial: Partial<ArchNode> & { name: string; type: NodeType; x: number; y: number },
): ArchitectureDocument {
  const node: ArchNode = {
    id: partial.id ?? newId("node"),
    name: sanitizeText(partial.name, 200),
    description: partial.description,
    type: partial.type,
    subtype: partial.subtype,
    origin: partial.origin ?? "manual",
    provider: partial.provider,
    environmentIds: partial.environmentIds,
    state: partial.state ?? "planned",
    x: partial.x,
    y: partial.y,
    width: partial.width ?? DEFAULT_NODE_W,
    height: partial.height ?? DEFAULT_NODE_H,
    zIndex: partial.zIndex ?? doc.nodes.length,
    boundaryId: partial.boundaryId ?? null,
    locked: partial.locked ?? false,
    layoutPinned: partial.layoutPinned ?? false,
    ports: partial.ports,
    resourceId: partial.resourceId ?? null,
    externalKey: partial.externalKey ?? null,
    linkStatus: partial.linkStatus,
    owner: partial.owner,
    tags: partial.tags,
    links: partial.links,
    metadata: partial.metadata,
  };
  return touchUpdated({ ...doc, nodes: [...doc.nodes, node] });
}

export function updateNode(
  doc: ArchitectureDocument,
  id: string,
  patch: Partial<ArchNode>,
): ArchitectureDocument {
  return touchUpdated({
    ...doc,
    nodes: doc.nodes.map((n) =>
      n.id === id
        ? {
            ...n,
            ...patch,
            name: patch.name !== undefined ? sanitizeText(patch.name, 200) : n.name,
          }
        : n,
    ),
  });
}

export function moveNodes(
  doc: ArchitectureDocument,
  deltas: Map<string, { dx: number; dy: number }>,
): ArchitectureDocument {
  return touchUpdated({
    ...doc,
    nodes: doc.nodes.map((n) => {
      const d = deltas.get(n.id);
      if (!d || n.locked) return n;
      return { ...n, x: n.x + d.dx, y: n.y + d.dy };
    }),
  });
}

export function setNodePositions(
  doc: ArchitectureDocument,
  positions: Map<string, { x: number; y: number }>,
): ArchitectureDocument {
  return touchUpdated({
    ...doc,
    nodes: doc.nodes.map((n) => {
      const p = positions.get(n.id);
      if (!p || n.locked) return n;
      return { ...n, x: p.x, y: p.y };
    }),
  });
}

export function addRelationship(
  doc: ArchitectureDocument,
  partial: {
    sourceNodeId: string;
    targetNodeId: string;
    type: RelationshipType;
    sourcePortId?: string;
    targetPortId?: string;
    label?: string;
    evidence?: boolean;
    origin?: RelationshipOrigin;
  },
): ArchitectureDocument {
  const rel: Relationship = {
    id: newId("rel"),
    sourceNodeId: partial.sourceNodeId,
    targetNodeId: partial.targetNodeId,
    sourcePortId: partial.sourcePortId,
    targetPortId: partial.targetPortId,
    type: partial.type,
    label: partial.label ? sanitizeText(partial.label, 200) : undefined,
    direction: "forward",
    evidence: partial.evidence ?? false,
    origin: partial.origin ?? "manual",
    zIndex: doc.relationships.length,
  };
  return touchUpdated({ ...doc, relationships: [...doc.relationships, rel] });
}

export function updateRelationship(
  doc: ArchitectureDocument,
  id: string,
  patch: Partial<Relationship>,
): ArchitectureDocument {
  return touchUpdated({
    ...doc,
    relationships: doc.relationships.map((r) =>
      r.id === id
        ? {
            ...r,
            ...patch,
            label: patch.label !== undefined ? sanitizeText(patch.label, 200) : r.label,
          }
        : r,
    ),
  });
}

export function deleteSelection(
  doc: ArchitectureDocument,
  sel: SelectionState,
): ArchitectureDocument {
  const nodeSet = new Set(sel.nodeIds);
  const relSet = new Set(sel.relationshipIds);
  const boundSet = new Set(sel.boundaryIds);

  const nodes = doc.nodes
    .filter((n) => !nodeSet.has(n.id))
    .map((n) =>
      n.boundaryId && boundSet.has(n.boundaryId) ? { ...n, boundaryId: null } : n,
    );

  const relationships = doc.relationships.filter(
    (r) =>
      !relSet.has(r.id) &&
      !nodeSet.has(r.sourceNodeId) &&
      !nodeSet.has(r.targetNodeId),
  );

  const boundaries = doc.boundaries.filter((b) => !boundSet.has(b.id));

  return touchUpdated({ ...doc, nodes, relationships, boundaries });
}

export function addBoundary(
  doc: ArchitectureDocument,
  partial: Partial<Boundary> & { name: string; x: number; y: number; width: number; height: number },
  memberIds: string[] = [],
): ArchitectureDocument {
  const id = partial.id ?? newId("boundary");
  const boundary: Boundary = {
    id,
    name: sanitizeText(partial.name, 200),
    kind: partial.kind ?? "custom",
    x: partial.x,
    y: partial.y,
    width: partial.width,
    height: partial.height,
    zIndex: partial.zIndex ?? -1,
    collapsed: partial.collapsed ?? false,
    locked: partial.locked ?? false,
    memberNodeIds: memberIds,
  };
  const memberSet = new Set(memberIds);
  return touchUpdated({
    ...doc,
    boundaries: [...doc.boundaries, boundary],
    nodes: doc.nodes.map((n) =>
      memberSet.has(n.id) ? { ...n, boundaryId: id } : n,
    ),
  });
}

export function moveBoundaryWithMembers(
  doc: ArchitectureDocument,
  boundaryId: string,
  dx: number,
  dy: number,
  withMembers: boolean,
): ArchitectureDocument {
  const b = doc.boundaries.find((x) => x.id === boundaryId);
  if (!b || b.locked) return doc;
  const members = new Set(
    withMembers
      ? doc.nodes.filter((n) => n.boundaryId === boundaryId).map((n) => n.id)
      : [],
  );
  return touchUpdated({
    ...doc,
    boundaries: doc.boundaries.map((x) =>
      x.id === boundaryId ? { ...x, x: x.x + dx, y: x.y + dy } : x,
    ),
    nodes: doc.nodes.map((n) =>
      members.has(n.id) && !n.locked ? { ...n, x: n.x + dx, y: n.y + dy } : n,
    ),
  });
}

export function documentBounds(doc: ArchitectureDocument) {
  if (doc.nodes.length === 0 && doc.boundaries.length === 0) {
    return { minX: 0, minY: 0, maxX: 800, maxY: 600 };
  }
  let minX = Infinity;
  let minY = Infinity;
  let maxX = -Infinity;
  let maxY = -Infinity;
  for (const n of doc.nodes) {
    minX = Math.min(minX, n.x);
    minY = Math.min(minY, n.y);
    maxX = Math.max(maxX, n.x + n.width);
    maxY = Math.max(maxY, n.y + n.height);
  }
  for (const b of doc.boundaries) {
    minX = Math.min(minX, b.x);
    minY = Math.min(minY, b.y);
    maxX = Math.max(maxX, b.x + b.width);
    maxY = Math.max(maxY, b.y + b.height);
  }
  return { minX, minY, maxX, maxY };
}
