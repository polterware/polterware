import { For, Show, createEffect, createSignal, onCleanup, on, type JSX } from "solid-js";
import { useSparkyShell } from "@polterware/feature-sparky-shell/feature";
import {
  Button, DatePicker, Field, FilterChip, NumberStepper, PageError, PageLoading, SegmentedControl, Select, StatusChip, SwitchRow, TimePicker, parseDate,
} from "@polterware/frontend-design-system";
import { browserBridge, type ScheduleBridge } from "./bridge";
import {
  END_OPTIONS, REPEAT_OPTIONS, WEEKDAYS, defaultFocusRecipe, defaultForm, errorText, focusFromSchedule, focusRecipeInput, formFromSchedule, formatOccurrence,
  reminderTone, reminderWording, timezoneOptions, toScheduleInput, validateFocusRecipe, validateScheduleForm, type FocusRecipeForm, type ReminderWording, type ScheduleForm, type ScheduleJSON,
} from "./mapping";

type Preview = { occurrences: string[]; description: string };
type ReminderStatus = { permission?: string; armed?: number; planned?: number; notifierLastSeenAt?: string };
type SaveState = "live" | "queued" | "failed" | "unavailable";

export function ScheduleCard(): JSX.Element {
  const shell = useSparkyShell();
  const memoryId = () => shell.selectedMemoryId();
  return <Show when={memoryId()}>{id => <ScheduleEditor memoryId={id()} />}</Show>;
}

