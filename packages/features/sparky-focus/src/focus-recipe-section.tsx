import { Show, createResource, createSignal, onCleanup, onMount } from "solid-js";
import { Button, StatusChip } from "@polterware/frontend-design-system";
import { useSparkyShell } from "@polterware/feature-sparky-shell/feature";
import { browserBridge } from "./bridge";
import { recipeFromSchedule } from "./timer";
import { focusStore } from "./store";

/** Read-only focus recipe. Editing belongs to W2-SCHEDULE. */
export function FocusRecipeSection() {
  const shell = useSparkyShell();
  const id = () => shell.selectedMemoryId();
  const [error, setError] = createSignal("");
  const [busy, setBusy] = createSignal(false);
  const [recipe] = createResource(id, async memoryId => {
    if (!memoryId) return null;
    focusStore.noteMemory(memoryId);
    const result = await browserBridge().memories.get({ id: memoryId, include: { schedule: true } });
    return { title: result.memory?.title ?? "Focus", recipe: recipeFromSchedule(result.memory?.schedule) };
  });

  onMount(() => onCleanup(focusStore.connect()));

  const start = async () => {
    const memoryId = id();
    if (!memoryId || busy()) return;
    setBusy(true); setError("");
    try { await focusStore.startMemory(memoryId); }
    catch (value) { setError(value instanceof Error ? value.message : "Could not start focus."); }
    finally { setBusy(false); }
  };

  return (
    <div data-focus-recipe class="flex flex-col gap-2">
      <div class="flex items-center justify-between gap-2">
        <p class="text-[13px] font-medium text-[var(--color-text-primary)]">Focus recipe</p>
        <StatusChip tone={recipe.loading ? "syncing" : recipe.error ? "danger" : recipe()?.recipe ? "ok" : "muted"}>
          {recipe.loading ? "Queued" : recipe.error ? "Failed" : recipe()?.recipe ? "Live" : "Unavailable"}
        </StatusChip>
      </div>
      <Show when={recipe.error}><p role="alert" class="text-[13px] text-[var(--color-text-secondary)]">Focus recipe unavailable.</p></Show>
      <Show when={!recipe.loading && !recipe.error && !recipe()?.recipe}>
        <p class="text-[13px] text-[var(--color-text-muted)]">No focus recipe. Add one in Schedule.</p>
      </Show>
      <Show when={recipe()?.recipe}>{value => (
        <>
          <p class="text-[13px] text-[var(--color-text-secondary)]">{value().workMin}/{value().shortBreakMin} · long break {value().longBreakMin} min after {value().cycles} sessions · {value().autoContinue ? "auto-continue on" : "auto-continue off"}</p>
          <Button variant="secondary" disabled={busy() || focusStore.snapshot().isSessionActive && focusStore.snapshot().memoryId !== id()} onClick={() => void start()}>
            {focusStore.snapshot().memoryId === id() && focusStore.snapshot().isSessionActive ? "Focus running" : "Start focus"}
          </Button>
        </>
      )}</Show>
      <Show when={error()}><p role="alert" class="text-[13px] text-[var(--color-text-secondary)]">{error()}</p></Show>
    </div>
  );
}
