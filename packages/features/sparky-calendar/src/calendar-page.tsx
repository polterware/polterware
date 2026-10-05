import { For, Show, createEffect, createMemo, createSignal, onCleanup, onMount } from "solid-js";
import { useSparkyShell } from "@polterware/feature-sparky-shell/feature";
import {
  A11yDialog, Button, CalendarGrid, ChevronLeft, ChevronRight, DatePicker, PageContent, PageError, PageHeader,
  PageLoading, PageToolbar, parseDate, Plus, Popover, SegmentedControl, StatusChip, ToolbarIconButton, prefersReducedMotion,
} from "@polterware/frontend-design-system";
import {
  civilFromInstant, dayHours, formatYmd, hourLabel, monthDates, occurrenceDate, pad, rangeTitle, sameMonth, shiftedAnchor,
  systemTimeZone, visibleRange, weekDates, weekdayLabels, type CalendarView,
} from "./grid";
import type { SlotDraft } from "./mapping";
import { calendarStore, type CalendarOccurrence, type CalendarStore } from "./store";
import "./calendar.css";

const VIEWS: { value: CalendarView; label: string }[] = [
  { value: "day", label: "Day" },
  { value: "week", label: "Week" },
  { value: "month", label: "Month" },
];
const MONTH_VISIBLE = 1;
const ALL_DAY_VISIBLE = 3;

function todayYmd(timeZone = systemTimeZone()): string {
  const wall = civilFromInstant(new Date().toISOString(), timeZone);
  return formatYmd(wall);
}

function dateLabel(date: string): string {
  return new Intl.DateTimeFormat("en-US", { dateStyle: "full", timeZone: "UTC" }).format(new Date(`${date}T12:00:00Z`));
}

function tone(state: string): "ok" | "syncing" | "danger" | "muted" {
  if (state === "live") return "ok";
  if (state === "queued") return "syncing";
  if (state === "failed") return "danger";
  return "muted";
}

function stateLabel(state: string): string {
  if (state === "live") return "Live";
  if (state === "queued") return "Queued";
  if (state === "failed") return "Failed";
  return "Unavailable";
}

