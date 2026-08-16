import type {
  ArchitectureDocument,
  ArchNode,
  Boundary,
  Relationship,
  SelectionState,
} from "../document/types";
import { newId, touchUpdated } from "../document/document";
import { deserializeAndMigrate, sanitizeText } from "../document/serialize";

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
) {
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
      .filter((id): id is string => id !== undefined),
  }));

  const relationships: Relationship[] = fragment.relationships
    .flatMap((r): Relationship[] => {
      const sourceNodeId = idMap.get(r.sourceNodeId);
      const targetNodeId = idMap.get(r.targetNodeId);
      if (!sourceNodeId || !targetNodeId) return [];
      return [{
        ...r,
        id: idMap.get(r.id)!,
        sourceNodeId,
        targetNodeId,
        manualPoints: undefined,
        label: r.label ? sanitizeText(r.label, 200) : undefined,
      }];
    });

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

import type { JsonObject, JsonValue } from "../document/types";

function isJsonObject(value: JsonValue): value is JsonObject {
  return value !== null && !Array.isArray(value) && Object(value) === value;
}

export function parseFragment(json: string): DocumentFragment | null {
  try {
    const p: JsonValue = JSON.parse(json);
    if (!isJsonObject(p)) return null;
    if (p.formatVersion !== 1 || !Array.isArray(p.nodes)) return null;
    const wrappedDocument = {
      formatVersion: 1,
      systemId: "clipboard",
      revision: 0,
      createdAt: "1970-01-01T00:00:00.000Z",
      updatedAt: "1970-01-01T00:00:00.000Z",
      nodes: p.nodes,
      relationships: Array.isArray(p.relationships) ? p.relationships : [],
      boundaries: Array.isArray(p.boundaries) ? p.boundaries : [],
    };
    const parsed = deserializeAndMigrate(JSON.stringify(wrappedDocument), "clipboard");
    if (parsed.warnings.some((warning) => warning.level === "error")) return null;
    return {
      formatVersion: 1,
      nodes: parsed.doc.nodes,
      relationships: parsed.doc.relationships,
      boundaries: parsed.doc.boundaries,
    };
  } catch {
    return null;
  }
}
