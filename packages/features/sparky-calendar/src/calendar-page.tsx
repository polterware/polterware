import { For, Show, createEffect, createMemo, createSignal, onCleanup, onMount } from "solid-js";
import { useSparkyShell } from "@polterware/feature-sparky-shell/feature";
import {
  A11yDialog, Button, CalendarGrid, ChevronLeft, ChevronRight, DatePicker, PageContent, PageError, PageHeader,
  PageLoading, PageToolbar, parseDate, Popover, SegmentedControl, StatusChip, ToolbarIconButton, prefersReducedMotion,
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

  function bump() {
    if (prefersReducedMotion()) return;
    setShift(true);
    requestAnimationFrame(() => setShift(false));
  }
  function go(next: string, nextView = view()) {
    setAnchor(next);
    setFocused(next);
    setView(nextView);
    bump();
  }
  async function create(draft: SlotDraft) {
    try {
      const id = await store.createFromSlot(draft);
      shell.openMemory(id);
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
    if (event.target instanceof HTMLInputElement || event.target instanceof HTMLTextAreaElement) return;
    const key = event.key;
    if (key === "ArrowLeft" || key === "ArrowRight") {
      event.preventDefault();
      go(shiftedAnchor(focused(), "day", key === "ArrowRight" ? 1 : -1));
    } else if (key === "ArrowUp" || key === "ArrowDown") {
      event.preventDefault();
      const step = view() === "month" || view() === "week" ? 7 : 1;
      go(shiftedAnchor(focused(), "day", key === "ArrowDown" ? step : -step));
    } else if (key === "t") {
      event.preventDefault();
      go(todayYmd(store.displayTz()));
    } else if (key === "d" || key === "w" || key === "m") {
      event.preventDefault();
      go(focused(), key === "d" ? "day" : key === "w" ? "week" : "month");
    }
  }

  return <div ref={root} class="sparky-calendar" data-calendar data-calendar-view={view()} tabindex="0" onKeyDown={keyboard}>
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
        <div class="sparky-calendar__grid-wrap" data-shift={shift() ? "" : undefined}>
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
  return <CalendarGrid.Day label={props.date} today={props.date === props.today}>
    <div
      class="sparky-calendar__day"
      data-day={props.date}
      data-focused={props.focused ? "" : undefined}
      data-outside={props.view === "month" && !props.inMonth ? "" : undefined}
    >
      <CalendarGrid.Header>
        <Button variant="ghost" onClick={props.onOpenDay}>{props.date.slice(8)}</Button>
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
            data-slot-date={props.date}
            data-slot-all-day="true"
            aria-label={`New all-day memory on ${props.date}`}
            onClick={() => props.onCreate({ date: props.date, time: "00:00", allDay: true })}
          >
            New all-day Memory
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
                    data-slot-date={props.date}
                    data-slot-time={slot.label}
                    aria-label={`Add at ${slot.label} on ${props.date}`}
                    onClick={() => props.onCreate({ date: props.date, time: slot.label, allDay: false })}
                  >
                    Add at {slot.label}
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
    data-occurrence-at={props.occurrence.occurrenceAt}
    completed={props.occurrence.completed}
    overdue={props.occurrence.overdue}
    draggable={props.draggable}
    onDragStart={event => {
      event.dataTransfer?.setData("application/x-sparky-occurrence", JSON.stringify(props.occurrence));
    }}
    onClick={props.onOpen}
    aria-label={props.occurrence.title}
  >
    {label()}
  </CalendarGrid.Occurrence>;
}

