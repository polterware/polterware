import type { ArchitectureDocument, ArchNode, NodeState, NodeType, Relationship } from "@polterware/frontend-canvas";
import type { AtlasCanvasNode } from "./atlas-canvas";
import {
  ATLAS_LEVEL_KEY,
  ATLAS_PARENT_KEY,
  NEST_CONTENT_H,
  NEST_CONTENT_W,
  NEST_CONTENT_X,
  NEST_CONTENT_Y,
  NEST_LEAF_H,
  NEST_LEAF_W,
  isNestLevel,
  layoutNest,
  type NestLevel,
  type NestSlot,
} from "./nest-layout";

const DOMAIN_STRIDE = 880;
/** Relationship metadata flag: the edge comes from a node's `uses:` in the model. */
export const ATLAS_USES_KEY = "atlasUses";

export function isAtlasUsesEdge(edge: Pick<Relationship, "metadata">): boolean {
  return edge.metadata?.[ATLAS_USES_KEY] === true;
}
// Room between sibling cards so the edges between them read.
const CHILD_GAP = NEST_LEAF_W + 56;

function nestState(status: AtlasCanvasNode["status"]): NodeState {
  if (status === "proposed") return "planned";
  if (status === "delivered") return "linked";
  if (status === "drift") return "drifted";
  return "observed";
}

function nestType(level: NestLevel): { type: NodeType; subtype?: string } {
  if (level === "system") return { type: "system" };
  if (level === "domain") return { type: "custom_resource", subtype: "domain" };
  return { type: "service" };
}

function slotOf(
  node: ArchNode | undefined,
  level: NestLevel,
  parentId: string,
  rootIndex: number,
  childIndex: number,
): { x: number; y: number } {
  if (node && isNestLevel(String(node.metadata?.[ATLAS_LEVEL_KEY] ?? ""))) {
    return { x: node.x, y: node.y };
  }
  if (level === "system") return { x: 0, y: 0 };
  if (level === "domain" || !parentId) return { x: 48 + rootIndex * DOMAIN_STRIDE, y: 48 };
  return { x: childIndex * CHILD_GAP, y: 0 };
}

function sameNode(a: ArchNode, b: ArchNode): boolean {
  return a.id === b.id
    && a.name === b.name
    && a.x === b.x
    && a.y === b.y
    && a.state === b.state
    && a.type === b.type
    && a.metadata?.[ATLAS_PARENT_KEY] === b.metadata?.[ATLAS_PARENT_KEY]
    && a.metadata?.[ATLAS_LEVEL_KEY] === b.metadata?.[ATLAS_LEVEL_KEY];
}

/** Local slots for the atlas nodes. Layout geometry stays on these nodes; the world rects are projected. */
export function mergeAtlasSlots(
  atlas: readonly AtlasCanvasNode[],
  local: ArchitectureDocument,
  parentOverrides: Readonly<Record<string, string>>,
): ArchitectureDocument {
  const previous = new Map(local.nodes.map((node) => [node.id, node]));
  const levelOf = new Map(atlas.map((node) => [node.id, node.level]));
  const childIndex = new Map<string, number>();
  let rootIndex = 0;
  const nodes: ArchNode[] = atlas.map((node) => {
    const parentId = parentOverrides[node.id] ?? node.parentId ?? "";
    const saved = previous.get(node.id);
    // A component under its system (outside any domain) sits loose on the canvas.
    const visualRoot = node.level === "domain"
      || (node.level !== "system" && (!parentId || levelOf.get(parentId) === "system"));
    const index = node.level === "system" ? 0 : visualRoot ? rootIndex : (childIndex.get(parentId) ?? 0);
    if (visualRoot) rootIndex += 1;
    else if (node.level !== "system") childIndex.set(parentId, index + 1);
    const slot = slotOf(saved, node.level, parentId, index, index);
    const typed = nestType(node.level);
    return {
      id: node.id,
      name: node.title,
      type: typed.type,
      subtype: typed.subtype,
      origin: "manual",
      state: nestState(node.status),
      x: slot.x,
      y: slot.y,
      width: NEST_LEAF_W,
      height: NEST_LEAF_H,
      zIndex: 1,
      boundaryId: null,
      metadata: {
        [ATLAS_LEVEL_KEY]: node.level,
        [ATLAS_PARENT_KEY]: parentId,
      },
    };
  });
  const ids = new Set(nodes.map((node) => node.id));
  const relationships = local.relationships.filter(
    (edge) => ids.has(edge.sourceNodeId) && ids.has(edge.targetNodeId) && !isAtlasUsesEdge(edge),
  );
  const unchanged = nodes.length === local.nodes.length
    && nodes.every((node, index) => sameNode(node, local.nodes[index]!))
    && relationships.length === local.relationships.length
    && local.boundaries.length === 0;
  if (unchanged) return local;
  return { ...local, nodes, relationships, boundaries: [] };
}

