import { For, Show, createEffect, createMemo, createResource, createSignal, onCleanup, onMount } from "solid-js";
import { Dynamic } from "solid-js/web";
import {
  Button,
  Card,
  ColorSwatchPicker,
  Dialog,
  Field,
  IconPicker,
  Input,
  PageEmpty,
  PageError,
  PageHeader,
  PageLoading,
  parseColor,
  Select,
  type PickerIconName,
} from "@polterware/frontend-design-system";
import "./minds.css";
import { useSparkyShell } from "@polterware/feature-sparky-shell";
import type { Mind } from "@polterware/sparky-contracts";
import { hexFromPicker, loadDeletePreview, loadMinds, mindsBridge, parentPatch, toMindNode, wireRev } from "./bridge";
import { MindsSidebarTree, MindMark, pickerIcon } from "./minds-tree";
import { bumpMinds, mindsRevision } from "./session";
import {
  buildMindTree,
  canMoveMind,
  deletePreviewCopy,
  isHexColor,
  mindAncestors,
  moveTargetOptions,
  nextSortOrder,
  siblingOrder,
  type DeletePreview,
} from "./tree";

const SWATCHES = ["#7EA8F0", "#9B8AFB", "#6CC7A5", "#F0A66B", "#E47F9A", "#75BFC8"].map((value) => ({
  value,
  label: value,
}));

