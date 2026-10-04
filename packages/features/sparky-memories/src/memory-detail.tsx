import { For, Index, Show, createEffect, createMemo, createSignal, onCleanup, onMount, on, type Component } from "solid-js";
import type { DetailSectionDef } from "@polterware/feature-sparky-shell/feature";
import { A11yDialog, Button, Checkbox, DetailPanel, Field, Input, MarkdownEditor, PageEmpty, PageError, PageLoading, Select, StatusChip, Switch, SwitchRow, Textarea, type CheckboxProps } from "@polterware/frontend-design-system";
import { trySparkyShell } from "./shell-access";
import { checklistInputs, errorText, localDate, reorderChecklist, type MemoryRecord } from "./model";
import { memoriesStore, type MemoriesStore } from "./store";
import { UndoNotice } from "./undo-notice";

export type MemoryDetailProps = { memoryId: string; sections?: DetailSectionDef[]; store?: MemoriesStore };
export function orderedSections(sections: DetailSectionDef[], memoryId: string) {
  return sections.filter(section => section.visible?.(memoryId) !== false).sort((a, b) => a.order - b.order || a.id.localeCompare(b.id));
}
/** The shell owns the single DetailPanel Root/Content/Header; this feature owns its record body. */
export function MemoryDetail(props: MemoryDetailProps) {
  const store = props.store ?? memoriesStore;
  const shell = trySparkyShell();
  onMount(() => onCleanup(store.connect()));
  const [error, setError] = createSignal("");
  const [pending, setPending] = createSignal(false);
  const [confirmDelete, setConfirmDelete] = createSignal(false);
  const [actionError, setActionError] = createSignal("");
  const current = () => store.record(props.memoryId);
  createEffect(on(() => props.memoryId, id => {
    setError(""); setPending(!store.record(id));
    let active = true;
    void store.load(id).catch(value => { if (active) setError(errorText(value)); }).finally(() => { if (active) setPending(false); });
    onCleanup(() => { active = false; void store.flush(id); });
  }));
  const sections = createMemo(() => orderedSections(props.sections ?? shell?.detailSections() ?? [], props.memoryId));
  async function remove() {
    try { await store.queueDelete(props.memoryId); setConfirmDelete(false); setActionError(""); }
    catch (value) { setActionError(errorText(value)); }
  }
  const edit = (patch: Parameters<typeof store.edit>[1]) => store.edit(props.memoryId, patch);
  const keyboard = (event: KeyboardEvent) => {
    if (!(event.target instanceof HTMLElement)) return;
    if ((event.metaKey || event.ctrlKey) && event.key === "Backspace" && !event.target.closest("input, textarea, [contenteditable=true]")) { event.preventDefault(); setConfirmDelete(true); }
    if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "s") { event.preventDefault(); void store.flush(props.memoryId); }
  };
  return <div onKeyDown={keyboard}>
    <Show when={pending()}><PageLoading label="Loading memory" /></Show>
    <Show when={error()}><PageError title="Memory unavailable" message={error()} onRetry={() => { setError(""); void store.load(props.memoryId, true).catch(value => setError(errorText(value))); }} /></Show>
    <Show when={store.deleting().has(props.memoryId)}>
      <PageEmpty title="Deletion queued" description="Undo within 8 seconds to keep this memory and its related data." />
      <UndoNotice store={store} />
    </Show>
    <Show when={!current() && !pending() && !error() && !store.deleting().has(props.memoryId)}><PageEmpty title="Memory removed" description="Select another memory to continue." /></Show>
    <Show when={!store.deleting().has(props.memoryId) && current()}>{memory => <DetailPanel.Body layout="record">
      <DetailPanel.Main>
        <div class="flex flex-col gap-4">
          <Show when={store.state(props.memoryId) === "conflict" || store.state(props.memoryId) === "failed"}>
            <div role="alert" class="flex flex-col gap-2">
              <StatusChip tone="danger">{store.state(props.memoryId) === "conflict" ? "Edit conflict" : "Save failed"}</StatusChip>
              <p>{store.drafts()[props.memoryId] ? "Your local edits are retained. Reload discards them; Overwrite applies them to the latest revision." : "The action failed. Reload the memory before retrying."} {store.errors()[props.memoryId]}</p>
              <div class="flex gap-2">
                <Button variant="secondary" onClick={() => void store.resolve(props.memoryId, "reload")}>Reload</Button>
                <Show when={store.drafts()[props.memoryId]}><Button onClick={() => void store.resolve(props.memoryId, "overwrite")}>Overwrite</Button></Show>
              </div>
            </div>
          </Show>
          <Field label="Title" for={`memory-title-${props.memoryId}`}>
            <Input id={`memory-title-${props.memoryId}`} data-memory-title value={memory().title} onInput={event => edit({ title: event.currentTarget.value })} onBlur={() => void store.flush(props.memoryId)} />
          </Field>
          <StatusChip tone={store.state(props.memoryId) === "live" ? "ok" : "syncing"}>{store.state(props.memoryId) === "live" ? "Live · Saved" : store.state(props.memoryId) === "queued" ? "Queued · Unsaved edits" : store.state(props.memoryId) === "saving" ? "Saving" : "Unsaved edits"}</StatusChip>
          <DetailPanel.Section title="Note">
            <MarkdownEditor aria-label="Memory note" value={memory().note} onChange={note => edit({ note })} />
          </DetailPanel.Section>
          <Checklist memory={memory()} store={store} />
          <DetailPanel.Section title="Tags">
            <Show when={store.tags().length} fallback={<p>No tags yet.</p>}>
              <div class="flex flex-wrap gap-3"><For each={store.tags()}>{tag => <Checkbox label={tag.name ?? "Unnamed tag"} checked={memory().tagIds.includes(tag.id ?? "")} onCheckedChange={({ checked }) => {
                const ids = memory().tagIds.filter(id => id !== tag.id);
                if (checked === true && tag.id) ids.push(tag.id);
                store.editTags(props.memoryId, ids);
              }} />}</For></div>
            </Show>
          </DetailPanel.Section>
          <For each={sections()}>{section => <DetailPanel.Section>
            <SectionContribution section={section} memoryId={props.memoryId} />
          </DetailPanel.Section>}</For>
          <DetailPanel.Section title="Activity">
            <For each={store.activity().filter(event => event.entityId === props.memoryId).slice(0, 5)} fallback={<p>Activity unavailable for this memory in the latest 100 changes.</p>}>
              {event => <p class="text-xs text-[var(--color-text-muted)]">{event.summary ?? event.op} · {event.actor ?? "Unknown actor"} · {event.at ? new Date(event.at).toLocaleString() : "Time unavailable"}</p>}
            </For>
          </DetailPanel.Section>
          <UndoNotice store={store} />
        </div>
      </DetailPanel.Main>
      <DetailPanel.Rail>
        <DetailPanel.Properties>
          <DetailPanel.Property label="Mind" value={<Select aria-label="Mind" options={[{ value: "", label: "Inbox" }, ...store.minds().map(mind => ({ value: mind.id ?? "", label: mind.name ?? "Unnamed Mind" }))]} value={memory().mindId} onChange={mindId => edit(mindId ? { mindId, clearMindId: false } : { clearMindId: true })} />} />
          <DetailPanel.Property label="Status" value={<Button variant="secondary" onClick={() => void store.toggle(props.memoryId, "status")}>{memory().status === "completed" ? "Reopen" : "Complete"}</Button>} />
          <DetailPanel.Property label="Pin" value={<Switch aria-label="Pin memory" checked={memory().pinned} onCheckedChange={() => void store.toggle(props.memoryId, "pinned")} />} />
          <DetailPanel.Property label="Priority" value={<Select aria-label="Priority" value={memory().priority?.toString() ?? ""} options={[{ value: "", label: "None" }, { value: "1", label: "Low" }, { value: "2", label: "Normal" }, { value: "3", label: "High" }]} onChange={value => edit(value ? { priority: Number(value), clearPriority: false } : { clearPriority: true })} />} />
          <DetailPanel.Property label="Due" value={<Input type={memory().dueAllDay ? "date" : "datetime-local"} aria-label="Due date" value={dueInput(memory())} onChange={event => {
            const value = event.currentTarget.value;
            if (!value) edit({ clearDueAt: true });
            else { const date = new Date(memory().dueAllDay ? `${value}T00:00:00Z` : value); if (!Number.isNaN(date.getTime())) edit({ dueAt: date.toISOString(), clearDueAt: false }); }
          }} />} />
          <DetailPanel.Property label="All day" value={<Switch aria-label="All-day deadline" checked={memory().dueAllDay} onCheckedChange={value => edit({ dueAllDay: value })} />} />
          <DetailPanel.Property label="Delete" value={<Button variant="danger" disabled={store.deleting().has(props.memoryId)} onClick={() => setConfirmDelete(true)}>Delete memory</Button>} />
        </DetailPanel.Properties>
        <Show when={store.metadataError()}><p role="status">Mind, tags or activity unavailable: {store.metadataError()}</p></Show>
      </DetailPanel.Rail>
    </DetailPanel.Body>}</Show>
    <A11yDialog open={confirmDelete()} title="Delete memory?" size="sm" onClose={() => setConfirmDelete(false)} footer={<><Button variant="secondary" onClick={() => setConfirmDelete(false)}>Cancel</Button><Button variant="danger" onClick={() => void remove()}>Delete</Button></>}>
      <p>Deletion is queued for 8 seconds. Undo cancels it and preserves the memory and all its related data.</p>
      <Show when={actionError()}><p role="alert">{actionError()}</p></Show>
    </A11yDialog>
  </div>;
}
function dueInput(memory: MemoryRecord) {
  if (!memory.dueAt) return "";
  if (memory.dueAllDay) return memory.dueAt.slice(0, 10);
  const date = new Date(memory.dueAt);
  return `${localDate(date)}T${String(date.getHours()).padStart(2, "0")}:${String(date.getMinutes()).padStart(2, "0")}`;
}
function Checklist(props: { memory: MemoryRecord; store: MemoriesStore }) {
  const items = () => checklistInputs(props.memory.checklist);
  const save = (next: ReturnType<typeof items>) => props.store.editChecklist(props.memory.id, next);
  return <DetailPanel.Section title="Checklist" aside={<Button variant="ghost" onClick={() => save([...items(), { title: "New item", detail: "", done: false, sortOrder: items().length }])}>Add item</Button>}>
    <div class="flex flex-col gap-3">
      <SwitchRow label="Complete memory when every item is done" checked={props.memory.autoCompleteOnChecklist} onCheckedChange={value => props.store.edit(props.memory.id, { autoCompleteOnChecklist: value })} />
      <Index each={items()}>{(item, index) => <div class="flex flex-col gap-2">
        <div class="flex items-center gap-2">
          <Checkbox {...roundCheckbox} aria-label={`Complete checklist item ${index + 1}`} checked={item().done ?? false} onCheckedChange={({ checked }) => {
            if (item().id) void props.store.toggleChecklist(props.memory.id, item().id!, checked === true);
            else save(items().map((row, i) => i === index ? { ...row, done: checked === true } : row));
          }} />
          <Input aria-label={`Checklist title ${index + 1}`} value={item().title ?? ""} onInput={event => save(items().map((row, i) => i === index ? { ...row, title: event.currentTarget.value } : row))} />
        </div>
        <Textarea aria-label={`Checklist detail ${index + 1}`} value={item().detail ?? ""} onInput={event => save(items().map((row, i) => i === index ? { ...row, detail: event.currentTarget.value } : row))} />
        <div class="flex gap-2">
          <Button variant="ghost" disabled={index === 0} onClick={() => save(reorderChecklist(items(), index, -1))}>Move up</Button>
          <Button variant="ghost" disabled={index === items().length - 1} onClick={() => save(reorderChecklist(items(), index, 1))}>Move down</Button>
          <Button variant="ghost" onClick={() => save(items().filter((_, i) => i !== index))}>Remove item</Button>
        </div>
      </div>}</Index>
      <Show when={!items().length}><p>No checklist items. Add one to break this memory into steps.</p></Show>
    </div>
  </DetailPanel.Section>;
}

const roundCheckbox: CheckboxProps = { ["shape"]: "round" };
function SectionContribution(props: { section: DetailSectionDef; memoryId: string }) {
  const Content: Component<{ memoryId?: string }> = props.section.component;
  return <Content memoryId={props.memoryId} />;
}
