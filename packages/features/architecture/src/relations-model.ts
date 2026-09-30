import type { AtlasCanvasNode } from "./atlas-canvas";

export type NodeRelations = {
  node: AtlasCanvasNode;
  /** Nodes this node depends on (`uses:`). */
  uses: AtlasCanvasNode[];
  /** Nodes that depend on this node. */
  usedBy: AtlasCanvasNode[];
};

/**
 * Every domain or component that takes part in at least one `uses:` relation.
 * Systems are skipped; unknown paths and self references are ignored.
 */
export function buildRelations(nodes: readonly AtlasCanvasNode[]): NodeRelations[] {
  const byPath = new Map(nodes.map((node) => [node.path, node]));
  const scoped = nodes.filter((node) => node.level !== "system");
  const usedBy = new Map<string, AtlasCanvasNode[]>();
  const uses = new Map<string, AtlasCanvasNode[]>();

  for (const source of scoped) {
    for (const path of source.uses ?? []) {
      const target = byPath.get(path);
      if (!target || target.id === source.id || target.level === "system") continue;
      const outgoing = uses.get(source.id) ?? [];
      if (!outgoing.some((node) => node.id === target.id)) outgoing.push(target);
      uses.set(source.id, outgoing);
      const incoming = usedBy.get(target.id) ?? [];
      if (!incoming.some((node) => node.id === source.id)) incoming.push(source);
      usedBy.set(target.id, incoming);
    }
  }

  return scoped
    .filter((node) => uses.has(node.id) || usedBy.has(node.id))
    .map((node) => ({ node, uses: uses.get(node.id) ?? [], usedBy: usedBy.get(node.id) ?? [] }));
}

export type RelationDirection = "uses" | "usedBy";

export type RelationFilters = {
  query: string;
  /** Domain ids; a section stays when its node is one of them or sits inside one. */
  domainIds: ReadonlySet<string>;
  direction: RelationDirection | null;
};

/** Narrows sections by search text, domain and direction; a direction also trims the other side. */
export function filterRelations(rows: readonly NodeRelations[], filters: RelationFilters): NodeRelations[] {
  const query = filters.query.trim().toLowerCase();
  const result: NodeRelations[] = [];
  for (const row of rows) {
    if (
      filters.domainIds.size > 0
      && !filters.domainIds.has(row.node.id)
      && !filters.domainIds.has(row.node.parentId)
    ) {
      continue;
    }
    if (filters.direction === "uses" && row.uses.length === 0) continue;
    if (filters.direction === "usedBy" && row.usedBy.length === 0) continue;
    if (query) {
      const titles = [row.node, ...row.uses, ...row.usedBy].map((node) => node.title.toLowerCase());
      if (!titles.some((title) => title.includes(query))) continue;
    }
    result.push({
      node: row.node,
      uses: filters.direction === "usedBy" ? [] : row.uses,
      usedBy: filters.direction === "uses" ? [] : row.usedBy,
    });
  }
  return result;
}
