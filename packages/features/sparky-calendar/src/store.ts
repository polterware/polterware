import { batch, createSignal } from "solid-js";
import { browserBridge, type CalendarBridge, type OccurrenceJSON } from "./bridge";
import { occurrenceDate, paddedVisibleRange, systemTimeZone, visibleRange, weekStartFromSettings, type CalendarView, type WeekStart } from "./grid";
import { errorText, isRecurring, moveSeriesAnchor, slotInstant, toOneShotSchedule, type ScheduleJSON, type SlotDraft } from "./mapping";

export type SaveState = "live" | "queued" | "failed" | "unavailable";

export type CalendarOccurrence = {
  memoryId: string;
  title: string;
  occurrenceAt: string;
  fireAt: string;
  completed: boolean;
  overdue: boolean;
  allDay: boolean;
  mindId: string;
  tz: string;
  color: string;
};

export type RangeQuery = { from: string; to: string };

function asRecord(value: unknown): Record<string, unknown> {
  return value && typeof value === "object" && !Array.isArray(value) ? value as Record<string, unknown> : {};
}

function occurrenceFromJson(row: OccurrenceJSON, minds: Record<string, string>, fallbackTz: string): CalendarOccurrence | undefined {
  const memoryId = String(row.memoryId ?? "");
  const occurrenceAt = String(row.occurrenceAt ?? "");
  if (!memoryId || !occurrenceAt) return undefined;
  const mindId = String(row.mindId ?? "");
  return {
    memoryId,
    title: String(row.title ?? "Untitled memory"),
    occurrenceAt,
    fireAt: row.fireAt ?? occurrenceAt,
    completed: Boolean(row.completed),
    overdue: Boolean(row.overdue),
    allDay: Boolean(row.allDay),
    mindId,
    tz: String(row.tz || fallbackTz),
    color: minds[mindId] ?? "",
  };
}