export function CalendarPage(props: { store?: CalendarStore }) {
  const store = props.store ?? calendarStore;
  const shell = useSparkyShell();
  const [view, setView] = createSignal<CalendarView>("week");
  const [anchor, setAnchor] = createSignal(todayYmd());
  const [focused, setFocused] = createSignal(todayYmd());
  const [focusPosition, setFocusPosition] = createSignal({ date: todayYmd(), index: 0 });
  let focusAfterRender = false;
  const [shift, setShift] = createSignal(false);
  const [overflow, setOverflow] = createSignal<{ date: string; items: CalendarOccurrence[] }>();
  const [pendingMove, setPendingMove] = createSignal<{ occurrence: CalendarOccurrence; draft: SlotDraft }>();
  const [now, setNow] = createSignal(new Date());
  let root: HTMLDivElement | undefined;
  const title = createMemo(() => rangeTitle(anchor(), view(), store.weekStart()));
  const dates = createMemo(() => view() === "day" ? [anchor()] : view() === "week" ? weekDates(anchor(), store.weekStart()) : monthDates(anchor(), store.weekStart()));
  const hours = createMemo(() => view() === "day" ? dayHours(anchor(), store.displayTz()) : Array.from({ length: 24 }, (_, hour) => ({ hour, fold: 0, label: hourLabel(hour), startUtc: "" })));

  onMount(() => {
    const disconnect = store.connect();
    const timer = setInterval(() => setNow(new Date()), 60_000);
    void store.loadSettings().then(() => store.ensureVisible(anchor(), view(), store.displayTz()));
    onCleanup(() => { disconnect(); clearInterval(timer); });
  });
  createEffect(() => {
    const next = visibleRange(anchor(), view(), store.weekStart(), store.displayTz());
    void store.ensureVisible(anchor(), view(), store.displayTz());
    root?.setAttribute("data-calendar-from", next.from);
    root?.setAttribute("data-calendar-to", next.to);
  });

  const gridButtons = () => [...(root?.querySelectorAll<HTMLButtonElement>(".sparky-calendar__grid-wrap button:not(:disabled)") ?? [])];
  const dayButtons = (date: string) => gridButtons().filter(button => button.closest<HTMLElement>("[data-day]")?.dataset.day === date);
  function syncFocus() {
    const position = focusPosition();
    const buttons = gridButtons();
    const day = dayButtons(position.date);
    const active = day[Math.min(position.index, day.length - 1)] ?? buttons[0];
    for (const button of buttons) button.tabIndex = button === active ? 0 : -1;
    if (focusAfterRender && active) {
      focusAfterRender = false;
      active.focus();
    }
  }
  createEffect(() => {
    dates(); view(); store.occurrences(); store.pending(); focusPosition();
    queueMicrotask(syncFocus);
  });
  function focusGrid(event: FocusEvent) {
    const button = event.target;
    if (!(button instanceof HTMLButtonElement)) return;
    const date = button.closest<HTMLElement>("[data-day]")?.dataset.day;
    if (!date) return;
    setFocused(date);
    setFocusPosition({ date, index: dayButtons(date).indexOf(button) });
    syncFocus();
  }
  function bump() {
    if (prefersReducedMotion()) return;
    setShift(true);
    requestAnimationFrame(() => setShift(false));
  }
  function go(next: string, nextView = view()) {
    setAnchor(next);
    setFocused(next);
    setFocusPosition({ date: next, index: 0 });
    setView(nextView);
    bump();
  }
  async function create(draft: SlotDraft) {
    try {
      await store.createFromSlot(draft, id => shell.openMemory(id));
    } catch {
      /* The status chip already reports failed. */
    }
  }
  async function drop(occurrence: CalendarOccurrence, draft: SlotDraft) {
    try {
      const result = await store.reschedule(occurrence, draft, false);
      if (result.needsSeriesConfirm) setPendingMove({ occurrence, draft });
    } catch {
      /* The status chip already reports failed. */
    }
  }
  async function confirmSeries() {
    const pending = pendingMove();
    if (!pending) return;
    try {
      await store.reschedule(pending.occurrence, pending.draft, true);
    } catch {
      /* Keep the dialog so the user can cancel after a failure. */
      return;
    }
    setPendingMove(undefined);
  }
  function keyboard(event: KeyboardEvent) {
    if (event.defaultPrevented || event.isComposing || event.metaKey || event.ctrlKey || event.altKey) return;
    const target = event.target;
    if (!(target instanceof HTMLElement)) return;
    const inGrid = target.closest(".sparky-calendar__grid-wrap");
    if (!inGrid && target !== root) return;
    const key = event.key;
    if (["ArrowLeft", "ArrowRight", "ArrowUp", "ArrowDown", "Home", "End"].includes(key)) {
      if (!inGrid || !(target instanceof HTMLButtonElement)) return;
      const date = target.closest<HTMLElement>("[data-day]")?.dataset.day;
      if (!date) return;
      event.preventDefault();
      const day = dayButtons(date);
      const index = day.indexOf(target);
      let nextDate = date;
      let nextIndex = index;
      if (key === "Home" || key === "End") nextIndex = key === "Home" ? 0 : day.length - 1;
      else if (view() === "month" && (key === "ArrowUp" || key === "ArrowDown")) {
        nextDate = shiftedAnchor(date, "day", key === "ArrowDown" ? 7 : -7);
      } else if (view() !== "month" && (key === "ArrowLeft" || key === "ArrowRight")) {
        nextDate = shiftedAnchor(date, "day", key === "ArrowRight" ? 1 : -1);
        const time = target.closest<HTMLElement>("[data-slot-time]")?.dataset.slotTime;
        const destination = dayButtons(nextDate);
        const sameTime = time ? destination.findIndex(button => button.closest<HTMLElement>("[data-slot-time]")?.dataset.slotTime === time) : -1;
        if (sameTime >= 0) nextIndex = sameTime;
      } else {
        nextIndex += key === "ArrowDown" || key === "ArrowRight" ? 1 : -1;
        if (nextIndex < 0 || nextIndex >= day.length) {
          nextDate = shiftedAnchor(date, "day", nextIndex < 0 ? -1 : 1);
          nextIndex = nextIndex < 0 ? Math.max(0, dayButtons(nextDate).length - 1) : 0;
        }
      }
      if (!dates().includes(nextDate)) setAnchor(nextDate);
      setFocused(nextDate);
      focusAfterRender = true;
      setFocusPosition({ date: nextDate, index: nextIndex });
      queueMicrotask(syncFocus);
    } else if (key === "t" || key === "d" || key === "w" || key === "m") {
      event.preventDefault();
      focusAfterRender = Boolean(inGrid);
      go(key === "t" ? todayYmd(store.displayTz()) : focused(), key === "d" ? "day" : key === "w" ? "week" : key === "m" ? "month" : view());
    }
  }

  return <div ref={root} class="sparky-calendar" data-calendar data-calendar-view={view()} tabindex="-1" onKeyDown={keyboard}>
    <PageHeader
      title={`${title().leading} ${title().trailing}`.trim()}
      subtitle={<span data-save-state={store.saveState()}><StatusChip tone={tone(store.saveState())}>{stateLabel(store.saveState())}</StatusChip></span>}
    />
    <PageToolbar.Root>
      <PageToolbar.Leading>
        <SegmentedControl aria-label="Calendar view" value={view()} options={VIEWS} onValueChange={details => { if (details.value) go(anchor(), details.value as CalendarView); }} />
      </PageToolbar.Leading>
      <PageToolbar.Actions>
        <ToolbarIconButton label="Previous" onClick={() => go(shiftedAnchor(anchor(), view(), -1))}><ChevronLeft size={14} /></ToolbarIconButton>
        <Button variant="secondary" onClick={() => go(todayYmd(store.displayTz()))}>Today</Button>
        <ToolbarIconButton label="Next" onClick={() => go(shiftedAnchor(anchor(), view(), 1))}><ChevronRight size={14} /></ToolbarIconButton>
        <DatePicker label="Jump to date" value={anchor() ? [parseDate(anchor())] : []} onValueChange={details => {
          const next = details.value[0]?.toString();
          if (next) go(next);
        }} />
      </PageToolbar.Actions>
    </PageToolbar.Root>
    <PageContent>
      <Show when={store.pending() && !store.occurrences().length}><PageLoading label="Loading calendar" /></Show>
      <Show when={store.loadError()}><PageError title="Calendar unavailable" message={store.loadError()} onRetry={() => void store.refresh()} /></Show>
      <Show when={store.actionError()}><p role="alert">{store.actionError()}</p></Show>
      <Show when={!store.loadError()}>
        <div class="sparky-calendar__grid-wrap" onFocusIn={focusGrid} data-shift={shift() ? "" : undefined}>
          <CalendarGrid.Root view={view()} label={`${title().leading} ${title().trailing}`.trim()}>
            <Show when={view() !== "day"}>
              <CalendarGrid.Days>
                <For each={weekdayLabels(store.weekStart())}>{label => <CalendarGrid.Header>{label}</CalendarGrid.Header>}</For>
              </CalendarGrid.Days>
            </Show>
            <CalendarGrid.Days>
              <For each={dates()}>{date =>
                <DayColumn
                  date={date}
                  view={view()}
                  today={todayYmd(store.displayTz())}
                  focused={focused() === date}
                  inMonth={sameMonth(date, anchor())}
                  hours={hours()}
                  now={now()}
                  displayTz={store.displayTz()}
                  allDay={store.allDayOn(date)}
                  timed={store.timedOn(date)}
                  onOpen={id => shell.openMemory(id)}
                  onCreate={create}
                  onDrop={drop}
                  onOpenDay={() => go(date, "day")}
                  onOverflow={items => setOverflow({ date, items })}
                />
              }</For>
            </CalendarGrid.Days>
          </CalendarGrid.Root>
        </div>
      </Show>
    </PageContent>
    <Popover
      open={Boolean(overflow())}
      onClose={() => setOverflow(undefined)}
      title="More occurrences"
      anchorRef={() => root?.querySelector(`[data-day="${overflow()?.date ?? ""}"] [aria-label$="more occurrences"]`) as HTMLElement | undefined}
      width={280}
    >
      <div class="sparky-calendar__overflow-list">
        <For each={overflow()?.items ?? []}>{item =>
          <OccurrencePill occurrence={item} onOpen={() => { setOverflow(undefined); shell.openMemory(item.memoryId); }} />
        }</For>
      </div>
    </Popover>
    <A11yDialog
      open={Boolean(pendingMove())}
      title="Move the whole series?"
      size="sm"
      onClose={() => setPendingMove(undefined)}
      footer={<><Button variant="secondary" onClick={() => setPendingMove(undefined)}>Cancel</Button><Button onClick={() => void confirmSeries()}>Move the series</Button></>}
    >
      <p>This occurrence only is not supported in v1. Moving this pill changes the series anchor for every occurrence.</p>
    </A11yDialog>
  </div>;
}