function slotsFrom(local: ArchitectureDocument): NestSlot[] {
  return local.nodes.flatMap((node) => {
    const level = String(node.metadata?.[ATLAS_LEVEL_KEY] ?? "");
    if (!isNestLevel(level)) return [];
    return [{
      id: node.id,
      parentId: String(node.metadata?.[ATLAS_PARENT_KEY] ?? ""),
      level,
      x: node.x,
      y: node.y,
    }];
  });
}

/** World positions for the canvas. Stored x/y stay local slots. */
export function projectAtlasWorld(local: ArchitectureDocument): ArchitectureDocument {
  const rects = layoutNest(slotsFrom(local));
  return {
    ...local,
    boundaries: [],
    nodes: local.nodes.map((node) => {
      const rect = rects.get(node.id);
      if (!rect) return node;
      return {
        ...node,
        x: rect.x,
        y: rect.y,
        width: rect.width,
        height: rect.height,
        zIndex: rect.depth + 1,
        metadata: {
          ...node.metadata,
          [NEST_CONTENT_X]: rect.content.x,
          [NEST_CONTENT_Y]: rect.content.y,
          [NEST_CONTENT_W]: rect.content.width,
          [NEST_CONTENT_H]: rect.content.height,
        },
      };
    }),
  };
}

function nearly(a: number, b: number): boolean {
  return Math.abs(a - b) < 0.5;
}

/** Turn a canvas edit in world space back into local slots. */
export function absorbNestEdit(
  local: ArchitectureDocument,
  world: ArchitectureDocument,
): ArchitectureDocument {
  const rects = layoutNest(slotsFrom(local));
  const worldById = new Map(world.nodes.map((node) => [node.id, node]));
  let moved = false;
  const nodes = local.nodes.map((node) => {
    const placed = worldById.get(node.id);
    if (!placed) return node;
    const parentId = String(node.metadata?.[ATLAS_PARENT_KEY] ?? "");
    const parent = parentId ? rects.get(parentId) : undefined;
    const x = parent ? Math.max(0, placed.x - parent.content.x) : placed.x;
    const y = parent ? Math.max(0, placed.y - parent.content.y) : placed.y;
    if (!nearly(x, node.x) || !nearly(y, node.y)) moved = true;
    if (nearly(x, node.x) && nearly(y, node.y)) return node;
    return { ...node, x, y };
  });
  const ids = new Set(local.nodes.map((node) => node.id));
  // Model edges are projected from uses:, never stored in the layout.
  const relationships = world.relationships.filter(
    (edge) => ids.has(edge.sourceNodeId) && ids.has(edge.targetNodeId) && !isAtlasUsesEdge(edge),
  );
  const sameEdges = relationships.length === local.relationships.length
    && relationships.every((edge, index) => edge.id === local.relationships[index]?.id && edge.type === local.relationships[index]?.type);
  const sameView = world.viewport.x === local.viewport.x
    && world.viewport.y === local.viewport.y
    && world.viewport.zoom === local.viewport.zoom;
  if (!moved && sameEdges && sameView) return local;
  return { ...local, nodes, relationships, viewport: world.viewport };
}

export function writeNestSlot(
  local: ArchitectureDocument,
  id: string,
  slot: { x: number; y: number },
  parentId?: string,
): ArchitectureDocument {
  return {
    ...local,
    nodes: local.nodes.map((node) => {
      if (node.id !== id) return node;
      const metadata = { ...node.metadata };
      if (parentId !== undefined) metadata[ATLAS_PARENT_KEY] = parentId;
      return { ...node, x: slot.x, y: slot.y, metadata };
    }),
  };
}

export function atlasUsesEdgeId(sourceId: string, targetId: string): string {
  return `uses:${sourceId}->${targetId}`;
}

/**
 * Adds one edge per `uses:` entry (source depends on target). A manual layout
 * edge between the same two nodes stays and hides the model duplicate.
 */
export function withAtlasUses(
  world: ArchitectureDocument,
  atlas: readonly AtlasCanvasNode[],
): ArchitectureDocument {
  const idByPath = new Map(atlas.map((node) => [node.path, node.id]));
  const drawn = new Set(world.nodes.map((node) => node.id));
  const manual = new Set(world.relationships.map((edge) => `${edge.sourceNodeId}->${edge.targetNodeId}`));
  const edges: Relationship[] = [];
  for (const node of atlas) {
    if (!drawn.has(node.id)) continue;
    for (const path of node.uses ?? []) {
      const targetId = idByPath.get(path);
      if (!targetId || targetId === node.id || !drawn.has(targetId)) continue;
      if (manual.has(`${node.id}->${targetId}`)) continue;
      edges.push({
        id: atlasUsesEdgeId(node.id, targetId),
        sourceNodeId: node.id,
        targetNodeId: targetId,
        type: "depends_on",
        direction: "forward",
        origin: "imported",
        metadata: { [ATLAS_USES_KEY]: true },
      });
    }
  }
  if (edges.length === 0) return world;
  return { ...world, relationships: [...world.relationships, ...edges] };
}
