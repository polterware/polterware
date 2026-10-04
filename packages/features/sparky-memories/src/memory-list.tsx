import { For, Show, createEffect, createMemo, createSignal, onCleanup, onMount } from "solid-js";
import { A11yDialog, Button, Checkbox, DataTable, Field, FilterChip, FilterPopover, Input, ListChecks, PageEmpty, PageError, PageHeader, PageLoading, PageOffline, PageToolbar, SearchBar, SegmentedControl, Select, StatusChip, type CheckboxProps } from "@polterware/frontend-design-system";
import { browserBridge } from "./bridge";
import { errorText, filterRows, todayGroup, DEFAULT_FILTER, type MemoryRow } from "./model";
import { MemoryDetailHost } from "./host";
import { trySparkyShell } from "./shell-access";
import { memoriesStore, type MemoriesStore } from "./store";
import { UndoNotice } from "./undo-notice";

export type MemoryCollectionProps = { mindId?: string; mode?: "today" | "inbox" | "mind"; store?: MemoriesStore; onOpen?: (id: string) => void; virtualized?: boolean };
export function MemoryCollection(props: MemoryCollectionProps) {
  const store = props.store ?? memoriesStore;
  const shell = trySparkyShell();
  const [query, setQuery] = createSignal("");
  const [status, setStatus] = createSignal("active");
  const [pinned, setPinned] = createSignal(false);
  const [tagId, setTagId] = createSignal("");
  const [order, setOrder] = createSignal("due");
  const [layout, setLayout] = createSignal("list");
  const [selection, setSelection] = createSignal<ReadonlySet<string>>(new Set<string>());
  const [activeId, setActiveId] = createSignal<string | null>(null);
  const [creating, setCreating] = createSignal(false);
  const [title, setTitle] = createSignal("");
  const [busy, setBusy] = createSignal(false);
  const [actionError, setActionError] = createSignal("");
  const [deleteIds, setDeleteIds] = createSignal<string[]>([]);
  const [bulkMind, setBulkMind] = createSignal("");
  const [now, setNow] = createSignal(new Date());
  let root: HTMLDivElement | undefined;
  const mode = () => props.mode ?? (props.mindId ? "mind" : "inbox");
  onMount(() => {
    root?.addEventListener("keydown", keyboard, true);
    const disconnect = store.connect();
    const timer = setInterval(() => setNow(new Date()), 60000);
    const unregDetail = shell?.registerMemoryDetail(({ memoryId }) => <MemoryDetailHost memoryId={memoryId} store={store} />);
    const unregKeys = shell?.registerCollectionShortcuts(action => {
      const id = activeId();
      if (!id) return;
      if (action === "open") open(id);
      else if (action === "toggle-done") void store.toggle(id, "status");
      else if (action === "edit-title") { open(id); queueMicrotask(() => document.querySelector<HTMLInputElement>("[data-memory-title]")?.focus()); }
      else if (action === "pin") void store.toggle(id, "pinned");
      else if (action === "delete") setDeleteIds([id]);
    });
    onCleanup(() => { disconnect(); clearInterval(timer); unregDetail?.(); unregKeys?.(); root?.removeEventListener("keydown", keyboard, true); });
  });
  const groups = createMemo(() => {
    const dates = new Map<string, "Overdue" | "Today" | "Upcoming 7 days">();
    for (const occurrence of store.occurrences()) {
      if (occurrence.completed || !occurrence.memoryId) continue;
      const group = todayGroup({ dueAt: occurrence.occurrenceAt, dueAllDay: occurrence.allDay ?? false }, now());
      if (group && (!dates.has(occurrence.memoryId) || group === "Today")) dates.set(occurrence.memoryId, group);
    }
    return dates;
  });
  const filtered = createMemo(() => filterRows(store.rows(), { ...DEFAULT_FILTER, query: query(), status: status(), pinned: pinned(), tagId: tagId(), order: order(),
    inbox: mode() === "inbox", mindId: mode() === "mind" ? props.mindId : undefined }));
  const displayRows = createMemo(() => {
    if (mode() !== "today") return filtered();
    const rank = { Overdue: 0, Today: 1, "Upcoming 7 days": 2 };
    return filtered().filter(row => todayGroup(row, now()) || groups().has(row.id))
      .sort((a, b) => rank[todayGroup(a, now()) ?? groups().get(a.id)!] - rank[todayGroup(b, now()) ?? groups().get(b.id)!]);
  });
  const group = (row: MemoryRow) => mode() === "today" ? todayGroup(row, now()) ?? groups().get(row.id) : row.pinned ? "Pinned" : "Memories";
  createEffect(() => {
    const ids = new Set(displayRows().map(row => row.id));
    setSelection(previous => new Set([...previous].filter(id => ids.has(id))));
  });
  const open = (id: string) => {
    setActiveId(id);
    if (props.onOpen) props.onOpen(id);
    else if (shell) shell.openMemory(id);
    else void browserBridge().app.openDeepLink(`sparky://memory/${encodeURIComponent(id)}`).catch((error: unknown) => setActionError(errorText(error)));
  };
  async function create() {
    if (!title().trim() || busy()) return;
    setBusy(true); setActionError("");
    try { const memory = await store.create({ title: title().trim(), mindId: props.mindId ?? "" }); setCreating(false); setTitle(""); open(memory.id); }
    catch (error) { setActionError(errorText(error)); }
    finally { setBusy(false); }
  }
  async function bulk(action: "complete" | "pin" | "move") {
    setBusy(true); setActionError("");
    try {
      const selectedIds = [...selection()];
      // Serialized per entity and expectedRev checked: partial failures remain visible individually.
      for (const id of selectedIds) {
        if (action === "move") { store.edit(id, bulkMind() ? { mindId: bulkMind(), clearMindId: false } : { clearMindId: true }); await store.flush(id); }
        else await store.toggle(id, action === "complete" ? "status" : "pinned", true);
      }
      if (selectedIds.some(id => ["failed", "conflict"].includes(store.state(id)))) throw new Error("Some actions failed. Open the affected memory to resolve its retained edits.");
      setSelection(new Set<string>());
    } catch (error) { setActionError(errorText(error)); }
    finally { setBusy(false); }
  }
  async function remove() {
    setBusy(true); setActionError("");
    try { for (const id of deleteIds()) await store.queueDelete(id); setDeleteIds([]); setSelection(new Set<string>()); }
    catch (error) { setActionError(errorText(error)); }
    finally { setBusy(false); }
  }
  const keyboard = (event: KeyboardEvent) => {
    const element = event.target;
    if (!(element instanceof HTMLElement)) return;
    const editing = element.closest("input, textarea, [contenteditable=true], [role=dialog], [role=menu]");
    if (editing) return;
    const id = element.closest<HTMLElement>("[data-row-id]")?.dataset.rowId ?? activeId();
    if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "n") { event.preventDefault(); event.stopPropagation(); setCreating(true); }
    else if (event.key === "/") { event.preventDefault(); event.stopPropagation(); root?.querySelector<HTMLInputElement>('[role="searchbox"]')?.focus(); }
    else if (id && (event.metaKey || event.ctrlKey) && event.key === "Backspace") { event.preventDefault(); event.stopPropagation(); setDeleteIds([id]); }
    else if (id && event.key.toLowerCase() === "p" && !event.metaKey && !event.ctrlKey) { event.preventDefault(); event.stopPropagation(); void store.toggle(id, "pinned"); }
    else if (id && event.key === " " && element.matches("[data-row-id]")) { event.preventDefault(); event.stopPropagation(); void store.toggle(id, "status"); }
    else if (id && event.key.toLowerCase() === "e" && !event.metaKey && !event.ctrlKey) { event.preventDefault(); event.stopPropagation(); open(id); queueMicrotask(() => document.querySelector<HTMLInputElement>("[data-memory-title]")?.focus()); }
  };
  return <div ref={element => { root = element; }} class="flex h-full min-h-0 flex-col gap-3">
    <Show when={mode() !== "mind"}><PageHeader title={mode() === "today" ? "Today" : "Inbox"} /></Show>
    <PageToolbar.Root>
      <PageToolbar.Leading><div class="flex flex-wrap items-center gap-2">
        <SearchBar aria-label="Filter memories" placeholder="Filter memories" value={query()} onInput={setQuery} />
        <FilterChip selected={status() === "active"} onClick={() => setStatus("active")}>Active</FilterChip>
        <FilterChip selected={status() === "completed"} onClick={() => setStatus("completed")}>Completed</FilterChip>
        <FilterChip selected={!status()} onClick={() => setStatus("")}>All</FilterChip>
        <FilterChip selected={pinned()} onClick={() => setPinned(!pinned())}>Pinned</FilterChip>
        <FilterPopover ariaLabel="Filter by tag" triggerLabel="Tags" active={Boolean(tagId())} onClear={() => setTagId("")} groups={[{ id: "tags", label: "Tags", selectionMode: "single", options: store.tags().map(tag => ({ id: tag.id ?? "", label: tag.name ?? "Unnamed tag", checked: tagId() === tag.id, onCheckedChange: checked => setTagId(checked ? tag.id ?? "" : "") })) }]} />
        <Select class="w-auto min-w-32" aria-label="Sort memories" value={order()} options={[{ value: "due", label: "Due date" }, { value: "title", label: "Title" }, { value: "updated", label: "Recently updated" }]} onChange={setOrder} />
        <SegmentedControl aria-label="Memory layout" value={layout()} onValueChange={details => setLayout(details.value ?? "list")} options={[{ value: "list", label: "List" }, { value: "table", label: "Table" }]} />
      </div></PageToolbar.Leading>
      <PageToolbar.Actions><Button onClick={() => setCreating(true)}>New memory</Button></PageToolbar.Actions>
    </PageToolbar.Root>
    <Show when={actionError()}><PageError title="Action failed" message={actionError()} /></Show>
    <For each={Object.entries(store.errors()).filter(([, value]) => Boolean(value))}>{([id, message]) => <div role="alert" class="flex items-center gap-2"><p>Memory action failed: {message}</p><Button variant="secondary" onClick={() => open(id)}>Open memory</Button></div>}</For>
    <Show when={mode() === "today" && store.scheduleError()}><p role="status">Scheduled memories unavailable: {store.scheduleError()}. Due memories are shown.</p></Show>
    <Show when={store.loadError()}><PageOffline title="Memories unavailable" message={store.loadError()} onRetry={() => void store.refresh()} /></Show>
    <Show when={store.loading() && !store.rows().length}><PageLoading label="Loading memories" /></Show>
    <Show when={!store.loadError() && (!store.loading() || store.rows().length)}>
      <Show when={displayRows().length} fallback={<PageEmpty title={query() || pinned() || tagId() ? "No matching memories" : mode() === "today" ? "Nothing due today" : "No memories here"} description={mode() === "inbox" ? "Memories without a Mind appear here." : "Create a memory or change your filters."} action={<Button onClick={() => setCreating(true)}>New memory</Button>} />}>
        <DataTable.Root rows={displayRows()} getRowId={row => row.id} selectionMode="multiple" selectedRowIds={selection()} onSelectionChange={setSelection} activeRowId={activeId()} onActiveRowChange={setActiveId} virtualized={props.virtualized !== false} estimateRowHeight={48}>
          <DataTable.Table aria-label="Memories">
            <DataTable.Header><DataTable.SelectionCell /><DataTable.ColumnHeader id="title" sortable={false}>Memory</DataTable.ColumnHeader>
              <DataTable.ColumnHeader id="group" sortable={false}>{mode() === "today" ? "When" : "Section"}</DataTable.ColumnHeader>
              <Show when={layout() === "table"}><DataTable.ColumnHeader id="due" sortable={false}>Due</DataTable.ColumnHeader><DataTable.ColumnHeader id="priority" sortable={false}>Priority</DataTable.ColumnHeader></Show>
              <DataTable.ColumnHeader id="actions" sortable={false}>Actions</DataTable.ColumnHeader>
            </DataTable.Header>
            <DataTable.Body>{(row: MemoryRow) => <DataTable.Row row={row} rowId={row.id} aria-label={row.title} onActivate={() => open(row.id)}>
              <DataTable.SelectionCell rowId={row.id} label={`Select ${row.title}`} />
              <DataTable.PrimaryCell leading={<ListChecks size={16} />} title={row.title} />
              <DataTable.Cell><span>{group(row)}</span></DataTable.Cell>
              <Show when={layout() === "table"}><DataTable.Cell>{row.dueAt ? new Date(row.dueAt).toLocaleDateString() : "No due date"}</DataTable.Cell><DataTable.Cell>{row.priority ?? "None"}</DataTable.Cell></Show>
              <DataTable.Cell><div class="flex items-center gap-2"><Checkbox {...roundCheckbox} aria-label={`Complete ${row.title}`} checked={row.status === "completed"} onCheckedChange={() => void store.toggle(row.id, "status")} /><Button variant="ghost" aria-label={`Pin ${row.title}`} aria-pressed={row.pinned} onClick={() => void store.toggle(row.id, "pinned")}>{row.pinned ? "Unpin" : "Pin"}</Button></div></DataTable.Cell>
            </DataTable.Row>}</DataTable.Body>
          </DataTable.Table>
          <DataTable.SelectionToolbar>{() => <>
            <Button variant="secondary" disabled={busy()} onClick={() => void bulk("complete")}>Complete</Button>
            <Button variant="secondary" disabled={busy()} onClick={() => void bulk("pin")}>Pin</Button>
            <Select aria-label="Move selected memories to Mind" value={bulkMind()} onChange={setBulkMind} options={[{ value: "", label: "Inbox" }, ...store.minds().map(mind => ({ value: mind.id ?? "", label: mind.name ?? "Unnamed Mind" }))]} />
            <Button variant="secondary" disabled={busy()} onClick={() => void bulk("move")}>Move</Button>
            <Button variant="danger" disabled={busy()} onClick={() => setDeleteIds([...selection()])}>Delete</Button>
          </>}</DataTable.SelectionToolbar>
        </DataTable.Root>
        <StatusChip class="self-start" tone="ok">Live · {displayRows().length} memories</StatusChip>
      </Show>
    </Show>
    <UndoNotice store={store} />
    <A11yDialog open={creating()} title="New memory" size="sm" onClose={() => setCreating(false)} footer={<><Button variant="secondary" onClick={() => setCreating(false)}>Cancel</Button><Button disabled={!title().trim() || busy()} onClick={() => void create()}>Create memory</Button></>}>
      <form onSubmit={event => { event.preventDefault(); void create(); }}><Field label="Title" for="new-memory-title"><Input id="new-memory-title" value={title()} onInput={event => setTitle(event.currentTarget.value)} autofocus /></Field></form>
      <Show when={actionError()}><p role="alert">{actionError()}</p></Show>
    </A11yDialog>
    <A11yDialog open={deleteIds().length > 0} title="Delete selected memories?" size="sm" onClose={() => setDeleteIds([])} footer={<><Button variant="secondary" onClick={() => setDeleteIds([])}>Cancel</Button><Button variant="danger" disabled={busy()} onClick={() => void remove()}>Delete</Button></>}>
      <p>Queue deletion of {deleteIds().length} memories. Undo is available for 8 seconds before the memory and its related data are deleted.</p>
      <Show when={actionError()}><p role="alert">{actionError()}</p></Show>
    </A11yDialog>
  </div>;
}
export function TodayPage() { return <MemoryCollection mode="today" />; }
export function InboxPage() { return <MemoryCollection mode="inbox" />; }
export function MindMemoriesPage(props: { mindId: string }) { return <MemoryCollection mode="mind" mindId={props.mindId} />; }

const roundCheckbox: CheckboxProps = { ["shape"]: "round" };