function DayColumn(props: {
  date: string;
  view: CalendarView;
  today: string;
  focused: boolean;
  inMonth: boolean;
  hours: { hour: number; fold: number; label: string; startUtc: string }[];
  now: Date;
  displayTz: string;
  allDay: CalendarOccurrence[];
  timed: CalendarOccurrence[];
  onOpen: (id: string) => void;
  onCreate: (draft: SlotDraft) => void;
  onDrop: (occurrence: CalendarOccurrence, draft: SlotDraft) => void;
  onOpenDay: () => void;
  onOverflow: (items: CalendarOccurrence[]) => void;
}) {
  const visibleLimit = () => props.view === "month" ? MONTH_VISIBLE : ALL_DAY_VISIBLE;
  const visible = () => props.view === "month" ? [...props.allDay, ...props.timed].slice(0, visibleLimit()) : props.allDay.slice(0, visibleLimit());
  const extra = () => {
    const all = props.view === "month" ? [...props.allDay, ...props.timed] : props.allDay;
    return all.slice(visible().length);
  };
  const nowParts = () => civilFromInstant(props.now.toISOString(), props.displayTz);
  return <CalendarGrid.Day label={dateLabel(props.date)} today={props.date === props.today}>
    <div
      class="sparky-calendar__day"
      data-day={props.date}
      data-focused={props.focused ? "" : undefined}
      data-outside={props.view === "month" && !props.inMonth ? "" : undefined}
    >
      <CalendarGrid.Header>
        <Button variant="ghost" aria-label={`Open ${dateLabel(props.date)} in day view`} onClick={props.onOpenDay}>{props.date.slice(8)}</Button>
      </CalendarGrid.Header>
      <CalendarGrid.AllDay label="All day">
        <For each={visible()}>{item =>
          <OccurrencePill occurrence={item} draggable={props.view !== "month"} onOpen={() => props.onOpen(item.memoryId)} onDrop={draft => props.onDrop(item, draft)} />
        }</For>
        <Show when={extra().length}>
          <CalendarGrid.Overflow count={extra().length} onClick={() => props.onOverflow(extra())} />
        </Show>
        <Show when={!props.allDay.length || props.view !== "month"}>
          <Button
            variant="ghost"
            size="sm"
            data-empty-slot
            data-detail-opener
            data-slot-date={props.date}
            data-slot-all-day="true"
            aria-label={`New all-day memory on ${dateLabel(props.date)}`}
            onClick={() => props.onCreate({ date: props.date, time: "00:00", allDay: true })}
          >
            Add memory
          </Button>
        </Show>
      </CalendarGrid.AllDay>
      <Show when={props.view !== "month"}>
        <div class="sparky-calendar__slots" style={{ position: "relative" }}>
          <For each={props.hours}>{slot => {
            const items = () => props.timed.filter(item => civilFromInstant(item.occurrenceAt, item.tz).hour === slot.hour);
            return <CalendarGrid.HourRow label={slot.label}>
              <div
                data-slot-date={props.date}
                data-slot-time={slot.label}
                onDragOver={event => event.preventDefault()}
                onDrop={event => {
                  const payload = event.dataTransfer?.getData("application/x-sparky-occurrence");
                  if (!payload) return;
                  const occurrence = JSON.parse(payload) as CalendarOccurrence;
                  props.onDrop(occurrence, { date: props.date, time: slot.label, allDay: false, tz: occurrence.tz });
                }}
              >
                <For each={items()}>{item =>
                  <OccurrencePill occurrence={item} draggable onOpen={() => props.onOpen(item.memoryId)} onDrop={draft => props.onDrop(item, draft)} />
                }</For>
                <Show when={!items().length}>
                  <Button
                    variant="ghost"
                    size="sm"
                    data-empty-slot
                    data-detail-opener
                    data-slot-date={props.date}
                    data-slot-time={slot.label}
                    aria-label={`Add at ${slot.label} on ${dateLabel(props.date)}`}
                    onClick={() => props.onCreate({ date: props.date, time: slot.label, allDay: false })}
                  >
                    <Plus size={12} aria-hidden="true" />
                  </Button>
                </Show>
              </div>
            </CalendarGrid.HourRow>;
          }}</For>
          <Show when={props.date === props.today && nowParts().year}>
            <div class="sparky-calendar__now" data-now-line style={{ top: `${(nowParts().hour + nowParts().minute / 60) * 56}px` }} />
          </Show>
        </div>
      </Show>
    </div>
  </CalendarGrid.Day>;
}

