export type MindNode = {
  id: string;
  parentId: string;
  name: string;
  color: string;
  icon: string;
  sortOrder: number;
};

export type MindTreeNode = MindNode & { children: MindTreeNode[] };

export type DeletePreview = {
  deletedMindIds: string[];
  movedMemoryCount: number;
};

export const MINDS_EXPANDED_KEY = "sparky.minds.expanded";

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
  if (parentId === "") return true;
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

export function cascadeDeleteIds(minds: readonly MindNode[], mindId: string): string[] {
  const seen = new Set<string>([mindId]);
  let changed = true;
  while (changed) {
    changed = false;
    for (const mind of minds) {
      if (mind.parentId && seen.has(mind.parentId) && !seen.has(mind.id)) {
        seen.add(mind.id);
        changed = true;
      }
    }
  }
  return [...seen].sort();
}

export function deletePreviewCopy(preview: DeletePreview): string {
  const childCount = Math.max(0, preview.deletedMindIds.length - 1);
  return `Deletes ${childCount} child Minds; ${preview.movedMemoryCount} memories move to Inbox.`;
}

export function mindAncestors(minds: readonly MindNode[], mindId: string): MindNode[] {
  const byId = new Map(minds.map((mind) => [mind.id, mind]));
  const chain: MindNode[] = [];
  const seen = new Set<string>();
  let current = byId.get(mindId);
  current = current ? byId.get(current.parentId) : undefined;
  while (current && !seen.has(current.id)) {
    seen.add(current.id);
    chain.unshift(current);
    current = byId.get(current.parentId);
  }
  return chain;
}

export function siblingOrder(minds: readonly MindNode[], parentId: string, mindId: string, direction: -1 | 1): string[] | undefined {
  const siblings = minds
    .filter((mind) => mind.parentId === parentId)
    .sort((a, b) => a.sortOrder - b.sortOrder || a.name.localeCompare(b.name));
  const from = siblings.findIndex((mind) => mind.id === mindId);
  const to = from + direction;
  if (from < 0 || to < 0 || to >= siblings.length) return undefined;
  const ordered = siblings.map((mind) => mind.id);
  const moving = ordered[from]!;
  ordered[from] = ordered[to]!;
  ordered[to] = moving;
  return ordered;
}

export function previousSiblingId(minds: readonly MindNode[], mindId: string): string | undefined {
  const current = minds.find((mind) => mind.id === mindId);
  if (!current) return undefined;
  const siblings = minds
    .filter((mind) => mind.parentId === current.parentId)
    .sort((a, b) => a.sortOrder - b.sortOrder || a.name.localeCompare(b.name));
  const index = siblings.findIndex((mind) => mind.id === mindId);
  return index > 0 ? siblings[index - 1]?.id : undefined;
}

export function readExpandedMindIds(storage: Pick<Storage, "getItem">): string[] {
  try {
    const value: unknown = JSON.parse(storage.getItem(MINDS_EXPANDED_KEY) ?? "[]");
    return Array.isArray(value) && value.every((id) => typeof id === "string") ? value : [];
  } catch {
    return [];
  }
}

export function writeExpandedMindIds(storage: Pick<Storage, "setItem">, ids: readonly string[]): void {
  try {
    storage.setItem(MINDS_EXPANDED_KEY, JSON.stringify([...ids]));
  } catch {
    /* Session-only when storage is unavailable. */
  }
}

/** @deprecated Use readExpandedMindIds. Kept for callers that stored collapse ids. */
export function readCollapsedMindIds(storage: Pick<Storage, "getItem">): string[] {
  return readExpandedMindIds(storage);
}

export function moveTargetOptions(minds: readonly MindNode[], mindId: string): { value: string; label: string; disabled: boolean }[] {
  return [
    { value: "", label: "Top level", disabled: false },
    ...minds.map((mind) => ({
      value: mind.id,
      label: mind.name,
      disabled: !canMoveMind(minds, mindId, mind.id),
    })),
  ];
}

export function nextSortOrder(minds: readonly MindNode[], parentId: string): number {
  return minds.filter((mind) => mind.parentId === parentId).reduce((max, mind) => Math.max(max, mind.sortOrder), -1) + 1;
}

export function isHexColor(value: string): boolean {
  return /^#[0-9A-Fa-f]{6}$/.test(value);
}
