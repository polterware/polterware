import type { ArchitectureDocument, Viewport } from "./types";
import { DEFAULT_NODE_H, DEFAULT_NODE_W } from "./types";

export type ValidationIssue = { level: "error" | "warning"; message: string; path?: string };

export function defaultViewport(): Viewport {
  return { x: 0, y: 0, zoom: 1, focusedElementId: null };
}

export function createEmptyDocument(meta: {
  id?: string;
  systemId: string;
  workspaceId?: string;
}): ArchitectureDocument {
  const now = new Date().toISOString();
  return {
    formatVersion: 1,
    id: meta.id ?? `layout-${meta.systemId}`,
    systemId: meta.systemId,
    workspaceId: meta.workspaceId,
    activeEnvironmentId: "all",
    nodes: [],
    relationships: [],
    boundaries: [],
    preferences: {
      gridEnabled: false,
      snapToGrid: true,
      snapToObjects: true,
      gridSize: 16,
      showMinimap: true,
      edgeRouting: "orthogonal",
    },
    viewport: defaultViewport(),
    createdAt: now,
    updatedAt: now,
  };
}

export function defaultNodeSize() {
  return { width: DEFAULT_NODE_W, height: DEFAULT_NODE_H };
}

export function validateDocument(doc: ArchitectureDocument): ValidationIssue[] {
  const issues: ValidationIssue[] = [];
  if (doc.formatVersion !== 1) {
    issues.push({ level: "error", message: `Unknown formatVersion ${String(doc.formatVersion)}` });
  }
  if (!doc.systemId) issues.push({ level: "error", message: "systemId required" });

  const nodeIds = new Set<string>();
  for (const n of doc.nodes) {
    if (nodeIds.has(n.id)) issues.push({ level: "error", message: `Duplicate node id ${n.id}` });
    nodeIds.add(n.id);
    if (!Number.isFinite(n.x) || !Number.isFinite(n.y)) {
      issues.push({ level: "error", message: `Non-finite position on node ${n.id}` });
    }
  }

  const boundaryIds = new Set(doc.boundaries.map((b) => b.id));
  for (const n of doc.nodes) {
    if (n.boundaryId && !boundaryIds.has(n.boundaryId)) {
      issues.push({ level: "warning", message: `Node ${n.id} references missing boundary ${n.boundaryId}` });
    }
  }

  const relIds = new Set<string>();
  for (const r of doc.relationships) {
    if (relIds.has(r.id)) issues.push({ level: "error", message: `Duplicate relationship id ${r.id}` });
    relIds.add(r.id);
    if (!nodeIds.has(r.sourceNodeId) || !nodeIds.has(r.targetNodeId)) {
      issues.push({
        level: "warning",
        message: `Relationship ${r.id} has unresolved endpoints`,
      });
    }
    if (
      r.manualPoints !== undefined &&
      (!Array.isArray(r.manualPoints) ||
        r.manualPoints.some((p) => !Number.isFinite(p.x) || !Number.isFinite(p.y)))
    ) {
      issues.push({
        level: "warning",
        message: `Relationship ${r.id} has invalid manual route points`,
      });
    }
  }

  if (!Number.isFinite(doc.viewport.zoom) || doc.viewport.zoom <= 0) {
    issues.push({ level: "error", message: "Invalid viewport.zoom" });
  }

  return issues;
}

export function touchUpdated(doc: ArchitectureDocument): ArchitectureDocument {
  return { ...doc, updatedAt: new Date().toISOString() };
}

export function newId(prefix = "id"): string {
  return `${prefix}_${Math.random().toString(36).slice(2, 10)}_${Date.now().toString(36)}`;
}
