export type MindNode = {
  id: string;
  parentId: string;
  name: string;
  color: string;
  icon: string;
  sortOrder: number;
};

export type MindTreeNode = MindNode & { children: MindTreeNode[] };

export function buildMindTree(minds: readonly MindNode[]): MindTreeNode[] {
  const nodes = new Map<string, MindTreeNode>();
  for (const mind of minds) nodes.set(mind.id, { ...mind, children: [] });
  const roots: MindTreeNode[] = [];
  for (const node of nodes.values()) {
    const parent = nodes.get(node.parentId);
    if (parent && parent.id !== node.id && !isDescendant(nodes, parent.id, node.id)) parent.children.push(node);
    else roots.push(node);
  }
  const sort = (items: MindTreeNode[]) => {
    items.sort((a, b) => a.sortOrder - b.sortOrder || a.name.localeCompare(b.name));
    items.forEach((item) => sort(item.children));
  };
  sort(roots);
  return roots;
}

function isDescendant(nodes: Map<string, MindTreeNode>, startId: string, targetId: string): boolean {
  const pending = [startId];
  const seen = new Set<string>();
  while (pending.length) {
    const id = pending.pop()!;
    if (id === targetId) return true;
    if (seen.has(id)) continue;
    seen.add(id);
    const node = nodes.get(id);
    if (node) pending.push(...node.children.map((child) => child.id));
  }
  return false;
}

export function canMoveMind(minds: readonly MindNode[], mindId: string, parentId: string): boolean {
  if (mindId === parentId) return false;
  const byId = new Map(minds.map((mind) => [mind.id, mind]));
  let parent = byId.get(parentId);
  const seen = new Set<string>();
  while (parent) {
    if (parent.id === mindId || seen.has(parent.id)) return false;
    seen.add(parent.id);
    parent = byId.get(parent.parentId);
  }
  return true;
}

export function readCollapsedMindIds(storage: Pick<Storage, "getItem">): string[] {
  try {
    const value: unknown = JSON.parse(storage.getItem("sparky.minds.collapsed") ?? "[]");
    return Array.isArray(value) && value.every((id) => typeof id === "string") ? value : [];
  } catch {
    return [];
  }
}
