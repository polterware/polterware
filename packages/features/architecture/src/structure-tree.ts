export type StructureKind = "domain" | "component";
export type StructureNode = {
  id: string;
  parentId: string;
  level: StructureKind;
  path: string;
  title: string;
  intent: string;
  repo: string;
  /** Folder inside the checkout. */
  folder?: string;
  /** What a component is (a canvas component kind). */
  kind?: string;
  /** Free-text technology. */
  tech?: string;
  decisions: string[];
  deploys?: StructureDeploy[];
};
/**
 * Where a component runs in one environment: the platform's resource, its own id and
 * dashboard page once linked, and the latest deploy (state ok, building, failed, stopped, or unknown).
 */
export type StructureDeploy = {
  id: string; environmentId: string; environment: string; provider: string; resource: string;
  targetId?: string; url?: string; state?: string; stateSummary?: string; stateUrl?: string; checkedAt?: string;
};
/** A deploy as the document writes it; `targetId` is the platform's id for the resource. */
export type StructureDeployInput = { environmentId: string; provider: string; resource: string; targetId?: string };
/** A project, app, site, or service a deploy can link to; `detected` when the component's checkout declares it. */
export type StructureDeployTarget = { name: string; id: string; kind: string; url: string; detected: boolean };
export type StructureTreeItem = { node: StructureNode; children: StructureTreeItem[] };

/**
 * Preserve input order and orphan records. A malformed cycle is shown once, never recursed forever.
 * A query or `match` keeps the nodes that pass both, with the ancestors that lead to them.
 */
export function buildStructureTree(nodes: readonly StructureNode[], query = "", match?: (node: StructureNode) => boolean): StructureTreeItem[] {
  const ids = new Set(nodes.map((node) => node.id));
  const children = new Map<string, StructureNode[]>();
  for (const node of nodes) {
    const parent = node.parentId && ids.has(node.parentId) ? node.parentId : "";
    children.set(parent, [...(children.get(parent) ?? []), node]);
  }
  const seen = new Set<string>();
  const build = (node: StructureNode): StructureTreeItem[] => {
    if (seen.has(node.id)) return [];
    seen.add(node.id);
    return [{ node, children: (children.get(node.id) ?? []).flatMap(build) }];
  };
  const roots = (children.get("") ?? []).flatMap(build);
  for (const node of nodes) roots.push(...build(node));
  const term = query.trim().toLowerCase();
  if (!term && !match) return roots;
  const passes = (node: StructureNode) =>
    (!term || `${node.title} ${node.level} ${node.path} ${node.kind ?? ""} ${node.tech ?? ""}`.toLowerCase().includes(term)) && (!match || match(node));
  const filter = (items: StructureTreeItem[]): StructureTreeItem[] => items.flatMap((item) => {
    const descendants = filter(item.children);
    // A text match keeps its whole branch; a `match` filter keeps only the nodes that pass it.
    return passes(item.node)
      ? [{ ...item, children: match ? descendants : item.children }]
      : descendants.length ? [{ ...item, children: descendants }] : [];
  });
  return filter(roots);
}

export function flattenStructureTree(items: readonly StructureTreeItem[]): StructureNode[] {
  return items.flatMap((item) => [item.node, ...flattenStructureTree(item.children)]);
}

/** A component's parent is a domain (or none: it sits loose under the project); a domain has none. */
export function structureParents(nodes: readonly StructureNode[], kind: StructureKind, movingId?: string): StructureNode[] {
  return nodes.filter((node) => node.id !== movingId && kind === "component" && node.level === "domain");
}

/** Scope nodes form a union of subtrees; their ancestors retain the project hierarchy. */
export function filterStructureNodes<T extends Pick<StructureNode, "id" | "parentId">>(nodes: readonly T[], scopeIds: readonly string[]): T[] {
  if (scopeIds.length === 0) return [...nodes];
  const selected = new Set(scopeIds);
  const byId = new Map(nodes.map((node) => [node.id, node]));
  const included = new Set<string>();
  for (const node of nodes) {
    const chain = new Set<string>();
    let current: T | undefined = node;
    while (current && !chain.has(current.id)) {
      chain.add(current.id);
      if (selected.has(current.id)) { included.add(node.id); break; }
      current = byId.get(current.parentId);
    }
  }
  for (const id of scopeIds) {
    const seen = new Set<string>();
    let current = byId.get(id);
    while (current && !seen.has(current.id)) {
      seen.add(current.id); included.add(current.id); current = byId.get(current.parentId);
    }
  }
  return nodes.filter((node) => included.has(node.id));
}
