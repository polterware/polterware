import { Show, createEffect, createMemo, createResource, createSignal, onCleanup, onMount } from "solid-js";
import {
  DitherIconTile,
  PICKER_ICONS,
  TreeView,
  type PickerIconName,
  type TreeViewNode,
} from "@polterware/frontend-design-system";
import { Dynamic } from "solid-js/web";
import { useSparkyShell } from "@polterware/feature-sparky-shell";
import type { Mind } from "@polterware/sparky-contracts";
import { loadMemoryCounts, loadMinds, mindsBridge, parentPatch, toMindNode, wireRev } from "./bridge";
import { bumpMinds, mindsRevision } from "./session";
import {
  buildMindTree,
  canMoveMind,
  previousSiblingId,
  readExpandedMindIds,
  siblingOrder,
  writeExpandedMindIds,
  type MindTreeNode,
} from "./tree";

export function pickerIcon(name: string): PickerIconName {
  return name in PICKER_ICONS ? (name as PickerIconName) : "brain";
}

export function MindMark(props: { color: string; icon: string; size?: 16 | 20 | 24 | 28 }) {
  return (
    <DitherIconTile size={props.size ?? 20} tone="knowledge" accent={props.color || undefined}>
      <Dynamic component={PICKER_ICONS[pickerIcon(props.icon)]} size={Math.max(10, (props.size ?? 20) - 8)} />
    </DitherIconTile>
  );
}

function labeledNodes(nodes: MindTreeNode[], counts: Record<string, number>): TreeViewNode[] {
  return nodes.map((node) => ({
    id: node.id,
    label: `${node.name} · ${counts[node.id] ?? 0}`,
    children: node.children.length ? labeledNodes(node.children, counts) : undefined,
  }));
}

function rowId(target: EventTarget | null): string | undefined {
  const element = target instanceof Element ? target.closest("[data-value], [data-mind-id]") : null;
  return element?.getAttribute("data-value") ?? element?.getAttribute("data-mind-id") ?? undefined;
}

export function MindsSidebarTree() {
  const shell = useSparkyShell();
  const [minds] = createResource(mindsRevision, async () => loadMinds(mindsBridge()));
  const nodes = createMemo(() => (minds() ?? []).map(toMindNode));
  const tree = createMemo(() => buildMindTree(nodes()));
  const [counts] = createResource(
    () => {
      const list = minds();
      return list ? `${mindsRevision()}:${list.map((mind) => mind.id).join(",")}` : false;
    },
    async () => loadMemoryCounts(mindsBridge(), (minds() ?? []).map((mind) => mind.id)),
  );
  const [expanded, setExpanded] = createSignal<string[]>(readExpandedMindIds(localStorage));
  const [dragging, setDragging] = createSignal<string>();
  const selectedId = () => shell.params().id ?? "";
  let root: HTMLDivElement | undefined;

  onMount(() => {
    try {
      const stop = mindsBridge().onChange(() => bumpMinds());
      onCleanup(stop);
    } catch {
      /* Core offline: the page surfaces that state. */
    }
  });

  createEffect(() => {
    writeExpandedMindIds(localStorage, expanded());
  });

  createEffect(() => {
    tree();
    counts();
    root?.querySelectorAll<HTMLElement>("[data-part='item'], [data-part='branch-control']").forEach((row) => {
      row.setAttribute("draggable", "true");
    });
  });

  const moveTo = async (mindId: string, parentId: string) => {
    const list = minds() ?? [];
    if (!canMoveMind(nodes(), mindId, parentId)) return;
    const current = list.find((mind) => mind.id === mindId);
    if (!current || current.parentId === parentId) return;
    await mindsBridge().minds.update({
      id: mindId,
      expectedRev: wireRev(current.rev),
      ...parentPatch(parentId),
    });
    bumpMinds();
  };

  const reorder = async (mind: Mind, direction: -1 | 1) => {
    const ordered = siblingOrder(nodes(), mind.parentId, mind.id, direction);
    if (!ordered) return;
    await mindsBridge().minds.reorder({ parentId: mind.parentId, orderedIds: ordered });
    bumpMinds();
  };

  const onTreeKey = async (event: KeyboardEvent) => {
    if (!event.altKey) return;
    const id = selectedId() || dragging();
    const current = (minds() ?? []).find((mind) => mind.id === id);
    if (!current) return;
    if (event.key === "ArrowUp") {
      event.preventDefault();
      await reorder(current, -1);
    } else if (event.key === "ArrowDown") {
      event.preventDefault();
      await reorder(current, 1);
    } else if (event.key === "ArrowLeft") {
      event.preventDefault();
      const parent = nodes().find((mind) => mind.id === current.parentId);
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
        if (!from) return;
        if (to && !canMoveMind(nodes(), from, to)) return;
        event.preventDefault();
        if (event.dataTransfer) event.dataTransfer.dropEffect = "move";
      }}
      onDrop={(event) => {
        event.preventDefault();
        const from = event.dataTransfer?.getData("text/plain") || dragging();
        const to = rowId(event.target) ?? "";
        setDragging(undefined);
        if (from) void moveTo(from, to);
      }}
    >
      <Show when={!minds.loading} fallback={<p class="text-xs text-[var(--color-text-muted)]">Loading Minds…</p>}>
        <Show when={!minds.error} fallback={<p role="status">Minds unavailable</p>}>
          <Show when={tree().length} fallback={<p class="text-xs text-[var(--color-text-muted)]">No Minds yet.</p>}>
            <TreeView
              label="Minds"
              nodes={labeledNodes(tree(), counts() ?? {})}
              expandedValue={expanded()}
              selectedValue={selectedId() ? [selectedId()] : []}
              onExpandedChange={(event) => setExpanded(event.expandedValue)}
              onSelectionChange={(event) => {
                const id = event.selectedValue[0];
                if (id) shell.navigate(`/mind/${encodeURIComponent(id)}`);
              }}
            />
            <p class="text-xs text-[var(--color-text-muted)]">Drag to nest. Alt+Arrow keys move the selected Mind. Cycles are not offered as drop targets.</p>
          </Show>
        </Show>
      </Show>
    </div>
  );
}
