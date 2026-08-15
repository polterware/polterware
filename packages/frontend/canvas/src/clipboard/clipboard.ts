import type {
  ArchitectureDocument,
  ArchNode,
  Boundary,
  Relationship,
  SelectionState,
} from "../document/types";
import { newId, touchUpdated } from "../document/document";
import { sanitizeText } from "../document/serialize";

export type DocumentFragment = {
  formatVersion: 1;
  nodes: ArchNode[];
  relationships: Relationship[];
  boundaries: Boundary[];
};

export const CLIPBOARD_MIME = "application/x-polter-architecture-fragment+json";

export function buildFragment(
  doc: ArchitectureDocument,
  sel: SelectionState,
): DocumentFragment {
  const nodeSet = new Set(sel.nodeIds);
  const nodes = doc.nodes
    .filter((n) => nodeSet.has(n.id))
    .map((n) => structuredClone(n));
  const boundaries = doc.boundaries
    .filter((b) => sel.boundaryIds.includes(b.id))
    .map((b) => structuredClone(b));

  // only relationships with both ends in selection
  const relationships = doc.relationships
    .filter(
      (r) =>
        (sel.relationshipIds.includes(r.id) ||
          (nodeSet.has(r.sourceNodeId) && nodeSet.has(r.targetNodeId))) &&
        nodeSet.has(r.sourceNodeId) &&
        nodeSet.has(r.targetNodeId),
    )
    .map((r) => structuredClone(r));

  return { formatVersion: 1, nodes, relationships, boundaries };
}

export function pasteFragment(
  doc: ArchitectureDocument,
  fragment: DocumentFragment,
  offset: { x: number; y: number },
): { doc: ArchitectureDocument; newSelection: SelectionState } {
  const idMap = new Map<string, string>();
  for (const n of fragment.nodes) idMap.set(n.id, newId("node"));
  for (const b of fragment.boundaries) idMap.set(b.id, newId("boundary"));
  for (const r of fragment.relationships) idMap.set(r.id, newId("rel"));

  const nodes: ArchNode[] = fragment.nodes.map((n) => ({
    ...n,
    id: idMap.get(n.id)!,
    name: sanitizeText(n.name, 200),
    x: n.x + offset.x,
    y: n.y + offset.y,
    boundaryId: n.boundaryId && idMap.has(n.boundaryId) ? idMap.get(n.boundaryId)! : null,
  }));

  const boundaries: Boundary[] = fragment.boundaries.map((b) => ({
    ...b,
    id: idMap.get(b.id)!,
    name: sanitizeText(b.name, 200),
    x: b.x + offset.x,
    y: b.y + offset.y,
    memberNodeIds: (b.memberNodeIds ?? [])
      .map((id) => idMap.get(id))
      .filter(Boolean) as string[],
  }));

  const relationships: Relationship[] = fragment.relationships
    .map((r) => {
      const sourceNodeId = idMap.get(r.sourceNodeId);
      const targetNodeId = idMap.get(r.targetNodeId);
      if (!sourceNodeId || !targetNodeId) return null;
      return {
        ...r,
        id: idMap.get(r.id)!,
        sourceNodeId,
        targetNodeId,
        manualPoints: undefined,
        label: r.label ? sanitizeText(r.label, 200) : undefined,
      };
    })
    .filter(Boolean) as Relationship[];

  const next = touchUpdated({
    ...doc,
    nodes: [...doc.nodes, ...nodes],
    relationships: [...doc.relationships, ...relationships],
    boundaries: [...doc.boundaries, ...boundaries],
  });

  return {
    doc: next,
    newSelection: {
      nodeIds: nodes.map((n) => n.id),
      relationshipIds: relationships.map((r) => r.id),
      boundaryIds: boundaries.map((b) => b.id),
    },
  };
}

export function serializeFragment(f: DocumentFragment): string {
  return JSON.stringify(f);
}

export function parseFragment(json: string): DocumentFragment | null {
  try {
    const p = JSON.parse(json) as DocumentFragment;
    if (p.formatVersion !== 1 || !Array.isArray(p.nodes)) return null;
    return {
      formatVersion: 1,
      nodes: p.nodes ?? [],
      relationships: p.relationships ?? [],
      boundaries: p.boundaries ?? [],
    };
  } catch {
    return null;
  }
}
