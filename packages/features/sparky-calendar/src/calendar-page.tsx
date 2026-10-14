import { For, Show, createEffect, createMemo, createSignal, onCleanup, onMount, type JSX } from "solid-js";
import { useSparkyShell } from "@polterware/feature-sparky-shell/feature";
import {
  A11yDialog, Button, CalendarGrid, ChevronLeft, ChevronRight, PageContent, PageEmpty, PageError, PageHeader,
  PageLoading, PageToolbar, Plus, Popover, StatusChip, Tabs, prefersReducedMotion,
} from "@polterware/frontend-design-system";
import {
  agendaHeading, civilFromInstant, clockLabel, formatYmd, gmtLabel, monthDates, monthName, periodDates,
  placeTimed, rangeTitle, sameMonth, shiftedAnchor, standardHours, systemTimeZone, timeLabel, visibleRange,
  weekdayLabels, weekdayName, yearMonthAnchors, type CalendarView, type WeekStart,
} from "./grid";
import type { SlotDraft } from "./mapping";
import { calendarStore, type CalendarOccurrence, type CalendarStore } from "./store";
import "./calendar.css";

const VIEWS: { value: CalendarView; label: string }[] = [
  { value: "schedule", label: "Schedule" },
  { value: "day", label: "Day" },
  { value: "week", label: "Week" },
  { value: "month", label: "Month" },
  { value: "year", label: "Year" },
];
const MONTH_VISIBLE = 3;
const ALL_DAY_VISIBLE = 3;
const HOUR_HEIGHT = 64;
const HOUR_COUNT = 24;

function isCalendarView(id: string): id is CalendarView {
  return id === "schedule" || id === "day" || id === "week" || id === "month" || id === "year";
}

function dayNumber(date: string): string {
  return String(Number(date.slice(8)));
}

function inkFor(fill: string): string {
  const match = /^#([0-9a-f]{6})$/i.exec(fill.trim());
  if (!match) return "var(--color-text-primary)";
  const value = Number.parseInt(match[1]!, 16);
  const channel = (shift: number) => (value >> shift) & 255;
  const luminance = (0.2126 * channel(16) + 0.7152 * channel(8) + 0.0722 * channel(0)) / 255;
  return luminance > 0.62 ? "var(--color-icon-tile-mark)" : "var(--color-text-primary)";
}

function paint(occurrence: CalendarOccurrence): { fill: string; ink: string } {
  if (occurrence.overdue) return { fill: "var(--color-danger)", ink: "var(--color-status-danger-foreground)" };
  const fill = occurrence.color || "var(--color-brand)";
  return { fill, ink: fill.startsWith("var(") ? "var(--color-icon-tile-mark)" : inkFor(fill) };
}

function eventBox(occurrence: CalendarOccurrence): { top: number; height: number; end: number } {
  const wall = civilFromInstant(occurrence.occurrenceAt, occurrence.tz);
  const total = HOUR_COUNT * HOUR_HEIGHT;
  const top = Math.min((wall.hour + wall.minute / 60) * HOUR_HEIGHT, total - 28);
  const height = Math.min(HOUR_HEIGHT, total - top);
  return { top, height, end: top + height };
}

function scrollParent(element: HTMLElement): HTMLElement | null {
  let node = element.parentElement;
  while (node) {
    const overflow = getComputedStyle(node).overflowY;
    if (overflow === "auto" || overflow === "scroll") return node;
    node = node.parentElement;
  }
  return null;
}

function revealNow(pane: HTMLElement) {
  const line = pane.querySelector<HTMLElement>("[data-now-line]");
  if (!line) return;
  const scroller = pane.scrollHeight > pane.clientHeight + 1 ? pane : scrollParent(line) ?? pane;
  const delta = line.getBoundingClientRect().top - scroller.getBoundingClientRect().top - scroller.clientHeight / 3;
  scroller.scrollTop += delta;
}
const PANES = [-1, 0, 1] as const;
const PAN_SLOP = 2;
const PAN_THRESHOLD = 0.18;
const PAN_FLICK = 0.6;