function OccurrencePill(props: {
  occurrence: CalendarOccurrence;
  draggable?: boolean;
  onOpen: () => void;
  onDrop?: (draft: SlotDraft) => void;
}) {
  const label = () => {
    if (props.occurrence.allDay) return props.occurrence.title;
    const wall = civilFromInstant(props.occurrence.occurrenceAt, props.occurrence.tz);
    return `${pad(wall.hour)}:${pad(wall.minute)} ${props.occurrence.title}`;
  };
  return <CalendarGrid.Occurrence
    class="sparky-calendar__pill"
    style={{ "--occurrence-accent": props.occurrence.color || "var(--color-brand)" } as Record<string, string>}
    data-occurrence={props.occurrence.memoryId}
    data-detail-opener
    data-occurrence-at={props.occurrence.occurrenceAt}
    completed={props.occurrence.completed}
    overdue={props.occurrence.overdue}
    draggable={props.draggable}
    onDragStart={event => {
      event.dataTransfer?.setData("application/x-sparky-occurrence", JSON.stringify(props.occurrence));
    }}
    onClick={props.onOpen}
    aria-label={`${label()}${props.occurrence.allDay ? " · All day" : ""}${props.occurrence.completed ? " · Completed" : props.occurrence.overdue ? " · Overdue" : ""}`}
    title={label()}
  >
    {label()}
  </CalendarGrid.Occurrence>;
}
