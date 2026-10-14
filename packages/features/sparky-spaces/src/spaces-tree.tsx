import { Show, createEffect, createMemo, createResource, createSignal, onCleanup, onMount } from "solid-js";
import {
  IconTile,
  PICKER_ICONS,
  TreeView,
  type AuroraPalette,
  type PickerIconName,
  type TreeViewNode,
} from "@polterware/frontend-design-system";
import { Dynamic } from "solid-js/web";
import { useSparkyShell } from "@polterware/feature-sparky-shell";
import type { Space } from "@polterware/sparky-contracts";
import { draggedNoteId, hasDraggedNote, loadNoteCounts, loadSpaces, spacesBridge, moveNoteToSpace, parentPatch, toSpaceNode, wireRev } from "./bridge";
import { bumpSpaces, spacesRevision } from "./session";
import {
  buildSpaceTree,
  canMoveSpace,
  previousSiblingId,
  readExpandedSpaceIds,
  siblingOrder,
  writeExpandedSpaceIds,
  type SpaceTreeNode,
} from "./tree";

export function pickerIcon(name: string): PickerIconName {
  return name in PICKER_ICONS ? (name as PickerIconName) : "brain";
}

export function SpaceMark(props: { color: string; icon: string; size?: 16 | 20 | 24 | 28 | 36; aurora?: AuroraPalette }) {
  return (
    <IconTile size={props.size ?? 20} tone="knowledge" aurora={props.aurora} accent={props.color || undefined}>
      <Dynamic component={PICKER_ICONS[pickerIcon(props.icon)]} size={Math.max(10, (props.size ?? 20) - ((props.size ?? 20) >= 36 ? 12 : 8))} />
    </IconTile>
  );
}

function labeledNodes(nodes: SpaceTreeNode[], counts: Record<string, number>): TreeViewNode[] {
  return nodes.map((node) => ({
    id: node.id,
    label: node.name,
    icon: <SpaceMark color={node.color} icon={node.icon} size={16} />,
    trailing: String(counts[node.id] ?? 0),
    children: node.children.length ? labeledNodes(node.children, counts) : undefined,
  }));
}

function rowId(target: EventTarget | null): string | undefined {
  const element = target instanceof Element ? target.closest("[data-value], [data-space-id]") : null;
  return element?.getAttribute("data-value") ?? element?.getAttribute("data-space-id") ?? undefined;
}

export function SpacesSidebarTree() {
  const shell = useSparkyShell();
  const [spaces] = createResource(spacesRevision, async () => loadSpaces(spacesBridge()));
  const nodes = createMemo(() => (spaces() ?? []).map(toSpaceNode));
  const tree = createMemo(() => buildSpaceTree(nodes()));
  const [counts] = createResource(
    () => {
      const list = spaces();
      return list ? `${spacesRevision()}:${list.map((space) => space.id).join(",")}` : false;
    },
    async () => loadNoteCounts(spacesBridge(), (spaces() ?? []).map((space) => space.id)),
  );
  const [expanded, setExpanded] = createSignal<string[]>(readExpandedSpaceIds(localStorage));
  const [dragging, setDragging] = createSignal<string>();
  const selectedId = () => shell.params().id ?? "";
  let root: HTMLDivElement | undefined;

  onMount(() => {
    try {
      const stop = spacesBridge().onChange(() => bumpSpaces());
      onCleanup(stop);
    } catch {
      /* Core offline: the page surfaces that state. */
    }
  });

  createEffect(() => {
    writeExpandedSpaceIds(localStorage, expanded());
  });

  createEffect(() => {
    tree();
    counts();
    root?.querySelectorAll<HTMLElement>("[data-part='item'], [data-part='branch-control']").forEach((row) => {
      row.setAttribute("draggable", "true");
    });
  });

  const moveTo = async (spaceId: string, parentId: string) => {
    const list = spaces() ?? [];
    if (!canMoveSpace(nodes(), spaceId, parentId)) return;
    const current = list.find((space) => space.id === spaceId);
    if (!current || current.parentId === parentId) return;
    await spacesBridge().spaces.update({
      id: spaceId,
      expectedRev: wireRev(current.rev),
      ...parentPatch(parentId),
    });
    bumpSpaces();
  };

  const reorder = async (space: Space, direction: -1 | 1) => {
    const ordered = siblingOrder(nodes(), space.parentId, space.id, direction);
    if (!ordered) return;
    await spacesBridge().spaces.reorder({ parentId: space.parentId, orderedIds: ordered });
    bumpSpaces();
  };

  const onTreeKey = async (event: KeyboardEvent) => {
    if (!event.altKey) return;
    const id = selectedId() || dragging();
    const current = (spaces() ?? []).find((space) => space.id === id);
    if (!current) return;
    if (event.key === "ArrowUp") {
      event.preventDefault();
      await reorder(current, -1);
    } else if (event.key === "ArrowDown") {
      event.preventDefault();
      await reorder(current, 1);
    } else if (event.key === "ArrowLeft") {
      event.preventDefault();
      const parent = nodes().find((space) => space.id === current.parentId);
      await moveTo(current.id, parent?.parentId ?? "");
    } else if (event.key === "ArrowRight") {
      event.preventDefault();
      const sibling = previousSiblingId(nodes(), current.id);
      if (sibling) await moveTo(current.id, sibling);
    }
  };

  return (
    <div
      ref={root}
      class="flex min-h-0 flex-col gap-2"
      onKeyDown={(event) => void onTreeKey(event)}
      onDragStart={(event) => {
        const id = rowId(event.target);
        if (!id) return;
        setDragging(id);
        event.dataTransfer?.setData("text/plain", id);
        if (event.dataTransfer) event.dataTransfer.effectAllowed = "move";
      }}
      onDragEnd={() => setDragging(undefined)}
      onDragOver={(event) => {
        const from = dragging();
        const to = rowId(event.target);
        if (!from && hasDraggedNote(event.dataTransfer)) { event.preventDefault(); if (event.dataTransfer) event.dataTransfer.dropEffect = "move"; return; }
        if (!from) return;
        if (to && !canMoveSpace(nodes(), from, to)) return;
        event.preventDefault();
        if (event.dataTransfer) event.dataTransfer.dropEffect = "move";
      }}
      onDrop={(event) => {
        event.preventDefault();
        const noteId = draggedNoteId(event.dataTransfer);
        if (noteId) {
          void moveNoteToSpace(noteId, rowId(event.target) ?? "").then(() => bumpSpaces());
          return;
        }
        const from = event.dataTransfer?.getData("text/plain") || dragging();
        const to = rowId(event.target) ?? "";
        setDragging(undefined);
        if (from) void moveTo(from, to);
      }}
    >
      <Show when={!spaces.loading} fallback={<p class="text-xs text-[var(--color-text-muted)]">Loading Spaces…</p>}>
        <Show when={!spaces.error} fallback={<p role="status">Spaces unavailable</p>}>
          <Show when={tree().length}>
            <TreeView
              label="Spaces"
              nodes={labeledNodes(tree(), counts() ?? {})}
              expandedValue={expanded()}
              selectedValue={selectedId() ? [selectedId()] : []}
              onExpandedChange={(event) => setExpanded(event.expandedValue)}
              onSelectionChange={(event) => {
                const id = event.selectedValue[0];
                if (id) shell.navigate(`/space/${encodeURIComponent(id)}`);
              }}
            />
          </Show>
        </Show>
      </Show>
    </div>
  );
}