function ScheduleEditor(props: { memoryId: string }) {
  const [form, setForm] = createSignal<ScheduleForm>(defaultForm());
  const [focus, setFocus] = createSignal<FocusRecipeForm>(defaultFocusRecipe());
  const [preview, setPreview] = createSignal<Preview>({ occurrences: [], description: "" });
  const [completions, setCompletions] = createSignal<string[]>([]);
  const [memoryStatus, setMemoryStatus] = createSignal("active");
  const [saved, setSaved] = createSignal(false);
  const [status, setStatus] = createSignal<ReminderStatus>();
  const [snoozeMinutes, setSnoozeMinutes] = createSignal(10);
  const [saveState, setSaveState] = createSignal<SaveState>("unavailable");
  const [pending, setPending] = createSignal(true);
  const [loadError, setLoadError] = createSignal("");
  const [formError, setFormError] = createSignal("");
  const [coreError, setCoreError] = createSignal("");
  const [previewError, setPreviewError] = createSignal("");
  const [busy, setBusy] = createSignal("");
  const validation = () => validateScheduleForm(form()) || validateFocusRecipe(focus());
  const wording = (): ReminderWording => reminderWording({
    ...status(),
    memoryStatus: memoryStatus(),
    nextOccurrenceCompleted: completed(preview().occurrences[0], completions()),
  });
  function patch(partial: Partial<ScheduleForm>) {
    setForm(current => ({ ...current, ...partial }));
    setSaveState("queued");
    setFormError("");
  }
  function patchFocus(partial: Partial<FocusRecipeForm>) {
    setFocus(current => ({ ...current, ...partial }));
    setSaveState("queued");
  }
  async function withBridge<T>(label: string, run: (bridge: ScheduleBridge) => Promise<T>): Promise<T | undefined> {
    try {
      setBusy(label);
      const result = await run(browserBridge());
      setCoreError("");
      return result;
    } catch (error) {
      setCoreError(errorText(error));
      setSaveState("failed");
      return undefined;
    } finally {
      setBusy("");
    }
  }
  async function refreshStatus(bridge: ScheduleBridge) {
    const next = await bridge.reminders.status({}) as ReminderStatus;
    setStatus(next);
    return next;
  }
  async function load(id: string) {
    setPending(true); setLoadError("");
    try {
      const bridge = browserBridge();
      const [memoryResponse, settingsResponse] = await Promise.all([
        bridge.memories.get({ id, include: { schedule: true, completions: true } }),
        bridge.settings.get({}).catch(() => ({ settings: { notificationsSnoozeMinutes: 10 } })),
      ]);
      const memory = (memoryResponse as { memory?: { status?: string; schedule?: ScheduleJSON; completions?: { occurrenceAt?: string }[] } }).memory;
      if (!memory) throw new Error("Memory unavailable. It may have been deleted.");
      const schedule = memory.schedule;
      setForm(formFromSchedule(schedule));
      setFocus(focusFromSchedule(schedule));
      setMemoryStatus(memory.status ?? "active");
      setCompletions((memory.completions ?? []).map(item => item.occurrenceAt ?? "").filter(Boolean));
      setSaved(Boolean(schedule?.fireAt));
      const settings = (settingsResponse as { settings?: { notificationsSnoozeMinutes?: number } }).settings;
      if (settings?.notificationsSnoozeMinutes) setSnoozeMinutes(settings.notificationsSnoozeMinutes);
      await refreshStatus(bridge);
      setSaveState("live");
    } catch (error) {
      setLoadError(errorText(error));
      setSaveState("unavailable");
    } finally {
      setPending(false);
    }
  }
  async function refreshPreview(current: ScheduleForm, id: string) {
    const issue = validateScheduleForm(current);
    if (issue) { setFormError(issue); setPreview({ occurrences: [], description: "" }); return; }
    setFormError("");
    try {
      const result = await browserBridge().schedule.preview({ schedule: toScheduleInput(current, id), limit: 5 }) as Preview;
      setPreview({ occurrences: result.occurrences ?? [], description: result.description ?? "" });
      setPreviewError("");
    } catch (error) {
      setPreview({ occurrences: [], description: "" });
      setPreviewError(errorText(error));
    }
  }
  createEffect(on(() => props.memoryId, id => {
    let active = true;
    void load(id).then(() => { if (active) void refreshPreview(form(), id); });
    let unsubscribe: (() => void) | undefined;
    try {
      unsubscribe = browserBridge().onChange(event => {
        if (saveState() === "queued") return;
        if (event.resync || event.changes.some(change => ["schedule", "memory", "memories", "reminders"].includes(change.entity))) {
          void load(id);
        }
      });
    } catch { /* The editor already reports an unavailable bridge. */ }
    onCleanup(() => { active = false; unsubscribe?.(); });
  }));
  createEffect(on(() => JSON.stringify(form()), () => {
    const id = props.memoryId;
    const current = form();
    const timer = setTimeout(() => { void refreshPreview(current, id); }, 250);
    onCleanup(() => clearTimeout(timer));
  }));
  async function save() {
    const issue = validation();
    if (issue) { setFormError(issue); return; }
    const id = props.memoryId;
    const payload = toScheduleInput(form(), id);
    const recipe = focus();
    await withBridge("save", async bridge => {
      await bridge.schedule.set({ schedule: payload });
      if (recipe.enabled || saved()) await bridge.schedule.setFocusRecipe(focusRecipeInput(recipe, id));
      setSaved(true);
      setSaveState("live");
      await refreshPreview(form(), id);
      const started = Date.now();
      while (Date.now() - started < 2000) {
        await refreshStatus(bridge);
        if ((status()?.armed ?? 0) > 0 || Date.now() - started > 1500) break;
        await new Promise(done => setTimeout(done, 250));
      }
    });
  }
  async function clear() {
    await withBridge("clear", async bridge => {
      await bridge.schedule.clear({ memoryId: props.memoryId });
      setSaved(false);
      setForm(defaultForm());
      setFocus(defaultFocusRecipe());
      setPreview({ occurrences: [], description: "" });
      setSaveState("live");
      await refreshStatus(bridge);
    });
  }
  async function complete(occurrenceAt: string, reopen = false) {
    await withBridge(reopen ? "reopen" : "complete", async bridge => {
      if (reopen) await bridge.schedule.reopenOccurrence({ memoryId: props.memoryId, occurrenceAt });
      else await bridge.schedule.completeOccurrence({ memoryId: props.memoryId, occurrenceAt });
      await load(props.memoryId);
      await refreshPreview(form(), props.memoryId);
    });
  }
  async function snooze() {
    const occurrenceAt = preview().occurrences.find(item => !completed(item, completions()));
    if (!occurrenceAt) { setCoreError("No upcoming occurrence to snooze."); return; }
    await withBridge("snooze", async bridge => {
      await bridge.reminders.snooze({ memoryId: props.memoryId, occurrenceAt, minutes: snoozeMinutes() });
      await refreshStatus(bridge);
    });
  }
  const keyboard = (event: KeyboardEvent) => {
    if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "s") {
      event.preventDefault();
      void save();
    }
  };
  return <div data-schedule-card class="flex flex-col gap-4" onKeyDown={keyboard}>
    <div class="flex flex-wrap items-center gap-2">
      <h2 class="text-sm font-medium text-[var(--color-text-primary)]">Schedule</h2>
      <StatusChip tone={saveState() === "live" ? "ok" : saveState() === "queued" ? "syncing" : saveState() === "failed" ? "danger" : "muted"}>
        {saveState() === "live" ? "Live" : saveState() === "queued" ? "Queued" : saveState() === "failed" ? "Failed" : "Unavailable"}
      </StatusChip>
      <span data-reminder-state={wording()}><StatusChip tone={reminderTone(wording())}>{wording()}</StatusChip></span>
    </div>
    <Show when={pending()}><PageLoading label="Loading schedule" /></Show>
    <Show when={loadError()}><PageError title="Schedule unavailable" message={loadError()} onRetry={() => void load(props.memoryId)} /></Show>
    <Show when={!pending() && !loadError()}>
      <DatePicker label="Start date" value={form().date ? [parseDate(form().date)] : []} onValueChange={details => {
        const next = details.value[0]?.toString() ?? "";
        patch({ date: next });
      }} />
      <SwitchRow label="All day" aria-label="All day" checked={form().allDay} onCheckedChange={value => patch({ allDay: value })} />
      <Show when={!form().allDay}><TimePicker label="Time" value={form().time} onValueChange={time => patch({ time })} /></Show>
      <Field label="Time zone" for="schedule-tz">
        <Select id="schedule-tz" aria-label="Time zone" value={form().tz} options={timezoneOptions(form().tz)} onChange={tz => patch({ tz })} />
      </Field>
      <Field label="Repeat">
        <Select aria-label="Repeat" value={form().freq} options={REPEAT_OPTIONS.map(option => ({ value: option.value, label: option.label }))} onChange={freq => patch({ freq: freq as ScheduleForm["freq"], weekdays: freq === "weekly" ? form().weekdays : WEEKDAYS.map(() => false) })} />
      </Field>
      <Show when={form().freq}>
        <NumberStepper label="Every" min={1} value={String(form().every)} onValueChange={details => patch({ every: Number.isFinite(details.valueAsNumber) ? Math.max(1, Math.trunc(details.valueAsNumber)) : 1 })} />
      </Show>
      <Show when={form().freq === "weekly"}>
        <div role="group" aria-label="Weekdays" class="flex flex-wrap gap-2">
          <For each={WEEKDAYS}>{day =>
            <FilterChip selected={form().weekdays[day.bit]} aria-label={day.label} onClick={() => {
              const weekdays = [...form().weekdays];
              weekdays[day.bit] = !weekdays[day.bit];
              patch({ weekdays });
            }}>{day.label}</FilterChip>
          }</For>
        </div>
      </Show>
      <Show when={form().freq}>
        <Field label="Ends">
          <SegmentedControl aria-label="Ends" value={form().endType} options={END_OPTIONS} onValueChange={details => patch({ endType: (details.value as ScheduleForm["endType"]) || "never" })} />
        </Field>
      </Show>
      <Show when={form().freq && form().endType === "until"}>
        <DatePicker label="Until date" value={form().untilDate ? [parseDate(form().untilDate)] : []} onValueChange={details => patch({ untilDate: details.value[0]?.toString() ?? "" })} />
        <Show when={!form().allDay}><TimePicker label="Until time" value={form().untilTime} onValueChange={untilTime => patch({ untilTime })} /></Show>
      </Show>
      <Show when={form().freq && form().endType === "count"}>
        <NumberStepper label="Occurrences" min={1} value={String(form().occurrenceCount)} onValueChange={details => patch({ occurrenceCount: Number.isFinite(details.valueAsNumber) ? Math.max(1, Math.trunc(details.valueAsNumber)) : 1 })} />
      </Show>
      <Show when={formError()}><p role="alert">{formError()}</p></Show>
      <Show when={previewError()}><p role="status">Preview failed: {previewError()}</p></Show>
      <Show when={preview().description}><p data-schedule-summary>{preview().description}</p></Show>
      <div data-schedule-preview class="flex flex-col gap-2">
        <p class="text-sm text-[var(--color-text-muted)]">Next 5 occurrences</p>
        <For each={preview().occurrences} fallback={<p>No upcoming occurrences.</p>}>{occurrence =>
          <div class="flex flex-wrap items-center gap-2" data-occurrence={occurrence}>
            <span>{formatOccurrence(occurrence, form().tz, form().allDay)}</span>
            <Show when={form().freq}>
              <Show when={completed(occurrence, completions())} fallback={<Button variant="secondary" onClick={() => void complete(occurrence)}>Complete occurrence</Button>}>
                <Button variant="secondary" onClick={() => void complete(occurrence, true)}>Reopen occurrence</Button>
              </Show>
            </Show>
          </div>
        }</For>
      </div>
      <SwitchRow label="Focus recipe" aria-label="Focus recipe" checked={focus().enabled} onCheckedChange={enabled => patchFocus({ enabled })} />
      <Show when={focus().enabled}>
        <NumberStepper label="Work" min={1} max={60} value={String(focus().workMin)} onValueChange={details => patchFocus({ workMin: clamp(details.valueAsNumber, 1, 60) })} />
        <NumberStepper label="Short break" min={1} max={60} value={String(focus().shortBreakMin)} onValueChange={details => patchFocus({ shortBreakMin: clamp(details.valueAsNumber, 1, 60) })} />
        <NumberStepper label="Long break" min={1} max={60} value={String(focus().longBreakMin)} onValueChange={details => patchFocus({ longBreakMin: clamp(details.valueAsNumber, 1, 60) })} />
        <NumberStepper label="Cycles" min={1} max={12} value={String(focus().cycles)} onValueChange={details => patchFocus({ cycles: clamp(details.valueAsNumber, 1, 12) })} />
        <SwitchRow label="Auto-continue" checked={focus().autoContinue} onCheckedChange={autoContinue => patchFocus({ autoContinue })} />
      </Show>
      <Show when={coreError()}><p role="alert">{coreError()}</p></Show>
      <div class="flex flex-wrap gap-2">
        <Button disabled={Boolean(busy()) || Boolean(validation())} onClick={() => void save()}>Save schedule</Button>
        <Show when={saved()}><Button variant="secondary" disabled={Boolean(busy())} onClick={() => void clear()}>Clear schedule</Button></Show>
        <Button variant="secondary" disabled={Boolean(busy()) || !preview().occurrences.length} onClick={() => void snooze()}>Snooze</Button>
      </div>
    </Show>
  </div>;
}

function completed(occurrence: string | undefined, completions: readonly string[]): boolean {
  if (!occurrence) return false;
  const stamp = Date.parse(occurrence);
  return completions.some(item => Date.parse(item) === stamp);
}

function clamp(value: number, min: number, max: number): number {
  if (!Number.isFinite(value)) return min;
  return Math.min(max, Math.max(min, Math.trunc(value)));
}