export function createCalendarStore(getBridge: () => CalendarBridge = browserBridge) {
  const [occurrences, setOccurrences] = createSignal<CalendarOccurrence[]>([]);
  const [range, setRange] = createSignal<RangeQuery>({ from: "", to: "" });
  const [weekStart, setWeekStart] = createSignal<WeekStart>("mon");
  const [displayTz, setDisplayTz] = createSignal(systemTimeZone());
  const [saveState, setSaveState] = createSignal<SaveState>("unavailable");
  const [loadError, setLoadError] = createSignal("");
  const [actionError, setActionError] = createSignal("");
  const [pending, setPending] = createSignal(false);
  const [busy, setBusy] = createSignal("");
  let lastQuery: RangeQuery | undefined;
  let unsubscribe: (() => void) | undefined;
  let users = 0;

  async function loadSettings() {
    try {
      const { settings } = await getBridge().settings.get({});
      setWeekStart(weekStartFromSettings(settings));
    } catch {
      setWeekStart("mon");
    }
  }

  async function loadRange(query: RangeQuery) {
    lastQuery = query;
    setPending(true);
    setLoadError("");
    try {
      const bridge = getBridge();
      const [list, mindsResponse] = await Promise.all([
        bridge.schedule.listOccurrences({ from: query.from, to: query.to, includeCompleted: true }),
        bridge.minds.list({}).catch(() => ({ minds: [] })),
      ]);
      const colors: Record<string, string> = {};
      for (const mind of (asRecord(mindsResponse).minds as Record<string, unknown>[] | undefined) ?? []) {
        const id = String(mind.id ?? "");
        if (id) colors[id] = String(mind.color ?? "");
      }
      const rows = (list.occurrences ?? [])
        .map(row => occurrenceFromJson(row, colors, displayTz()))
        .filter((row): row is CalendarOccurrence => Boolean(row));
      batch(() => {
        setOccurrences(rows);
        setRange(query);
        setSaveState("live");
        setActionError("");
      });
    } catch (error) {
      setLoadError(errorText(error));
      setSaveState("unavailable");
    } finally {
      setPending(false);
    }
  }

  async function refresh() {
    if (lastQuery) await loadRange(lastQuery);
  }

  function connect() {
    users += 1;
    if (users === 1) {
      void loadSettings();
      try {
        unsubscribe = getBridge().onChange(event => {
          if (event.resync || event.changes.some(change => ["schedule", "memory", "memories", "minds", "settings"].includes(change.entity))) {
            void refresh();
            if (event.changes.some(change => change.entity === "settings")) void loadSettings();
          }
        });
      } catch {
        setSaveState("unavailable");
        setLoadError("Sparky bridge unavailable. Open this page in Sparky Desktop.");
      }
    }
    return () => {
      users -= 1;
      if (users <= 0) {
        users = 0;
        unsubscribe?.();
        unsubscribe = undefined;
      }
    };
  }

  async function ensureVisible(anchor: string, view: CalendarView, timeZone = displayTz()) {
    const visible = visibleRange(anchor, view, weekStart(), timeZone);
    const padded = paddedVisibleRange(anchor, view, weekStart(), timeZone);
    if (range().from === padded.from && range().to === padded.to) return visible;
    await loadRange(padded);
    return visible;
  }

  async function createFromSlot(draft: SlotDraft, onCreated?: (id: string) => void) {
    setBusy("create");
    setSaveState("queued");
    try {
      const bridge = getBridge();
      const created = asRecord(await bridge.memories.create({ title: "Untitled memory" }));
      const memory = asRecord(created.memory);
      const id = String(memory.id ?? "");
      if (!id) throw new Error("The core did not create a memory.");
      // The memory already exists; keep it reachable if the schedule write fails.
      onCreated?.(id);
      const schedule = toOneShotSchedule({ ...draft, tz: draft.tz || displayTz() }, id);
      await bridge.schedule.set({ schedule });
      setSaveState("live");
      setActionError("");
      await refresh();
      return id;
    } catch (error) {
      setSaveState("failed");
      setActionError(errorText(error));
      throw error;
    } finally {
      setBusy("");
    }
  }

  async function reschedule(occurrence: CalendarOccurrence, draft: SlotDraft, confirmSeries: boolean) {
    setBusy("reschedule");
    setSaveState("queued");
    try {
      const bridge = getBridge();
      const response = asRecord(await bridge.memories.get({ id: occurrence.memoryId, include: { schedule: true } }));
      const memory = asRecord(response.memory);
      const schedule = asRecord(memory.schedule) as ScheduleJSON;
      if (!schedule.fireAt) throw new Error("This memory has no schedule to move.");
      if (isRecurring(schedule) && !confirmSeries) {
        setSaveState("live");
        return { needsSeriesConfirm: true as const };
      }
      const dropAt = slotInstant({ ...draft, tz: occurrence.tz || schedule.tz || displayTz() });
      const next = moveSeriesAnchor({ ...schedule, memoryId: occurrence.memoryId }, occurrence.occurrenceAt, dropAt);
      if (draft.allDay) next.allDay = true;
      await bridge.schedule.set({ schedule: next });
      setSaveState("live");
      setActionError("");
      await refresh();
      return { needsSeriesConfirm: false as const };
    } catch (error) {
      setSaveState("failed");
      setActionError(errorText(error));
      throw error;
    } finally {
      setBusy("");
    }
  }

  const forDate = (date: string) => occurrences().filter(item => occurrenceDate(item.occurrenceAt, item.tz) === date);
  const timedOn = (date: string) => forDate(date).filter(item => !item.allDay);
  const allDayOn = (date: string) => forDate(date).filter(item => item.allDay);

  return {
    occurrences, range, weekStart, displayTz, saveState, loadError, actionError, pending, busy,
    connect, loadRange, ensureVisible, refresh, loadSettings, createFromSlot, reschedule, forDate, timedOn, allDayOn,
    setDisplayTz,
  };
}

export type CalendarStore = ReturnType<typeof createCalendarStore>;
export const calendarStore = createCalendarStore();