export function MindPage() {
  const shell = useSparkyShell();
  const [minds, { refetch }] = createResource(mindsRevision, async () => loadMinds(mindsBridge()));
  const [name, setName] = createSignal("");
  const [color, setColor] = createSignal("#7EA8F0");
  const [icon, setIcon] = createSignal<PickerIconName>("brain");
  const [createName, setCreateName] = createSignal("");
  const [createColor, setCreateColor] = createSignal("#7EA8F0");
  const [createIcon, setCreateIcon] = createSignal<PickerIconName>("brain");
  const [createParent, setCreateParent] = createSignal("");
  const [createOpen, setCreateOpen] = createSignal(false);
  const [message, setMessage] = createSignal("");
  const [deletePreview, setDeletePreview] = createSignal<DeletePreview>();
  const selectedId = () => shell.params().id ?? "";
  const memoryCollection = createMemo(() => shell.features.find(feature => feature.memoryCollection)?.memoryCollection);
  const nodes = createMemo(() => (minds() ?? []).map(toMindNode));
  const tree = createMemo(() => buildMindTree(nodes()));
  const selected = createMemo(() => (minds() ?? []).find((mind) => mind.id === selectedId()));
  const children = createMemo(() => tree().flatMap(function walk(node): typeof node.children {
    if (node.id === selectedId()) return node.children;
    return node.children.flatMap(walk);
  }));
  const ancestors = createMemo(() => mindAncestors(nodes(), selectedId()));
  const moveOptions = createMemo(() => moveTargetOptions(nodes(), selectedId()));

  onMount(() => {
    try {
      const stop = mindsBridge().onChange(() => bumpMinds());
      onCleanup(stop);
    } catch {
      /* Core offline: resource error handles this. */
    }
  });

  createEffect(() => {
    const mind = selected();
    setName(mind?.name ?? "");
    setColor(isHexColor(mind?.color ?? "") ? mind!.color : "#7EA8F0");
    setIcon(pickerIcon(mind?.icon ?? "brain"));
  });

  const refresh = async () => {
    bumpMinds();
    await refetch();
  };

  const createMind = async () => {
    const label = createName().trim();
    if (!label) return;
    const parentId = createParent();
    try {
      await mindsBridge().minds.create({
        parentId,
        name: label,
        color: createColor(),
        icon: createIcon(),
        sortOrder: nextSortOrder(nodes(), parentId),
      });
      setCreateName("");
      setCreateOpen(false);
      setMessage("");
      await refresh();
    } catch (error) {
      setMessage(error instanceof Error ? error.message : "Mind could not be created.");
    }
  };

  const saveMind = async () => {
    const current = selected();
    if (!current || !name().trim()) return;
    try {
      const nextColor = color();
      await mindsBridge().minds.update({
        id: current.id,
        expectedRev: wireRev(current.rev),
        name: name().trim(),
        icon: icon(),
        ...(isHexColor(nextColor) ? { color: nextColor } : { clearColor: true }),
      });
      setMessage("");
      await refresh();
    } catch (error) {
      setMessage(error instanceof Error ? error.message : "Mind could not be updated.");
    }
  };

  const requestDelete = async () => {
    const current = selected();
    if (!current) return;
    try {
      setDeletePreview(await loadDeletePreview(mindsBridge(), nodes(), current.id));
      setMessage("");
    } catch {
      setMessage("Mind cascade details are unavailable. Try again when the core is online.");
    }
  };

  const confirmDelete = async () => {
    const current = selected();
    const preview = deletePreview();
    if (!current || !preview) return;
    try {
      const result = await mindsBridge().minds.delete({ id: current.id, expectedRev: wireRev(current.rev) });
      const actual: DeletePreview = { deletedMindIds: result.deletedMindIds, movedMemoryCount: result.movedMemoryCount };
      setDeletePreview(undefined);
      setMessage(deletePreviewCopy(actual));
      shell.navigate("/minds");
      await refresh();
    } catch (error) {
      setMessage(error instanceof Error ? error.message : "Mind could not be deleted.");
    }
  };

  const moveSelected = async (parentId: string) => {
    const current = selected();
    if (!current || !canMoveMind(nodes(), current.id, parentId) || current.parentId === parentId) return;
    try {
      await mindsBridge().minds.update({
        id: current.id,
        expectedRev: wireRev(current.rev),
        ...parentPatch(parentId),
      });
      setMessage("");
      await refresh();
    } catch (error) {
      setMessage(error instanceof Error ? error.message : "Mind could not be moved.");
    }
  };

  const reorderChild = async (mind: { id: string; parentId: string }, direction: -1 | 1) => {
    const ordered = siblingOrder(nodes(), mind.parentId, mind.id, direction);
    if (!ordered) return;
    await mindsBridge().minds.reorder({ parentId: mind.parentId, orderedIds: ordered });
    await refresh();
  };

  const openCreate = (parentId = "") => {
    setCreateParent(parentId);
    setCreateName("");
    setCreateColor("#7EA8F0");
    setCreateIcon("brain");
    setCreateOpen(true);
  };

  return (
    <Show when={!minds.loading} fallback={<PageLoading label="Loading Minds" />}>
      <Show
        when={!minds.error}
        fallback={<PageError message="Minds are unavailable while the local core is offline." onRetry={() => void refresh()} />}
      >
        <div class="flex h-full min-h-0">
          <aside class="sparky-minds-navigation shrink-0 overflow-auto p-3" aria-label="Mind navigation">
            <div class="mb-3 flex items-center justify-between">
              <h2 class="text-sm font-semibold">Minds</h2>
              <Button size="sm" aria-label="Create Mind" onClick={() => openCreate()}>
                New
              </Button>
            </div>
            <MindsSidebarTree />
          </aside>
          <section aria-label="Mind workspace" class="min-w-0 flex-1 overflow-auto">
            <Show
              when={selected()}
              fallback={
                <>
                  <PageHeader
                    title="Minds"
                    actions={
                      <Button variant="primary" onClick={() => openCreate()}>
                        Create Mind
                      </Button>
                    }
                  />
                  <Show
                    when={tree().length}
                    fallback={
                      <PageEmpty
                        title="Create your first Mind"
                        description="Minds organize memories into a hierarchy."
                        action={
                          <Button variant="primary" onClick={() => openCreate()}>
                            Create Mind
                          </Button>
                        }
                      />
                    }
                  >
                    <div class="grid grid-cols-[repeat(auto-fill,minmax(200px,1fr))] gap-3 p-4">
                      <For each={tree()}>
                        {(mind) => (
                          <Button variant="ghost" class="sparky-mind-tile" onClick={() => shell.navigate(`/mind/${encodeURIComponent(mind.id)}`)}>
                            <Card>
                              <MindMark color={mind.color} icon={mind.icon} size={28} />
                              <p class="mt-2 break-words text-sm font-medium text-[var(--color-text-primary)]">{mind.name}</p>
                              <p class="text-xs text-[var(--color-text-muted)]">{mind.children.length} child Minds</p>
                            </Card>
                          </Button>
                        )}
                      </For>
                    </div>
                  </Show>
                </>
              }
            >
              {(mind) => (
                <>
                  <PageHeader
                    title={mind().name}
                    leading={<MindMark color={mind().color} icon={mind().icon} size={24} />}
                    breadcrumb={
                      <nav aria-label="Breadcrumb" class="flex flex-wrap items-center gap-1 text-xs text-[var(--color-text-muted)]">
                        <Button variant="ghost" size="sm" onClick={() => shell.navigate("/minds")}>
                          Minds
                        </Button>
                        <For each={ancestors()}>
                          {(item) => (
                            <>
                              <span aria-hidden="true">/</span>
                              <Button variant="ghost" size="sm" onClick={() => shell.navigate(`/mind/${encodeURIComponent(item.id)}`)}>
                                {item.name}
                              </Button>
                            </>
                          )}
                        </For>
                      </nav>
                    }
                    actions={
                      <Button onClick={() => openCreate(mind().id)}>Add child Mind</Button>
                    }
                  />
                  <div class="space-y-4 p-4">
                    <Card>
                      <div class="flex flex-wrap items-end gap-3">
                        <Field label="Name">
                          <Input value={name()} onInput={(event) => setName(event.currentTarget.value)} />
                        </Field>
                        <IconPicker label="Mind icon" value={icon()} onValueChange={(value) => setIcon(value)} />
                        <ColorSwatchPicker
                          label="Mind color"
                          value={parseColor(color())}
                          onValueChange={(details) => setColor(hexFromPicker(details))}
                          swatches={SWATCHES}
                        />
                        <Field label="Move to">
                          <Select
                            aria-label="Move to"
                            value={mind().parentId}
                            options={moveOptions()}
                            onChange={(value) => void moveSelected(value)}
                          />
                        </Field>
                        <Button variant="primary" onClick={() => void saveMind()}>
                          Save changes
                        </Button>
                        <Button variant="danger" onClick={() => void requestDelete()}>
                          Delete Mind
                        </Button>
                      </div>
                    </Card>
                    <Show when={message()}>
                      <p role="status" class="text-sm text-[var(--color-text-secondary)]">
                        {message()}
                      </p>
                    </Show>
                    <section aria-labelledby="children-title">
                      <h2 id="children-title" class="mb-3 text-sm font-semibold">
                        Child Minds
                      </h2>
                      <Show
                        when={children().length}
                        fallback={
                          <PageEmpty
                            title="No child Minds"
                            description="Add a child Mind to build this hierarchy."
                            action={<Button onClick={() => openCreate(mind().id)}>Add child Mind</Button>}
                          />
                        }
                      >
                        <div class="grid grid-cols-[repeat(auto-fill,minmax(200px,1fr))] gap-3">
                          <For each={children()}>
                            {(child) => (
                              <Card>
                                <Button variant="ghost" class="sparky-mind-tile" onClick={() => shell.navigate(`/mind/${encodeURIComponent(child.id)}`)}>
                                  <MindMark color={child.color} icon={child.icon} size={24} />
                                  <p class="mt-2 text-sm text-[var(--color-text-primary)]">{child.name}</p>
                                </Button>
                                <div class="mt-2 flex gap-2">
                                  <Button size="sm" aria-label={`Move ${child.name} up`} onClick={() => void reorderChild(child, -1)}>
                                    Move up
                                  </Button>
                                  <Button size="sm" aria-label={`Move ${child.name} down`} onClick={() => void reorderChild(child, 1)}>
                                    Move down
                                  </Button>
                                </div>
                              </Card>
                            )}
                          </For>
                        </div>
                      </Show>
                    </section>
                  </div>
                  <Show when={memoryCollection()}>{collection =>
                    <section aria-label="Mind memories" class="flex min-h-0 flex-col">
                      <Dynamic component={collection()} mindId={mind().id} />
                    </section>
                  }</Show>
                </>
              )}
            </Show>
          </section>
        </div>
        <Dialog
          open={createOpen()}
          title="Create Mind"
          description="Create a Mind to organize memories."
          onClose={() => setCreateOpen(false)}
          footer={
            <>
              <Button variant="ghost" onClick={() => setCreateOpen(false)}>
                Cancel
              </Button>
              <Button variant="primary" onClick={() => void createMind()}>
                Create
              </Button>
            </>
          }
        >
          <div class="space-y-3 p-4">
            <Field label="Name">
              <Input value={createName()} onInput={(event) => setCreateName(event.currentTarget.value)} />
            </Field>
            <IconPicker label="Mind icon" value={createIcon()} onValueChange={(value) => setCreateIcon(value)} />
            <ColorSwatchPicker
              label="Mind color"
              value={parseColor(createColor())}
              onValueChange={(details) => setCreateColor(hexFromPicker(details))}
              swatches={SWATCHES}
            />
          </div>
        </Dialog>
        <Dialog
          open={!!deletePreview()}
          title="Delete Mind"
          description="Review the cascade before deleting."
          onClose={() => setDeletePreview(undefined)}
          footer={
            <>
              <Button variant="ghost" onClick={() => setDeletePreview(undefined)}>
                Cancel
              </Button>
              <Button variant="danger" onClick={() => void confirmDelete()}>
                Delete Mind
              </Button>
            </>
          }
        >
          <p class="p-4">{deletePreview() ? deletePreviewCopy(deletePreview()!) : ""}</p>
        </Dialog>
      </Show>
    </Show>
  );
}
