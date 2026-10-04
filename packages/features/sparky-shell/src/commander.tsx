import { createEffect, createMemo, createSignal, For, onCleanup, Show } from "solid-js";
import { Button, Dialog, Field, Input, MenuItem, MenuList, PageEmpty, PageLoading, StatusChip } from "@polterware/frontend-design-system";
import type { MessageShape } from "@bufbuild/protobuf";
import type { SearchHitSchema } from "@polterware/sparky-contracts";
import type { ShellBridge } from "./bridge";
import type { CommandDef } from "./feature";
import { createQuickMemory, parseQuickAdd } from "./quick-add";
import { findCommands } from "./route";

export function Commander(props: {
  open: boolean; mode: "commands" | "quick-add" | "search"; bridge?: ShellBridge;
  commands: CommandDef[]; live: boolean; onClose: () => void; onMemory: (id: string) => void;
}) {
  const [query, setQuery] = createSignal("");
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal("");
  const [notice, setNotice] = createSignal("");
  const [hits, setHits] = createSignal<MessageShape<typeof SearchHitSchema>[]>([]);
  const [searching, setSearching] = createSignal(false);
  const [searched, setSearched] = createSignal(false);
  let generation = 0;
  let input: HTMLInputElement | undefined;
  let dialogBody: HTMLDivElement | undefined;
  const commands = createMemo(() => findCommands(props.commands, query()));
  const preview = createMemo(() => {
    try { return { parsed: parseQuickAdd(query()), error: "" }; }
    catch (error) { return { parsed: undefined, error: error instanceof Error ? error.message : String(error) }; }
  });
  createEffect(() => {
    if (props.open) { props.mode; setQuery(""); setError(""); }
  });
  createEffect(() => {
    const text = query().trim();
    const current = ++generation;
    setHits([]); setNotice(""); setSearching(false); setSearched(false);
    if (!props.open || props.mode === "quick-add" || !text || !props.live || !props.bridge) return;
    const bridge = props.bridge;
    const timer = setTimeout(() => {
      setSearching(true);
      void bridge.search.search({ query: text, limit: 20 }).then(result => {
        if (generation !== current) return;
        setHits(result.hits ?? []); setNotice(result.notice ?? ""); setSearched(true);
      }).catch(error => {
        if (generation === current) setError(error instanceof Error ? error.message : String(error));
      }).finally(() => { if (generation === current) setSearching(false); });
    }, 180);
    onCleanup(() => { clearTimeout(timer); if (generation === current) generation++; });
  });
  async function execute(command: CommandDef) {
    if (busy()) return;
    setBusy(true); setError("");
    try { await command.run(); props.onClose(); }
    catch (error) { setError(error instanceof Error ? error.message : String(error)); }
    finally { setBusy(false); }
  }
  async function add() {
    if (busy() || !props.live || !props.bridge) return;
    setBusy(true); setError("");
    try {
      const memory = await createQuickMemory(props.bridge, parseQuickAdd(query()));
      props.onClose(); props.onMemory(memory.id!);
    } catch (error) { setError(error instanceof Error ? error.message : String(error)); }
    finally { setBusy(false); }
  }
  function onKeyDown(event: KeyboardEvent) {
    if (event.isComposing) return;
    const buttons = Array.from(dialogBody?.querySelectorAll<HTMLButtonElement>('button[data-command]:not(:disabled)') ?? []);
    if (event.key === "ArrowDown" || event.key === "ArrowUp") {
      event.preventDefault();
      const index = buttons.indexOf(document.activeElement as HTMLButtonElement);
      const next = event.key === "ArrowDown" ? index + 1 : index < 0 ? buttons.length - 1 : index - 1;
      if (next < 0) input?.focus(); else buttons[next % buttons.length]?.focus();
    } else if (event.key === "Enter" && event.target === input && props.mode !== "quick-add") {
      event.preventDefault(); buttons[0]?.click();
    }
  }
  return <Dialog open={props.open} title={props.mode === "quick-add" ? "New memory" : "Commander"} onClose={props.onClose}
    description={props.mode === "quick-add" ? 'Try: Buy milk tomorrow 9am #errands @Home' : "Navigate, search memories, or run an action."}>
    <div ref={dialogBody} onKeyDown={onKeyDown} class="flex min-h-0 flex-col gap-3">
      <form onSubmit={event => { event.preventDefault(); if (props.mode === "quick-add") void add(); }}>
        <Field label={props.mode === "quick-add" ? "Memory" : "Search or command"} for="sparky-command-input">
          <Input id="sparky-command-input" ref={input} value={query()} onInput={event => { setQuery(event.currentTarget.value); setError(""); }} disabled={busy()} />
        </Field>
        <Show when={props.mode === "quick-add"}>
          <div class="flex flex-col gap-3 pt-3">
            <p aria-live="polite">{preview().parsed ? `Title: ${preview().parsed!.title}${preview().parsed!.dueAt ? ` · Due: ${new Date(preview().parsed!.dueAt!).toLocaleString()}` : ""}` : preview().error}</p>
            <Show when={!props.live}><StatusChip tone="down" label="Core offline — creation unavailable" /></Show>
            <Button type="submit" disabled={busy() || !props.live || !preview().parsed}>{busy() ? "Creating…" : "Create memory"}</Button>
          </div>
        </Show>
      </form>
      <Show when={error()}><p role="alert" class="text-[var(--color-danger)]">{error()}</p></Show>
      <Show when={props.mode !== "quick-add"}>
        <MenuList>
          <For each={commands()}>{command => <MenuItem data-command disabled={busy()} onClick={() => void execute(command)}>{command.label}</MenuItem>}</For>
          <For each={hits()}>{hit => <MenuItem data-command disabled={busy()} onClick={() => { if (hit.memoryId) { props.onClose(); props.onMemory(hit.memoryId); } }}>
            <span class="flex flex-col"><span>{hit.title}</span><span class="text-xs text-[var(--color-text-muted)]">{hit.snippet}</span></span>
          </MenuItem>}</For>
        </MenuList>
        <Show when={searching()}><PageLoading label="Searching memories…" /></Show>
        <Show when={!commands().length && !hits().length && searched() && !searching()}><PageEmpty title="No results" description="Try another search." /></Show>
        <Show when={!props.live}><StatusChip tone="down" label="Core offline — memory search unavailable" /></Show>
        <Show when={notice()}><p role="status">{notice()}</p></Show>
      </Show>
    </div>
  </Dialog>;
}