function panDirection(offset: number, velocity: number, width: number): -1 | 0 | 1 {
  const distance = Math.abs(offset);
  if (distance < PAN_SLOP) return 0;
  const threshold = width > 0 ? width * PAN_THRESHOLD : 8;
  const flicked = Math.abs(velocity) > PAN_FLICK && distance >= 8;
  if (!flicked && distance < threshold) return 0;
  if (flicked) return velocity < 0 ? 1 : -1;
  return offset < 0 ? 1 : -1;
}

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
  const [panX, setPanX] = createSignal(0);
  const [settling, setSettling] = createSignal(false);
  const [panning, setPanning] = createSignal(false);
  let root: HTMLDivElement | undefined;
  let viewport: HTMLDivElement | undefined;
  let track: HTMLDivElement | undefined;
  let drag: { pointerId: number; startX: number; startY: number; origin: number; lastX: number; lastTime: number; velocity: number; moved: boolean } | undefined;
  let settleToken = 0;
  let wheelOffset = 0;
  let wheelVelocity = 0;
  let wheelLast = 0;
  let wheelTimer = 0;
  let settleTimer = 0;
  const title = createMemo(() => rangeTitle(anchor(), view(), store.weekStart()));
  const dates = createMemo(() => periodDates(anchor(), view(), store.weekStart()));
  const paneWidth = () => viewport?.clientWidth ?? 0;

  onMount(() => {
    const disconnect = store.connect();
    const timer = setInterval(() => setNow(new Date()), 60_000);
    void store.loadSettings().then(() => store.ensureVisible(anchor(), view(), store.displayTz()));
    const wheel = (event: WheelEvent) => onWheel(event);
    viewport?.addEventListener("wheel", wheel, { passive: false });
    onCleanup(() => { disconnect(); clearInterval(timer); viewport?.removeEventListener("wheel", wheel); window.clearTimeout(wheelTimer); window.clearTimeout(settleTimer); });
  });
  createEffect(() => {
    const next = visibleRange(anchor(), view(), store.weekStart(), store.displayTz());
    void store.ensureVisible(anchor(), view(), store.displayTz());
    root?.setAttribute("data-calendar-from", next.from);
    root?.setAttribute("data-calendar-to", next.to);
  });
  let scrolledFor = "";
  createEffect(() => {
    const current = view();
    const key = `${current}:${anchor()}:${store.displayTz()}`;
    if (current !== "day" && current !== "week") return;
    if (scrolledFor === key) return;
    queueMicrotask(() => requestAnimationFrame(() => requestAnimationFrame(() => {
      if (scrolledFor === key) return;
      const pane = root?.querySelector<HTMLElement>("[data-pane='current'] .sparky-calendar__pane-scroll");
      if (!pane || pane.clientHeight === 0) return;
      scrolledFor = key;
      revealNow(pane);
    })));
  });

  const gridButtons = () => [...(root?.querySelectorAll<HTMLButtonElement>(".sparky-calendar__pane[data-pane='current'] button:not(:disabled)") ?? [])];
  const allGridButtons = () => [...(root?.querySelectorAll<HTMLButtonElement>(".sparky-calendar__grid-wrap button") ?? [])];
  const dayButtons = (date: string) => gridButtons().filter(button => button.closest<HTMLElement>("[data-day]")?.dataset.day === date);
  function syncFocus() {
    const position = focusPosition();
    const buttons = gridButtons();
    const day = dayButtons(position.date);
    const active = day[Math.min(position.index, day.length - 1)] ?? buttons[0];
    for (const button of allGridButtons()) button.tabIndex = button === active ? 0 : -1;
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
  function go(next: string, nextView = view(), shiftView = true) {
    setAnchor(next);
    setFocused(next);
    setFocusPosition({ date: next, index: 0 });
    setView(nextView);
    if (shiftView) bump();
  }
  function finishSettle(token: number, direction: -1 | 0 | 1) {
    if (token !== settleToken) return;
    settleToken += 1;
    setSettling(false);
    setPanX(0);
    if (direction !== 0) go(shiftedAnchor(anchor(), view(), direction), view(), false);
  }
  function settle(direction: -1 | 0 | 1) {
    const width = paneWidth();
    const token = ++settleToken;
    if (direction === 0 && panX() === 0) {
      setSettling(false);
      return;
    }
    if (width === 0 || prefersReducedMotion()) {
      finishSettle(token, direction);
      return;
    }
    setSettling(true);
    const target = direction === 0 ? 0 : direction > 0 ? -width : width;
    requestAnimationFrame(() => {
      if (token !== settleToken) return;
      setPanX(target);
    });
    const node = track;
    const onEnd = (event: TransitionEvent) => {
      if (event.target !== node || event.propertyName !== "transform") return;
      node?.removeEventListener("transitionend", onEnd);
      finishSettle(token, direction);
    };
    node?.addEventListener("transitionend", onEnd);
    window.clearTimeout(settleTimer);
    settleTimer = window.setTimeout(() => {
      node?.removeEventListener("transitionend", onEnd);
      finishSettle(token, direction);
    }, 240);
  }
  function beginPointerPan(event: PointerEvent) {
    if (event.button !== 2) return;
    const target = event.target;
    if (!(target instanceof Element) || target.closest("[data-occurrence]")) return;
    event.preventDefault();
    settleToken += 1;
    setSettling(false);
    if (event.currentTarget instanceof HTMLElement) {
      try { event.currentTarget.setPointerCapture(event.pointerId); } catch { /* jsdom has no capture. */ }
    }
    drag = {
      pointerId: event.pointerId,
      startX: event.clientX,
      startY: event.clientY,
      origin: panX(),
      lastX: event.clientX,
      lastTime: performance.now(),
      velocity: 0,
      moved: false,
    };
  }
  function movePointerPan(event: PointerEvent) {
    if (!drag || event.pointerId !== drag.pointerId) return;
    const dx = event.clientX - drag.startX;
    const dy = event.clientY - drag.startY;
    if (!drag.moved) {
      if (Math.hypot(dx, dy) < PAN_SLOP) return;
      if (Math.abs(dy) > Math.abs(dx)) {
        drag = undefined;
        return;
      }
      drag.moved = true;
      setPanning(true);
    }
    event.preventDefault();
    const now = performance.now();
    drag.velocity = (event.clientX - drag.lastX) / Math.max(now - drag.lastTime, 1);
    drag.lastX = event.clientX;
    drag.lastTime = now;
    const width = paneWidth();
    const next = drag.origin + dx;
    setPanX(width > 0 ? Math.max(-width, Math.min(width, next)) : next);
  }
  function endPointerPan(event: PointerEvent) {
    if (!drag || event.pointerId !== drag.pointerId) return;
    const current = drag;
    drag = undefined;
    setPanning(false);
    if (!current.moved) return;
    settle(panDirection(panX(), current.velocity, paneWidth()));
  }
  function onWheel(event: WheelEvent) {
    if (drag || event.ctrlKey || event.metaKey) return;
    const line = event.deltaMode === 1 ? 16 : event.deltaMode === 2 ? Math.max(paneWidth(), 1) : 1;
    const deltaX = event.deltaX * line;
    const deltaY = event.deltaY * line;
    if (Math.abs(deltaX) <= Math.abs(deltaY)) return;
    event.preventDefault();
    settleToken += 1;
    setSettling(false);
    const now = performance.now();
    wheelVelocity = -deltaX / Math.max(now - wheelLast, 1);
    wheelLast = now;
    wheelOffset += deltaX;
    const width = paneWidth();
    const next = -wheelOffset;
    setPanX(width > 0 ? Math.max(-width, Math.min(width, next)) : next);
    setPanning(true);
    window.clearTimeout(wheelTimer);
    wheelTimer = window.setTimeout(() => {
      const offset = panX();
      const velocity = wheelVelocity;
      wheelOffset = 0;
      wheelVelocity = 0;
      setPanning(false);
      settle(panDirection(offset, velocity, paneWidth()));
    }, 80);
  }
  async function create(draft: SlotDraft) {
    try {
      await store.createFromSlot(draft, id => shell.openNote(id));
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
      if (view() === "year" && target.classList.contains("pw-calendar__mini-day")) {
        event.preventDefault();
        const buttons = [...(root?.querySelectorAll<HTMLButtonElement>(".sparky-calendar__pane[data-pane='current'] .pw-calendar__mini-day") ?? [])];
        const index = buttons.indexOf(target);
        const delta = key === "ArrowRight" ? 1 : key === "ArrowLeft" ? -1 : key === "ArrowDown" ? 7 : key === "ArrowUp" ? -7 : key === "Home" ? -index : buttons.length - 1 - index;
        const next = buttons[index + delta];
        if (next) {
          const date = next.closest<HTMLElement>("[data-day]")?.dataset.day ?? focused();
          setFocused(date);
          focusAfterRender = true;
          setFocusPosition({ date, index: Math.max(0, dayButtons(date).indexOf(next)) });
          return;
        }
        if (delta !== 0) go(shiftedAnchor(anchor(), "year", delta > 0 ? 1 : -1), "year");
        return;
      }
      if (view() === "schedule" && target.classList.contains("pw-calendar__agenda-item")) {
        event.preventDefault();
        const buttons = gridButtons().filter(button => button.classList.contains("pw-calendar__agenda-item"));
        const index = buttons.indexOf(target);
        const delta = key === "ArrowDown" || key === "ArrowRight" ? 1 : key === "ArrowUp" || key === "ArrowLeft" ? -1 : key === "Home" ? -index : buttons.length - 1 - index;
        const next = buttons[index + delta];
        if (next) {
          const date = next.closest<HTMLElement>("[data-day]")?.dataset.day ?? focused();
          setFocused(date);
          focusAfterRender = true;
          setFocusPosition({ date, index: Math.max(0, dayButtons(date).indexOf(next)) });
          return;
        }
        if (delta !== 0) go(shiftedAnchor(anchor(), "schedule", delta > 0 ? 1 : -1), "schedule");
        return;
      }
      const date = target.closest<HTMLElement>("[data-day]")?.dataset.day;
      if (!date) return;
      event.preventDefault();
      const day = dayButtons(date);
      const index = day.indexOf(target);
      let nextDate = date;
      let nextIndex = index;
      const timed = view() === "day" || view() === "week";
      if (key === "Home" || key === "End") nextIndex = key === "Home" ? 0 : day.length - 1;
      else if (view() === "month" && (key === "ArrowUp" || key === "ArrowDown")) {
        nextDate = shiftedAnchor(date, "day", key === "ArrowDown" ? 7 : -7);
      } else if (timed && (key === "ArrowLeft" || key === "ArrowRight")) {
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
    } else if (key === "t" || key === "d" || key === "w" || key === "m" || key === "y" || key === "s") {
      event.preventDefault();
      focusAfterRender = Boolean(inGrid);
      const nextView = key === "d" ? "day" : key === "w" ? "week" : key === "m" ? "month" : key === "y" ? "year" : key === "s" ? "schedule" : view();
      go(key === "t" ? todayYmd(store.displayTz()) : focused(), nextView);
    }
  }

  return <div ref={root} class="sparky-calendar" data-calendar data-calendar-view={view()} data-save-state={store.saveState()} tabindex="-1" onKeyDown={keyboard}>
    <PageHeader
      title={`${title().leading} ${title().trailing}`.trim()}
      subtitle={<Show when={store.saveState() !== "live"}><StatusChip tone={tone(store.saveState())}>{stateLabel(store.saveState())}</StatusChip></Show>}
    />
    <PageToolbar.Root>
      <PageToolbar.Tabs>
        <Tabs aria-label="Calendar view" items={VIEWS.map(item => ({ id: item.value, label: item.label }))} value={view()} onChange={id => { if (isCalendarView(id)) go(anchor(), id); }} />
      </PageToolbar.Tabs>
      <PageToolbar.Actions>
        <div class="sparky-calendar__nav">
          <Button variant="ghost" size="icon" class="sparky-calendar__nav-button" aria-label="Previous" onClick={() => go(shiftedAnchor(anchor(), view(), -1))}><ChevronLeft size={14} /></Button>
          <Button variant="ghost" size="sm" class="sparky-calendar__today" onClick={() => go(todayYmd(store.displayTz()))}>Today</Button>
          <Button variant="ghost" size="icon" class="sparky-calendar__nav-button" aria-label="Next" onClick={() => go(shiftedAnchor(anchor(), view(), 1))}><ChevronRight size={14} /></Button>
        </div>
      </PageToolbar.Actions>
    </PageToolbar.Root>
    <PageContent class="sparky-calendar__content">
      <Show when={store.pending() && !store.occurrences().length}><PageLoading label="Loading calendar" /></Show>
      <Show when={store.loadError()}><PageError title="Calendar unavailable" message={store.loadError()} onRetry={() => void store.refresh()} /></Show>
      <Show when={store.actionError()}><p role="alert">{store.actionError()}</p></Show>
      <Show when={!store.loadError()}>
        <div
          ref={viewport}
          class="sparky-calendar__grid-wrap"
          onFocusIn={focusGrid}
          data-shift={shift() && !panning() ? "" : undefined}
          data-panning={panning() ? "" : undefined}
          onPointerDown={beginPointerPan}
          onPointerMove={movePointerPan}
          onPointerUp={endPointerPan}
          onPointerCancel={endPointerPan}
          onContextMenu={event => event.preventDefault()}
          onAuxClick={event => event.preventDefault()}
        >
          <div
            ref={track}
            class="sparky-calendar__track"
            style={{
              transform: `translate3d(calc(-33.333333% + ${panX()}px), 0, 0)`,
              transition: settling() ? "transform var(--spring-moderate-duration) var(--spring-moderate-ease)" : "none",
            }}
          >
            <For each={PANES}>{offset =>
              <PeriodPane
                anchor={shiftedAnchor(anchor(), view(), offset)}
                view={view()}
                current={offset === 0}
                weekStart={store.weekStart()}
                today={todayYmd(store.displayTz())}
                focused={focused()}
                now={now()}
                displayTz={store.displayTz()}
                allDay={store.allDayOn}
                timed={store.timedOn}
                onOpen={id => shell.openNote(id)}
                onCreate={create}
                onDrop={drop}
                onOpenDay={date => go(date, "day")}
                onOpenMonth={date => go(date, "month")}
                onOverflow={(date, items) => setOverflow({ date, items })}
              />
            }</For>
          </div>
        </div>
      </Show>
    </PageContent>
    <Popover
      open={Boolean(overflow())}
      onClose={() => setOverflow(undefined)}
      title="More occurrences"
      anchorRef={() => root?.querySelector(`[data-pane="current"] [data-day="${overflow()?.date ?? ""}"] [aria-label$="more occurrences"]`) as HTMLElement | undefined}
      width={280}
    >
      <div class="sparky-calendar__overflow-list">
        <For each={overflow()?.items ?? []}>{item =>
          <OccurrencePill occurrence={item} onOpen={() => { setOverflow(undefined); shell.openNote(item.noteId); }} />
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

function PeriodPane(props: {
  anchor: string;
  view: CalendarView;
  current: boolean;
  weekStart: WeekStart;
  today: string;
  focused: string;
  now: Date;
  displayTz: string;
  allDay: (date: string) => CalendarOccurrence[];
  timed: (date: string) => CalendarOccurrence[];
  onOpen: (id: string) => void;
  onCreate: (draft: SlotDraft) => void;
  onDrop: (occurrence: CalendarOccurrence, draft: SlotDraft) => void;
  onOpenDay: (date: string) => void;
  onOpenMonth: (date: string) => void;
  onOverflow: (date: string, items: CalendarOccurrence[]) => void;
}) {
  const dates = createMemo(() => periodDates(props.anchor, props.view === "year" || props.view === "schedule" ? "week" : props.view, props.weekStart));
  const label = createMemo(() => {
    const heading = rangeTitle(props.anchor, props.view, props.weekStart);
    return `${heading.leading} ${heading.trailing}`.trim();
  });
  const hours = createMemo(() => standardHours());
  const timeDates = createMemo(() => props.view === "day" ? [props.anchor] : dates());
  return <div class="sparky-calendar__pane" data-pane={props.current ? "current" : "neighbor"} inert={!props.current} aria-hidden={props.current ? undefined : true}>
    <div class="sparky-calendar__pane-scroll">
      <CalendarGrid.Root view={props.view} label={label()} hourHeight={HOUR_HEIGHT} gutterWidth={64}>
        <Show when={props.view === "schedule"}>
          <ScheduleList
            dates={periodDates(props.anchor, "schedule", props.weekStart)}
            today={props.today}
            focused={props.current ? props.focused : ""}
            allDay={props.allDay}
            timed={props.timed}
            onOpen={props.onOpen}
          />
        </Show>
        <Show when={props.view === "year"}>
          <YearGrid
            anchor={props.anchor}
            weekStart={props.weekStart}
            today={props.today}
            focused={props.current ? props.focused : ""}
            onOpenDay={props.onOpenDay}
            onOpenMonth={props.onOpenMonth}
          />
        </Show>
        <Show when={props.view === "month"}>
          <MonthGrid
            anchor={props.anchor}
            weekStart={props.weekStart}
            today={props.today}
            focused={props.current ? props.focused : ""}
            allDay={props.allDay}
            timed={props.timed}
            onOpen={props.onOpen}
            onOpenDay={props.onOpenDay}
            onOverflow={props.onOverflow}
          />
        </Show>
        <Show when={props.view === "day" || props.view === "week"}>
          <TimeGrid
            dates={timeDates()}
            hours={hours()}
            weekStart={props.weekStart}
            today={props.today}
            focused={props.current ? props.focused : ""}
            now={props.now}
            displayTz={props.displayTz}
            allDay={props.allDay}
            timed={props.timed}
            onOpen={props.onOpen}
            onCreate={props.onCreate}
            onDrop={props.onDrop}
            onOpenDay={props.onOpenDay}
            onOverflow={props.onOverflow}
          />
        </Show>
      </CalendarGrid.Root>
    </div>
  </div>;
}

function TimeGrid(props: {
  dates: string[];
  hours: { hour: number; label: string }[];
  weekStart: WeekStart;
  today: string;
  focused: string;
  now: Date;
  displayTz: string;
  allDay: (date: string) => CalendarOccurrence[];
  timed: (date: string) => CalendarOccurrence[];
  onOpen: (id: string) => void;
  onCreate: (draft: SlotDraft) => void;
  onDrop: (occurrence: CalendarOccurrence, draft: SlotDraft) => void;
  onOpenDay: (date: string) => void;
  onOverflow: (date: string, items: CalendarOccurrence[]) => void;
}) {
  const nowParts = () => civilFromInstant(props.now.toISOString(), props.displayTz);
  return <CalendarGrid.Time columns={props.dates.length}>
    <CalendarGrid.TimeSticky zone={gmtLabel(props.displayTz, props.now)}>
      <For each={props.dates}>{date => {
        const items = () => props.allDay(date);
        const visible = () => items().slice(0, ALL_DAY_VISIBLE);
        const extra = () => items().slice(visible().length);
        return <div class="pw-calendar__head-day" data-day={date} data-focused={props.focused === date ? "" : undefined}>
          <Button
            variant="ghost"
            class="pw-calendar__day-heading"
            data-today={date === props.today ? "" : undefined}
            aria-label={`Open ${dateLabel(date)} in day view`}
            onClick={() => props.onOpenDay(date)}
          >
            <span class="pw-calendar__weekday">{weekdayName(date, props.weekStart)}</span>
            <span class="pw-calendar__date-num" data-today={date === props.today ? "" : undefined}>{dayNumber(date)}</span>
          </Button>
          <Show when={visible().length || extra().length}>
            <div class="pw-calendar__all-day" aria-label="All day">
              <For each={visible()}>{item =>
                <OccurrencePill variant="banner" occurrence={item} draggable onOpen={() => props.onOpen(item.noteId)} />
              }</For>
              <Show when={extra().length}>
                <CalendarGrid.Overflow count={extra().length} onClick={() => props.onOverflow(date, extra())} />
              </Show>
            </div>
          </Show>
        </div>;
      }}</For>
    </CalendarGrid.TimeSticky>
    <CalendarGrid.TimeBody gutter={
      <For each={props.hours}>{(slot, index) => <CalendarGrid.GutterLabel hour={index()}>{clockLabel(slot.hour)}</CalendarGrid.GutterLabel>}</For>
    }>
      <For each={props.dates}>{date =>
        <TimeColumn
          date={date}
          hours={props.hours}
          today={date === props.today}
          nowTop={(nowParts().hour + nowParts().minute / 60) * HOUR_HEIGHT}
          timed={props.timed(date)}
          onOpen={props.onOpen}
          onCreate={props.onCreate}
          onDrop={props.onDrop}
        />
      }</For>
    </CalendarGrid.TimeBody>
  </CalendarGrid.Time>;
}

function TimeColumn(props: {
  date: string;
  hours: { hour: number; label: string }[];
  today: boolean;
  nowTop: number;
  timed: CalendarOccurrence[];
  onOpen: (id: string) => void;
  onCreate: (draft: SlotDraft) => void;
  onDrop: (occurrence: CalendarOccurrence, draft: SlotDraft) => void;
}) {
  const placed = createMemo(() => {
    const boxes = props.timed.map(eventBox);
    const columns = placeTimed(boxes.map(box => ({ start: box.top, end: box.end })));
    return props.timed.map((item, index) => ({ item, box: boxes[index]!, place: columns[index]! }));
  });
  return <div class="pw-calendar__column" data-day={props.date} data-today={props.today ? "" : undefined}>
    <For each={props.hours}>{slot =>
      <Button
        variant="ghost"
        class="pw-calendar__slot"
        data-empty-slot
        data-detail-opener
        data-slot-date={props.date}
        data-slot-time={slot.label}
        aria-label={`Add at ${slot.label} on ${dateLabel(props.date)}`}
        onClick={() => props.onCreate({ date: props.date, time: slot.label, allDay: false })}
        onDragOver={event => event.preventDefault()}
        onDrop={event => {
          const payload = event.dataTransfer?.getData("application/x-sparky-occurrence");
          if (!payload) return;
          const occurrence = JSON.parse(payload) as CalendarOccurrence;
          props.onDrop(occurrence, { date: props.date, time: slot.label, allDay: false, tz: occurrence.tz });
        }}
      >
        <Plus size={12} aria-hidden="true" />
      </Button>
    }</For>
    <For each={placed()}>{entry =>
      <OccurrencePill
        variant="block"
        occurrence={entry.item}
        draggable
        style={{
          top: `${entry.box.top}px`,
          height: `${Math.max(entry.box.height - 2, 28)}px`,
          left: `calc(${entry.place.column} * (100% - 4px) / ${entry.place.columns} + 2px)`,
          width: `calc((100% - 4px) / ${entry.place.columns})`,
        }}
        onOpen={() => props.onOpen(entry.item.noteId)}
      />
    }</For>
    <Show when={props.today}>
      <div class="pw-calendar__now" data-now-line style={{ top: `${props.nowTop}px` }}><span class="pw-calendar__now-dot"></span></div>
    </Show>
  </div>;
}

function MonthGrid(props: {
  anchor: string;
  weekStart: WeekStart;
  today: string;
  focused: string;
  allDay: (date: string) => CalendarOccurrence[];
  timed: (date: string) => CalendarOccurrence[];
  onOpen: (id: string) => void;
  onOpenDay: (date: string) => void;
  onOverflow: (date: string, items: CalendarOccurrence[]) => void;
}) {
  const dates = createMemo(() => monthDates(props.anchor, props.weekStart));
  return <>
    <CalendarGrid.Days class="pw-calendar__weekdays">
      <For each={weekdayLabels(props.weekStart)}>{name => <CalendarGrid.Header>{name}</CalendarGrid.Header>}</For>
    </CalendarGrid.Days>
    <CalendarGrid.Days>
      <For each={dates()}>{date => {
        const items = () => [...props.allDay(date), ...props.timed(date)];
        const visible = () => items().slice(0, MONTH_VISIBLE);
        const extra = () => items().slice(visible().length);
        return <CalendarGrid.Day label={dateLabel(date)} today={date === props.today}>
          <div
            class="pw-calendar__month-day"
            data-day={date}
            data-focused={props.focused === date ? "" : undefined}
            data-outside={sameMonth(date, props.anchor) ? undefined : ""}
          >
            <div class="pw-calendar__month-head">
              <Button
                variant="ghost"
                class="pw-calendar__date-num"
                data-today={date === props.today ? "" : undefined}
                aria-label={`Open ${dateLabel(date)} in day view`}
                onClick={() => props.onOpenDay(date)}
              >
                {dayNumber(date)}
              </Button>
            </div>
            <div class="pw-calendar__month-events">
              <For each={visible()}>{item =>
                <OccurrencePill variant="chip" occurrence={item} onOpen={() => props.onOpen(item.noteId)} />
              }</For>
              <Show when={extra().length}>
                <CalendarGrid.Overflow count={extra().length} onClick={() => props.onOverflow(date, extra())} />
              </Show>
            </div>
          </div>
        </CalendarGrid.Day>;
      }}</For>
    </CalendarGrid.Days>
  </>;
}

function YearGrid(props: {
  anchor: string;
  weekStart: WeekStart;
  today: string;
  focused: string;
  onOpenDay: (date: string) => void;
  onOpenMonth: (date: string) => void;
}) {
  const months = createMemo(() => yearMonthAnchors(props.anchor));
  const currentMonth = () => props.today.slice(0, 7);
  return <CalendarGrid.Year>
    <For each={months()}>{month =>
      <CalendarGrid.MiniMonth>
        <Button
          variant="ghost"
          class="pw-calendar__mini-name"
          data-current={month.slice(0, 7) === currentMonth() ? "" : undefined}
          aria-label={`Open ${monthName(month)} in month view`}
          onClick={() => props.onOpenMonth(month)}
        >
          {monthName(month)}
        </Button>
        <div class="pw-calendar__mini-weekdays" aria-hidden="true">
          <For each={weekdayLabels(props.weekStart)}>{name => <span>{name.slice(0, 1)}</span>}</For>
        </div>
        <div class="pw-calendar__mini-days">
          <For each={monthDates(month, props.weekStart)}>{date =>
            <div data-day={date} data-focused={props.focused === date ? "" : undefined}>
              <Button
                variant="ghost"
                class="pw-calendar__mini-day"
                data-today={date === props.today ? "" : undefined}
                data-outside={sameMonth(date, month) ? undefined : ""}
                aria-label={`Open ${dateLabel(date)} in day view`}
                onClick={() => props.onOpenDay(date)}
              >
                {dayNumber(date)}
              </Button>
            </div>
          }</For>
        </div>
      </CalendarGrid.MiniMonth>
    }</For>
  </CalendarGrid.Year>;
}

function ScheduleList(props: {
  dates: string[];
  today: string;
  focused: string;
  allDay: (date: string) => CalendarOccurrence[];
  timed: (date: string) => CalendarOccurrence[];
  onOpen: (id: string) => void;
}) {
  const days = createMemo(() => props.dates.flatMap(date => {
    const items = [...props.allDay(date), ...props.timed(date)].sort((left, right) => {
      if (left.allDay !== right.allDay) return left.allDay ? -1 : 1;
      return left.occurrenceAt.localeCompare(right.occurrenceAt);
    });
    if (!items.length && date !== props.today) return [];
    return [{ date, items }];
  }));
  const hasItems = createMemo(() => days().some(day => day.items.length > 0));
  return <Show when={days().length} fallback={<PageEmpty title="Nothing scheduled" />}>
    <CalendarGrid.Agenda>
      <For each={days()}>{day =>
        <section class="pw-calendar__agenda-day" data-day={day.date} data-focused={props.focused === day.date ? "" : undefined}>
          <h3 class="pw-calendar__agenda-date" data-today={day.date === props.today ? "" : undefined}>{agendaHeading(day.date)}</h3>
          <Show when={day.date === props.today}><div class="pw-calendar__agenda-now" data-now-line></div></Show>
          <For each={day.items}>{item =>
            <OccurrencePill variant="agenda" occurrence={item} onOpen={() => props.onOpen(item.noteId)} />
          }</For>
          <Show when={!hasItems()}><p class="pw-calendar__agenda-empty">Nothing scheduled</p></Show>
        </section>
      }</For>
    </CalendarGrid.Agenda>
  </Show>;
}

function occurrenceText(occurrence: CalendarOccurrence): string {
  if (occurrence.allDay) return occurrence.title;
  const wall = civilFromInstant(occurrence.occurrenceAt, occurrence.tz);
  return `${timeLabel(wall.hour, wall.minute)} ${occurrence.title}`;
}

function OccurrencePill(props: {
  occurrence: CalendarOccurrence;
  variant?: "banner" | "block" | "chip" | "agenda";
  draggable?: boolean;
  style?: JSX.CSSProperties;
  onOpen: () => void;
}) {
  const variant = () => props.variant ?? "banner";
  const colors = () => paint(props.occurrence);
  const wall = () => civilFromInstant(props.occurrence.occurrenceAt, props.occurrence.tz);
  const when = () => props.occurrence.allDay ? "All day" : timeLabel(wall().hour, wall().minute);
  const label = () => `${occurrenceText(props.occurrence)}${props.occurrence.allDay ? " · All day" : ""}${props.occurrence.completed ? " · Completed" : props.occurrence.overdue ? " · Overdue" : ""}`;
  const body = () => {
    if (variant() === "block") return <>
      <span class="pw-calendar__block-title">{props.occurrence.title}</span>
      <span class="pw-calendar__block-time">{when()}</span>
    </>;
    if (variant() === "chip") return <>
      <span class="pw-calendar__dot" style={{ background: props.occurrence.color || "var(--color-brand)" }}></span>
      <span class="pw-calendar__chip-label">{occurrenceText(props.occurrence)}</span>
    </>;
    if (variant() === "agenda") return <>
      <span class="pw-calendar__dot" style={{ background: props.occurrence.color || "var(--color-brand)" }}></span>
      <span class="pw-calendar__agenda-time">{when()}</span>
      <span class="pw-calendar__agenda-title">{props.occurrence.title}</span>
    </>;
    return <>{props.occurrence.title}</>;
  };
  return <CalendarGrid.Occurrence
    class={variant() === "block" ? "pw-calendar__block" : variant() === "chip" ? "pw-calendar__chip" : variant() === "agenda" ? "pw-calendar__agenda-item" : "pw-calendar__banner"}
    style={{
      ...(variant() === "block" || variant() === "banner" ? { background: colors().fill, color: colors().ink } : {}),
      ...props.style,
    }}
    data-occurrence={props.occurrence.noteId}
    data-detail-opener
    data-occurrence-at={props.occurrence.occurrenceAt}
    completed={props.occurrence.completed}
    overdue={props.occurrence.overdue}
    draggable={props.draggable}
    onDragStart={event => {
      event.dataTransfer?.setData("application/x-sparky-occurrence", JSON.stringify(props.occurrence));
    }}
    onClick={props.onOpen}
    aria-label={label()}
    title={occurrenceText(props.occurrence)}
  >
    {body()}
  </CalendarGrid.Occurrence>;
}
