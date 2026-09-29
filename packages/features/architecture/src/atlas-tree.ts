import type { AtlasCanvasCard, AtlasCanvasNode } from "./atlas-canvas";

/** One row of the Hierarchy tree: an Atlas node (with its child nodes and Work cards) or a Work card leaf. */
export type TreeItem =
  | { kind: "node"; key: string; hasChildren: boolean; node: AtlasCanvasNode; children: TreeItem[] }
  | { kind: "card"; key: string; hasChildren: false; card: AtlasCanvasCard; children: [] };

/** Detail-panel entity id of a row; node ids and card slugs live in one namespace. */
export function entityIdOf(item: TreeItem): string {
  return item.kind === "node" ? `node:${item.node.id}` : item.key;
}

/** Child nodes first, then the Work cards of a component. An orphan node is shown at the root instead of vanishing. */
export function buildTree(nodes: readonly AtlasCanvasNode[], cards: readonly AtlasCanvasCard[]): TreeItem[] {
  const ids = new Set(nodes.map((node) => node.id));
  const byParent = new Map<string, AtlasCanvasNode[]>();
  for (const node of nodes) {
    const parent = node.parentId && ids.has(node.parentId) ? node.parentId : "";
    byParent.set(parent, [...(byParent.get(parent) ?? []), node]);
  }
  const build = (parentId: string): TreeItem[] =>
    (byParent.get(parentId) ?? []).map((node): TreeItem => {
      const children: TreeItem[] = build(node.id);
      if (node.level === "component") {
        for (const card of cards.filter((candidate) => candidate.under === node.path)) {
          children.push({ kind: "card", key: `card:${card.slug}`, hasChildren: false, card, children: [] });
        }
      }
      return { kind: "node", key: node.id, hasChildren: children.length > 0, node, children };
    });
  return build("");
}

/** Rows a person can reach, in display order: the children of a collapsed node are left out. */
export function visibleItems(items: readonly TreeItem[], collapsed: ReadonlySet<string>): TreeItem[] {
  return items.flatMap((item) => [
    item,
    ...(collapsed.has(item.key) ? [] : visibleItems(item.children, collapsed)),
  ]);
}

/** Ids of every node that has something to expand. */
export function expandableKeys(items: readonly TreeItem[]): string[] {
  return items.flatMap((item) => [
    ...(item.hasChildren ? [item.key] : []),
    ...expandableKeys(item.children),
  ]);
}

/** The chain of items from a root to the item with `key` (inclusive), or [] when it is not in the tree. */
export function pathTo(items: readonly TreeItem[], key: string): TreeItem[] {
  for (const item of items) {
    if (item.key === key) return [item];
    const below = pathTo(item.children, key);
    if (below.length > 0) return [item, ...below];
  }
  return [];
}

/** A node with children is a folder: clicking it enters it. */
export function isFolder(item: TreeItem): boolean {
  return item.kind === "node" && item.hasChildren;
}

const OPEN = { duration: 220, easing: "cubic-bezier(0.16, 1, 0.3, 1)" } as const; // --ease-enter
const CLOSE = { duration: 140, easing: "cubic-bezier(0.2, 0, 0, 1)" } as const; // --ease-standard

/**
 * Animates a group of child rows between closed and open. Only the height runs here; the fade, slide, and
 * stagger of the rows are CSS transitions in atlas-tree.css, keyed on `data-state`.
 *
 * Interruptible: a toggle during a run starts from the height on screen, not from zero. The previous animation
 * is returned by each call and handed back on the next one.
 */
export function animateGroupHeight(
  group: HTMLElement,
  open: boolean,
  options: { previous?: Animation; instant: boolean },
): Animation | undefined {
  const wasHidden = group.hidden;
  const from = options.previous ? group.getBoundingClientRect().height : wasHidden ? 0 : group.scrollHeight;
  options.previous?.cancel();

  group.hidden = false;
  if (wasHidden && open) {
    // Leaving display:none has no "before" style, so start the row transitions from the closed state.
    group.dataset.state = "closed";
    void group.offsetHeight;
  }
  group.dataset.state = open ? "open" : "closed";
  group.inert = !open;

  const to = open ? group.scrollHeight : 0;
  const finish = () => {
    group.style.overflow = "";
    group.hidden = !open;
  };
  if (options.instant || from === to) {
    finish();
    return undefined;
  }

  group.style.overflow = "hidden";
  const animation = group.animate([{ height: `${from}px` }, { height: `${to}px` }], open ? OPEN : CLOSE);
  animation.onfinish = finish;
  return animation;
}
