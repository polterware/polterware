import { createEffect, createSignal, onCleanup, Show } from "solid-js";
import { Button, Field, Input, Textarea } from "@polterware/frontend-design-system";
import type { ShellBridge } from "./bridge";
import { createQuickMemory, parseQuickAdd } from "./quick-add";

export function MemoryComposer(props: {
  open: boolean;
  bridge?: ShellBridge;
  live: boolean;
  onCreated: (id: string) => void;
  onClose: () => void;
}) {
  const [title, setTitle] = createSignal("");
  const [note, setNote] = createSignal("");
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal("");
  let input: HTMLInputElement | undefined;
  const preview = () => {
    try { return { parsed: title().trim() ? parseQuickAdd(title()) : undefined, error: "" }; }
    catch (caught) { return { parsed: undefined, error: caught instanceof Error ? caught.message : String(caught) }; }
  };
  createEffect(() => { if (props.open) queueMicrotask(() => input?.focus({ preventScroll: true })); });
  createEffect(() => {
    if (!props.open) return;
    const onKey = (event: KeyboardEvent) => { if (event.key === "Escape") { event.preventDefault(); props.onClose(); } };
    document.addEventListener("keydown", onKey);
    onCleanup(() => document.removeEventListener("keydown", onKey));
  });
  async function submit(event: SubmitEvent) {
    event.preventDefault();
    const parsed = preview().parsed;
    if (busy() || !props.live || !props.bridge || !parsed) return;
    setBusy(true); setError("");
    try {
      const memory = await createQuickMemory(props.bridge, parsed, note());
      setTitle(""); setNote("");
      props.onCreated(memory.id!);
      props.onClose();
    } catch (caught) { setError(caught instanceof Error ? caught.message : String(caught)); }
    finally { setBusy(false); }
  }
  return <form class="flex h-full min-h-0 flex-col gap-3 p-4" onSubmit={submit}>
    <h2 class="text-[15px] font-medium text-[var(--color-text-primary)]">New memory</h2>
    <Field label="Title" for="sparky-memory-title">
      <Input id="sparky-memory-title" ref={input} value={title()} disabled={busy()} onInput={event => { setTitle(event.currentTarget.value); setError(""); }} />
    </Field>
    <Field label="Note" for="sparky-memory-note">
      <Textarea id="sparky-memory-note" value={note()} disabled={busy()} onInput={event => setNote(event.currentTarget.value)} />
    </Field>
    <p aria-live="polite" class="text-[12px] text-[var(--color-text-muted)]">{preview().parsed?.dueAt ? `Due ${new Date(preview().parsed.dueAt).toLocaleString()}` : preview().error || "Add a due time, #tag, @Mind, or ! priority in the title."}</p>
    <Show when={!props.live}><p role="status">Core offline — creation unavailable</p></Show>
    <Show when={error()}><p role="alert">{error()}</p></Show>
    <Button type="submit" disabled={busy() || !props.live || !preview().parsed}>{busy() ? "Creating…" : "Create memory"}</Button>
  </form>;
}
