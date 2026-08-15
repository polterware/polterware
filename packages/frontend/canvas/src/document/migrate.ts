import type {
  ArchitectureDocument,
  ArchNode,
  NodeType,
  Relationship,
  RelationshipType,
} from "./types";
import { DEFAULT_NODE_H, DEFAULT_NODE_W } from "./types";
import { createEmptyDocument, defaultViewport } from "./document";

/** Legacy format 0 body (pre-engine). */
export type LegacyCanvasBody = {
  nodes?: Array<{
    id: string;
    x: number;
    y: number;
    label: string;
    kind?: string;
    source?: string;
    externalKey?: string;
  }>;
  edges?: Array<{
    id: string;
    from: string;
    to: string;
    kind?: string;
    evidence?: boolean;
  }>;
  revision?: number;
  formatVersion?: number;
};

function mapLegacyNodeType(kind?: string): NodeType {
  if (kind === "system") return "system";
  if (kind === "environment") return "custom_resource";
  return "custom_resource";
}

function mapLegacyRelType(kind?: string): RelationshipType {
  if (kind === "contains") return "contains";
  if (kind === "related") return "related";
  return "related";
}

export function migrateLegacyToV1(
  body: LegacyCanvasBody,
  systemId = "sys-local",
): ArchitectureDocument {
  const base = createEmptyDocument({ systemId, id: `layout-${systemId}` });
  const nodes: ArchNode[] = (body.nodes ?? []).map((n, i) => ({
    id: n.id,
    name: n.label || n.id,
    type: mapLegacyNodeType(n.kind),
    origin: n.source ? "discovered" : "manual",
    state: "unknown",
    x: Number.isFinite(n.x) ? n.x : 40 + (i % 4) * 160,
    y: Number.isFinite(n.y) ? n.y : 40 + Math.floor(i / 4) * 80,
    width: DEFAULT_NODE_W,
    height: DEFAULT_NODE_H,
    zIndex: i,
    externalKey: n.externalKey ?? null,
    resourceId: n.kind === "resource" ? n.id : null,
    metadata: n.kind ? { legacyKind: n.kind } : undefined,
  }));

  const relationships: Relationship[] = (body.edges ?? []).map((e, i) => ({
    id: e.id || `rel_${i}`,
    sourceNodeId: e.from,
    targetNodeId: e.to,
    type: mapLegacyRelType(e.kind),
    direction: "forward" as const,
    evidence: Boolean(e.evidence),
    origin: "imported",
    zIndex: i,
  }));

  return {
    ...base,
    nodes,
    relationships,
    viewport: defaultViewport(),
    meta: { migratedFrom: 0, generator: "migrateLegacyToV1" },
  };
}

export function isArchitectureDocumentV1(value: unknown): value is ArchitectureDocument {
  if (!value || typeof value !== "object") return false;
  const v = value as ArchitectureDocument;
  return v.formatVersion === 1 && Array.isArray(v.nodes) && Array.isArray(v.relationships);
}
