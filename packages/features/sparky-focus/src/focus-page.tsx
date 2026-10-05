import { For, Show, createEffect, createMemo, createSignal, onCleanup, onMount } from "solid-js";
import {
  Button, Card, FilterChipGroup, PageContent, PageEmpty, PageError, PageHeader, PageLoading, PageOffline,
  PageToolbar, Popover, ProgressRing, StatusChip,
} from "@polterware/frontend-design-system";
import { useSparkyShell } from "@polterware/feature-sparky-shell/feature";
import { formatRemaining, phaseLabel, WORK_PRESETS, type FocusRecipe } from "./timer";
import { focusStore, type FocusStore } from "./store";

function honestyTone(state: string): "ok" | "syncing" | "danger" | "muted" {
  if (state === "live") return "ok";
  if (state === "queued") return "syncing";
  if (state === "failed") return "danger";
  return "muted";
}

export function FocusPage(props: { store?: FocusStore }) {
  const store = props.store ?? focusStore;
  const shell = useSparkyShell();
  const [recipeOpen, setRecipeOpen] = createSignal(false);
  let recipeAnchor: HTMLButtonElement | undefined;
  const snap = () => store.snapshot();
  const recipe = createMemo(() => snap().recipe ?? store.quickRecipe());

  onMount(() => onCleanup(store.connect()));

  createEffect(() => {
    const url = shell.deepLink();
    if (!url || url.hostname !== "memory" || url.searchParams.get("action") !== "focus") return;
    const id = url.pathname.replace(/^\//, "");
    if (!id) return;
    void store.startMemory(id).catch(error => store.reload().then(() => { throw error; }).catch(() => {}));
  });

  const cycles = () => Array.from({ length: recipe().cycles }, (_, index) => index);
  const filled = () => {
    const count = recipe().cycles;
    if (snap().phase === "long_break") return count;
    return snap().completedPomodoros % count;
  };
  const primary = () => {
    if (!snap().isSessionActive) return { label: "Start", action: () => void store.startQuick(), kind: "start" as const };
    if (snap().isWaitingForManualStart) return { label: snap().phase === "work" ? "Start focus" : "Start break", action: () => store.resume(), kind: "start" as const };
    if (snap().isRunning) return { label: "Pause", action: () => store.pause(), kind: "pause" as const };
    return { label: "Resume", action: () => store.resume(), kind: "start" as const };
  };

  const applyWork = (minutes: number) => {
    const next: FocusRecipe = { ...recipe(), workMin: Number(minutes) };
    store.setQuickRecipe(next);
  };

  const title = () => {
    if (!snap().isSessionActive) return "Focus";
    if (snap().phase !== "work") return phaseLabel(snap().phase);
    return snap().memoryTitle || "Focus";
  };

  return (
    <div data-focus-page class="flex h-full min-h-0 flex-col">
      <PageHeader
        title={title()}
        subtitle={<StatusChip tone={honestyTone(store.loadState())}>{store.loadState() === "live" ? "Live" : store.loadState() === "queued" ? "Queued" : store.loadState() === "failed" ? "Failed" : "Unavailable"}</StatusChip>}
        actions={
          <Show when={!snap().isSessionActive}>
            <Button ref={element => { recipeAnchor = element; }} aria-haspopup="dialog" aria-expanded={recipeOpen()} variant="secondary" onClick={() => setRecipeOpen(open => !open)}>Recipe</Button>
          </Show>
        }
      />
      <Popover open={recipeOpen()} onClose={() => setRecipeOpen(false)} title="Quick focus recipe" anchorRef={() => recipeAnchor} width={320}>
        <div class="flex flex-col gap-3 p-2">
          <FilterChipGroup aria-label="Work duration" value={String(recipe().workMin)} onChange={value => applyWork(Number(value))} items={WORK_PRESETS.map(minutes => ({ id: String(minutes), label: `${minutes} min` }))} />
          <p class="text-[13px] text-[var(--color-text-muted)]">{recipe().workMin}/{recipe().shortBreakMin} · long break {recipe().longBreakMin} min after {recipe().cycles} sessions · {recipe().autoContinue ? "auto-continue on" : "auto-continue off"}</p>
        </div>
      </Popover>
      <PageToolbar.Root>
        <PageToolbar.Leading>
          <StatusChip tone={honestyTone(store.reminderState())}>{store.reminderDetail()}</StatusChip>
        </PageToolbar.Leading>
      </PageToolbar.Root>
      <PageContent>
        <Show when={store.loadState() === "queued"}><PageLoading label="Loading focus" /></Show>
        <Show when={store.loadState() === "unavailable"}>
          <PageOffline title="Focus unavailable" message={store.loadError() || "The local core is unreachable."} onRetry={() => void store.reload()} />
        </Show>
        <Show when={store.loadState() === "failed"}>
          <PageError title="Focus failed" message={store.loadError()} onRetry={() => void store.reload()} />
        </Show>
        <Show when={store.loadState() === "live"}>
          <div class="mx-auto flex w-full max-w-xl flex-col items-center gap-6 px-4 py-8">
            <Show when={store.interrupted()}>
              <p role="status" class="w-full text-center text-[13px] text-[var(--color-text-secondary)]">{store.interrupted()!.message} {store.interrupted()!.recorded ? "The interrupted phase was recorded as stopped." : "The interrupted phase could not be recorded."}</p>
            </Show>
            <Show when={snap().isQuickSession}><p class="text-[13px] text-[var(--color-text-muted)]">Quick focus</p></Show>
            <Show when={snap().memoryId}><p class="text-[13px] text-[var(--color-text-muted)]">Memory-bound</p></Show>
            <ProgressRing label={`${phaseLabel(snap().phase)} remaining`} value={Math.round(snap().progress * 100)} size={180} thickness={8} />
            <p class="text-[28px] font-semibold tabular-nums text-[var(--color-text-primary)]" role="timer" aria-live="off" aria-label="Time remaining">{formatRemaining(snap().remainingMs)}</p>
            <p class="text-[13px] text-[var(--color-text-secondary)]" role="status" aria-live="polite" aria-atomic="true">{phaseLabel(snap().phase)}{snap().isSessionActive ? snap().isRunning ? " · Running" : snap().isWaitingForManualStart ? " · Ready" : " · Paused" : ""}</p>
            <div class="flex gap-2" role="list" aria-label="Completed focus cycles">
              <For each={cycles()}>{index => (
                <span role="listitem" class={`h-2 w-2 rounded-full ${index < filled() ? "bg-[var(--color-brand)]" : "bg-[var(--color-surface-selected)]"}`} aria-label={index < filled() ? `Cycle ${index + 1} complete` : `Cycle ${index + 1} remaining`} />
              )}</For>
            </div>
            <div class="flex flex-wrap items-center justify-center gap-2">
              <Button variant={primary().kind === "pause" ? "secondary" : "primary"} onClick={() => primary().action()}>{primary().label}</Button>
              <Show when={snap().isSessionActive}>
                <Button variant="secondary" onClick={() => store.skip()}>Skip</Button>
                <Button variant="danger" onClick={() => store.reset()}>Reset</Button>
              </Show>
            </div>
            <Show when={store.recordState() !== "live"}>
              <StatusChip tone={honestyTone(store.recordState())}>{store.recordState() === "queued" ? "Recording session" : store.recordState() === "failed" ? "Session record failed" : "Session record unavailable"}</StatusChip>
            </Show>
            <section class="w-full" aria-label="Today's sessions">
              <h2 class="mb-2 text-[13px] font-medium text-[var(--color-text-secondary)]">Today</h2>
              <Show when={store.sessions().length} fallback={<PageEmpty title="No sessions yet" description="Completed and stopped phases from today appear here and in Review." />}>
                <div class="flex flex-col gap-2">
                  <For each={store.sessions()}>{session => (
                    <Card>
                      <p class="text-[13px] text-[var(--color-text-primary)]">{phaseLabel((session.phase as "work" | "short_break" | "long_break") ?? "idle")} · {session.state === "completed" ? "Completed" : "Stopped"}</p>
                      <p class="text-[12px] text-[var(--color-text-muted)]">{session.elapsedS ?? 0}s of {session.plannedS ?? 0}s{session.endedAt ? ` · ${new Date(session.endedAt).toLocaleTimeString()}` : ""}</p>
                    </Card>
                  )}</For>
                </div>
              </Show>
            </section>
          </div>
        </Show>
      </PageContent>
    </div>
  );
}
