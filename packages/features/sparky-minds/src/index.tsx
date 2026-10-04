import { createEffect, createResource, createSignal, For, Show } from "solid-js";
import { Button, Card, ColorSwatchPicker, DitherIconTile, Dialog, Field, IconPicker, Input, PageEmpty, PageError, PageHeader, PageLoading, parseColor, TreeView, type PickerIconName, type TreeViewNode } from "@polterware/frontend-design-system";
import type { Mind } from "@polterware/sparky-contracts";
import type { SparkyFeature } from "@polterware/feature-sparky-shell/feature";
import { buildMindTree, readCollapsedMindIds, type MindNode } from "./tree";

const SWATCHES = ["#7EA8F0", "#9B8AFB", "#6CC7A5", "#F0A66B", "#E47F9A", "#75BFC8"].map((value) => ({ value, label: value }));
const STORAGE_KEY = "sparky.minds.expanded";
const selectedId = () => decodeURIComponent(location.pathname.split("/")[2] ?? "");
const toNode = (mind: Mind): MindNode => ({ id: mind.id, parentId: mind.parentId, name: mind.name, color: mind.color, icon: mind.icon, sortOrder: mind.sortOrder });

async function loadMinds() {
  const response = await window.sparky.minds.list();
  return response.minds;
}

function MindPage() {
  const [minds, { refetch }] = createResource(loadMinds);
  const [name, setName] = createSignal("");
  const [color, setColor] = createSignal("#7EA8F0");
  const [icon, setIcon] = createSignal("brain");
  const [message, setMessage] = createSignal("");
  const [dialogOpen, setDialogOpen] = createSignal(false);
  const [deletePreview, setDeletePreview] = createSignal<{ ids: string[]; memories: number }>();
  const tree = () => buildMindTree((minds() ?? []).map(toNode));
  const selected = () => (minds() ?? []).find((mind: Mind) => mind.id === selectedId());
  const children = () => tree().filter((mind) => mind.id === selectedId())[0]?.children ?? [];
  const treeNodes = (): TreeViewNode[] => tree().map((mind) => ({ id: mind.id, label: mind.name, children: mind.children.map((child) => ({ id: child.id, label: child.name })) }));

  createEffect(() => {
    const mind = selected();
    setName(mind?.name ?? "");
    setColor(mind?.color || "#7EA8F0");
    setIcon((mind?.icon || "brain") as PickerIconName);
  });

  const createMind = async (parentId = "") => {
    if (!name().trim()) return;
    try {
      await window.sparky.minds.create({ parentId, name: name().trim(), color: color(), icon: icon(), sortOrder: 0 });
      setName(""); setDialogOpen(false); setMessage(""); await refetch();
    } catch (error) { setMessage(error instanceof Error ? error.message : "Mind could not be created."); }
  };

  const saveMind = async () => {
    const current = selected();
    if (!current || !name().trim()) return;
    try {
      await window.sparky.minds.update({ id: current.id, expectedRev: current.rev.toString(), name: name().trim(), color: color(), icon: icon() });
      setMessage(""); await refetch();
    } catch (error) { setMessage(error instanceof Error ? error.message : "Mind could not be updated."); }
  };

  const requestDelete = async () => {
    const current = selected();
    if (!current) return;
    const ids = descendants(current.id, tree());
    try {
      const counts = await Promise.all(ids.map((mindId) => window.sparky.memories.list({ mindId, includeDescendants: false, limit: 1 })));
      setDeletePreview({ ids, memories: counts.reduce((sum: number, count) => sum + count.total, 0) });
    } catch { setMessage("Mind cascade details are unavailable. Try again when the core is online."); }
  };

  const confirmDelete = async () => {
    const current = selected();
    if (!current || !deletePreview()) return;
    try {
      const result = await window.sparky.minds.delete({ id: current.id, expectedRev: current.rev.toString() });
      setDeletePreview(undefined); setMessage(`${result.deletedMindIds.length} Minds deleted; ${result.movedMemoryCount} memories moved to Inbox.`);
      await refetch();
    } catch (error) { setMessage(error instanceof Error ? error.message : "Mind could not be deleted."); }
  };

  const reorder = async (mind: MindNode, direction: -1 | 1) => {
    const siblings = (minds() ?? []).filter((item: Mind) => item.parentId === mind.parentId).sort((a: Mind, b: Mind) => a.sortOrder - b.sortOrder);
    const from = siblings.findIndex((item) => item.id === mind.id);
    const to = from + direction;
    if (from < 0 || to < 0 || to >= siblings.length) return;
    [siblings[from], siblings[to]] = [siblings[to]!, siblings[from]!];
    await window.sparky.minds.reorder({ parentId: mind.parentId, orderedIds: siblings.map((item: Mind) => item.id) });
    await refetch();
  };

  const onTreeExpanded = (ids: string[]) => {
    try { localStorage.setItem(STORAGE_KEY, JSON.stringify(ids)); } catch { /* Storage may be unavailable. */ }
  };

  return <Show when={!minds.loading} fallback={<PageLoading label="Loading Minds" />}>
    <Show when={!minds.error} fallback={<PageError message="Minds are unavailable while the local core is offline." onRetry={() => void refetch()} />}>
      <div class="flex h-full min-h-0">
        <aside class="w-64 shrink-0 overflow-auto border-r border-[var(--color-border-subtle)] p-3" aria-label="Mind navigation">
          <div class="mb-3 flex items-center justify-between"><h2 class="text-sm font-semibold">Minds</h2><Button size="sm" onClick={() => setDialogOpen(true)}>New</Button></div>
          <Show when={treeNodes().length} fallback={<p class="text-sm text-[var(--color-text-muted)]">No Minds yet.</p>}>
            <TreeView label="Mind tree" nodes={treeNodes()} defaultExpandedValue={readCollapsedMindIds(localStorage)} onExpandedChange={(event) => onTreeExpanded(event.expandedValue)} />
          </Show>
          <For each={minds()}>{(mind) => <a class="block rounded-[var(--radius-control)] px-2 py-1 text-sm text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)]" href={`/mind/${mind.id}`} aria-current={selectedId() === mind.id ? "page" : undefined}>{mind.name}</a>}</For>
        </aside>
        <main class="min-w-0 flex-1 overflow-auto">
          <Show when={selected()} fallback={<>
            <PageHeader title="Minds" actions={<Button variant="primary" onClick={() => setDialogOpen(true)}>Create Mind</Button>} />
            <Show when={tree().length} fallback={<PageEmpty title="Create your first Mind" description="Minds organize memories into a hierarchy." action={<Button variant="primary" onClick={() => setDialogOpen(true)}>Create Mind</Button>} />}>
              <div class="grid grid-cols-[repeat(auto-fill,minmax(200px,1fr))] gap-3 p-4"><For each={tree()}>{(mind) => <a href={`/mind/${mind.id}`} class="no-underline"><Card><DitherIconTile tone="neutral" size={28}>{mind.icon.slice(0, 1).toUpperCase()}</DitherIconTile><p class="mt-2 text-sm font-medium text-[var(--color-text-primary)]">{mind.name}</p><p class="text-xs text-[var(--color-text-muted)]">{mind.children.length} child Minds</p></Card></a>}</For></div>
            </Show>
          </>}>
            {(mind) => <>
              <PageHeader title={mind().name} breadcrumb={<a href="/minds">Minds</a>} actions={<Button onClick={() => void createMind(mind().id)}>Add child Mind</Button>} />
              <div class="space-y-4 p-4">
                <Card><div class="flex flex-wrap items-end gap-3"><Field label="Name"><Input value={name()} onInput={(event) => setName(event.currentTarget.value)} /></Field><IconPicker label="Mind icon" value={icon() as PickerIconName} onValueChange={(value) => setIcon(value)} /><ColorSwatchPicker label="Mind color" value={parseColor(color())} onValueChange={(value) => setColor(value.toString())} swatches={SWATCHES} /><Button variant="primary" onClick={() => void saveMind()}>Save changes</Button><Button variant="danger" onClick={() => void requestDelete()}>Delete Mind</Button></div></Card>
                <Show when={message()}><p role="status" class="text-sm text-[var(--color-text-secondary)]">{message()}</p></Show>
                <section aria-labelledby="children-title"><h2 id="children-title" class="mb-3 text-sm font-semibold">Child Minds</h2><Show when={children().length} fallback={<PageEmpty title="No child Minds" description="Add a child Mind to build this hierarchy." action={<Button onClick={() => void createMind(mind().id)}>Add child Mind</Button>} />}><div class="grid grid-cols-[repeat(auto-fill,minmax(200px,1fr))] gap-3"><For each={children()}>{(child) => <Card><a href={`/mind/${child.id}`} class="text-sm text-[var(--color-text-primary)]">{child.name}</a><div class="mt-2 flex gap-2"><Button size="sm" aria-label={`Move ${child.name} up`} onClick={() => void reorder(child, -1)}>Move up</Button><Button size="sm" aria-label={`Move ${child.name} down`} onClick={() => void reorder(child, 1)}>Move down</Button></div></Card>}</For></div></Show></section>
              </div>
            </>}
          </Show>
        </main>
      </div>
      <Dialog open={dialogOpen()} title="Create Mind" description="Create a Mind to organize memories." onClose={() => setDialogOpen(false)}><div class="space-y-3 p-4"><Field label="Name"><Input value={name()} onInput={(event) => setName(event.currentTarget.value)} /></Field><IconPicker label="Mind icon" value={icon() as PickerIconName} onValueChange={(value) => setIcon(value)} /><ColorSwatchPicker label="Mind color" value={parseColor(color())} onValueChange={(value) => setColor(value.toString())} swatches={SWATCHES} /><Button variant="primary" onClick={() => void createMind()}>Create</Button></div></Dialog>
      <Dialog open={!!deletePreview()} title="Delete Mind" description="Review the cascade before deleting." onClose={() => setDeletePreview(undefined)}><div class="space-y-3 p-4"><p>Deletes {Math.max(0, (deletePreview()?.ids.length ?? 1) - 1)} child Minds; {deletePreview()?.memories ?? 0} memories move to Inbox.</p><Button variant="danger" onClick={() => void confirmDelete()}>Delete Mind</Button></div></Dialog>
    </Show>
  </Show>;
}

function descendants(id: string, tree: ReturnType<typeof buildMindTree>): string[] {
  const result: string[] = [];
  const visit = (nodes: typeof tree) => nodes.forEach((node) => { result.push(node.id); visit(node.children); });
  visit(tree.filter((node) => node.id === id));
  return result;
}

const feature: SparkyFeature = {
  id: "minds",
  routes: [
    { path: "/minds", component: MindPage, title: "Minds" },
    { path: "/mind/:id", component: MindPage, title: "Mind" },
  ],
  sidebar: [{ id: "minds", label: "Minds", path: "/minds", icon: "brain" }],
};

export default feature;
