export type CalendarView = "day" | "week" | "month";
import type { SettingsJSON } from "./bridge";
export type WeekStart = "mon" | "sun";

export type CivilDate = { year: number; month: number; day: number };
export type Wall = CivilDate & { hour: number; minute: number; second: number };

export type DayHour = {
  hour: number;
  fold: number;
  label: string;
  startUtc: string;
};

export type VisibleRange = {
  from: string;
  to: string;
  dates: string[];
};

const DATE = /^(\d{4})-(\d{2})-(\d{2})$/;
const MONTH_CELLS = 42;
const WEEKDAYS_SUN = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"] as const;
const WEEKDAYS_MON = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"] as const;
const MONTH_NAMES = ["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"] as const;

export function pad(value: number): string {
  return String(value).padStart(2, "0");
}

export function formatYmd(parts: CivilDate): string {
  return `${parts.year}-${pad(parts.month)}-${pad(parts.day)}`;
}

export function parseYmd(value: string): CivilDate {
  const match = DATE.exec(value);
  if (!match) throw new Error("A valid date is required.");
  return { year: Number(match[1]), month: Number(match[2]), day: Number(match[3]) };
}

export function systemTimeZone(): string {
  return Intl.DateTimeFormat().resolvedOptions().timeZone || "UTC";
}

export function zoneParts(date: Date, timeZone: string): Wall {
  const parts = new Intl.DateTimeFormat("en-US", {
    timeZone,
    hourCycle: "h23",
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
  }).formatToParts(date);
  const read = (type: Intl.DateTimeFormatPartTypes) => Number(parts.find(part => part.type === type)?.value ?? "0");
  let hour = read("hour");
  if (hour === 24) hour = 0;
  return { year: read("year"), month: read("month"), day: read("day"), hour, minute: read("minute"), second: read("second") };
}

function offsetMs(instant: number, timeZone: string): number {
  const wall = zoneParts(new Date(instant), timeZone);
  return Date.UTC(wall.year, wall.month - 1, wall.day, wall.hour, wall.minute, wall.second) - instant;
}

function wallMatches(instant: number, parts: { year: number; month: number; day: number; hour: number; minute: number }, timeZone: string): boolean {
  const wall = zoneParts(new Date(instant), timeZone);
  return wall.year === parts.year && wall.month === parts.month && wall.day === parts.day && wall.hour === parts.hour && wall.minute === parts.minute;
}

/** Wall-clock in `timeZone` → UTC instant. Gaps shift forward; folds pick the earlier instant. */
export function wallClockToUtc(date: string, time: string, timeZone: string): string {
  const day = parseYmd(date);
  const [hourText, minuteText] = time.split(":");
  const hour = Number(hourText);
  const minute = Number(minuteText);
  if (!Number.isInteger(hour) || !Number.isInteger(minute) || hour > 23 || minute > 59) throw new Error("A valid time is required.");
  const utcGuess = Date.UTC(day.year, day.month - 1, day.day, hour, minute, 0, 0);
  let instant = utcGuess - offsetMs(utcGuess, timeZone);
  instant = utcGuess - offsetMs(instant, timeZone);
  if (wallMatches(instant, { ...day, hour, minute }, timeZone)) {
    let first = instant;
    for (let probe = instant - 4 * 3_600_000; probe < instant; probe += 60_000) {
      if (wallMatches(probe, { ...day, hour, minute }, timeZone)) {
        first = probe;
        break;
      }
    }
    return new Date(first).toISOString();
  }
  const previousDay = utcGuess - 24 * 3_600_000;
  return new Date(utcGuess - offsetMs(previousDay, timeZone)).toISOString();
}

export function civilFromInstant(iso: string, timeZone: string): Wall {
  const instant = Date.parse(iso);
  if (Number.isNaN(instant)) throw new Error("A valid instant is required.");
  return zoneParts(new Date(instant), timeZone);
}

export function addDays(date: string, amount: number): string {
  const { year, month, day } = parseYmd(date);
  const next = new Date(Date.UTC(year, month - 1, day + amount));
  return formatYmd({ year: next.getUTCFullYear(), month: next.getUTCMonth() + 1, day: next.getUTCDate() });
}

export function weekdayIndex(date: string): number {
  const { year, month, day } = parseYmd(date);
  return new Date(Date.UTC(year, month - 1, day)).getUTCDay();
}

export function startOfWeek(date: string, weekStart: WeekStart): string {
  const weekday = weekdayIndex(date);
  const first = weekStart === "sun" ? 0 : 1;
  const offset = (weekday - first + 7) % 7;
  return addDays(date, -offset);
}

export function weekDates(anchor: string, weekStart: WeekStart): string[] {
  const start = startOfWeek(anchor, weekStart);
  return Array.from({ length: 7 }, (_, index) => addDays(start, index));
}

export function monthStart(date: string): string {
  const { year, month } = parseYmd(date);
  return formatYmd({ year, month, day: 1 });
}

export function monthDates(anchor: string, weekStart: WeekStart): string[] {
  const start = startOfWeek(monthStart(anchor), weekStart);
  return Array.from({ length: MONTH_CELLS }, (_, index) => addDays(start, index));
}

export function addMonths(date: string, amount: number): string {
  const { year, month, day } = parseYmd(date);
  const cursor = new Date(Date.UTC(year, month - 1 + amount, 1));
  const last = new Date(Date.UTC(cursor.getUTCFullYear(), cursor.getUTCMonth() + 1, 0)).getUTCDate();
  return formatYmd({ year: cursor.getUTCFullYear(), month: cursor.getUTCMonth() + 1, day: Math.min(day, last) });
}

export function shiftedAnchor(anchor: string, view: CalendarView, direction: number): string {
  if (view === "day") return addDays(anchor, direction);
  if (view === "week") return addDays(anchor, direction * 7);
  return addMonths(anchor, direction);
}

export function weekdayLabels(weekStart: WeekStart): readonly string[] {
  return weekStart === "sun" ? WEEKDAYS_SUN : WEEKDAYS_MON;
}

export function rangeTitle(anchor: string, view: CalendarView, weekStart: WeekStart): { leading: string; trailing: string } {
  if (view === "week") {
    const dates = weekDates(anchor, weekStart);
    const first = parseYmd(dates[0]!);
    const last = parseYmd(dates[6]!);
    if (first.month === last.month && first.year === last.year) {
      return { leading: MONTH_NAMES[first.month - 1]!, trailing: String(first.year) };
    }
    if (first.year === last.year) {
      return { leading: `${MONTH_NAMES[first.month - 1]} – ${MONTH_NAMES[last.month - 1]}`, trailing: String(first.year) };
    }
    return { leading: `${MONTH_NAMES[first.month - 1]} ${first.year} – ${MONTH_NAMES[last.month - 1]} ${last.year}`, trailing: "" };
  }
  const { year, month, day } = parseYmd(anchor);
  if (view === "day") return { leading: `${MONTH_NAMES[month - 1]} ${day}`, trailing: String(year) };
  return { leading: MONTH_NAMES[month - 1]!, trailing: String(year) };
}

/** Half-open UTC range that covers every visible civil date in `timeZone`. */
export function visibleRange(anchor: string, view: CalendarView, weekStart: WeekStart, timeZone: string): VisibleRange {
  const dates = view === "day" ? [anchor] : view === "week" ? weekDates(anchor, weekStart) : monthDates(anchor, weekStart);
  const from = wallClockToUtc(dates[0]!, "00:00", timeZone);
  const to = wallClockToUtc(addDays(dates[dates.length - 1]!, 1), "00:00", timeZone);
  return { from, to, dates };
}

/** Hours that exist on a civil date. Spring-forward days have 23; fall-back days have 25. */
export function dayHours(date: string, timeZone: string): DayHour[] {
  const start = Date.parse(wallClockToUtc(date, "00:00", timeZone));
  const end = Date.parse(wallClockToUtc(addDays(date, 1), "00:00", timeZone));
  const hours: DayHour[] = [];
  const seen = new Map<number, number>();
  for (let instant = start; instant < end; instant += 3_600_000) {
    const wall = zoneParts(new Date(instant), timeZone);
    const fold = seen.get(wall.hour) ?? 0;
    seen.set(wall.hour, fold + 1);
    hours.push({ hour: wall.hour, fold, label: `${pad(wall.hour)}:00`, startUtc: new Date(instant).toISOString() });
  }
  return hours;
}

export function sameMonth(date: string, anchor: string): boolean {
  const left = parseYmd(date);
  const right = parseYmd(anchor);
  return left.year === right.year && left.month === right.month;
}

export function weekStartFromSettings(settings: SettingsJSON | undefined): WeekStart {
  return settings?.calendarWeekStart === "sun" ? "sun" : "mon";
}

export function occurrenceDate(iso: string, timeZone: string): string {
  const wall = civilFromInstant(iso, timeZone);
  return formatYmd(wall);
}

export function hourLabel(hour: number): string {
  return `${pad(hour)}:00`;
}
