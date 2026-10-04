import type { HeatmapDay } from "@polterware/frontend-design-system";
import type { ReviewDayJSON, ReviewJSON, SettingsJSON } from "./bridge";

export type Honesty = "live" | "queued" | "failed" | "unavailable";
export type ReviewRange = "week" | "month" | "year";
export type WeekStart = 0 | 1;
export type CharacterMood = "sad" | "hopeful" | "happy" | "euphoric";
export type ActivityPeriod = "morning" | "afternoon" | "evening" | "night";

export const PERIODS: readonly ActivityPeriod[] = ["morning", "afternoon", "evening", "night"];
export const PERIOD_LABEL: Record<ActivityPeriod, string> = {
  morning: "Morning",
  afternoon: "Afternoon",
  evening: "Evening",
  night: "Night",
};

/** Swift ContributionCalendarLayout.intensityLevel mapped onto heatmapLevel thresholds. */
export const HEATMAP_THRESHOLDS = [1, 2, 3, 5] as const;

/** Core review.StreakMood (types.go): 0 sad, 1–6 hopeful, 7–29 happy, ≥30 euphoric. */
export const CORE_STREAK_MOOD = { hopefulMin: 1, happyMin: 7, euphoricMin: 30 } as const;
/** Core review.CompletedMood: 0 sad, 1–24 hopeful, 25–99 happy, ≥100 euphoric. */
export const CORE_COMPLETED_MOOD = { hopefulMin: 1, happyMin: 25, euphoricMin: 100 } as const;

const SWIFT_STREAK_FIXED = [3, 7, 14, 30] as const;
const SWIFT_COMPLETED_FIXED = [7, 25, 50, 100] as const;

export type CivilDate = { year: number; month: number; day: number; weekday: number; hour: number; minute: number; second: number; ms: number };

const WEEKDAY: Record<string, number> = { Sun: 0, Mon: 1, Tue: 2, Wed: 3, Thu: 4, Fri: 5, Sat: 6 };

export function honestyTone(state: Honesty): "ok" | "syncing" | "danger" | "muted" {
  if (state === "live") return "ok";
  if (state === "queued") return "syncing";
  if (state === "failed") return "danger";
  return "muted";
}

export function honestyLabel(state: Honesty): string {
  if (state === "live") return "Live";
  if (state === "queued") return "Queued";
  if (state === "failed") return "Failed";
  return "Unavailable";
}

export function resolveTimezone(tz?: string): string {
  if (tz && tz.length > 0) return tz;
  return Intl.DateTimeFormat().resolvedOptions().timeZone || "UTC";
}

/**
 * Frozen Settings has no week-start field. Monday matches MCP `get_review`.
 * Callers still load `settings.get` so a later calendar key can be wired here.
 */
export function weekStartFromSettings(_settings: SettingsJSON | undefined, localeFirstDay?: number): WeekStart {
  if (localeFirstDay === 7) return 0;
  return 1;
}

export function civilDateInZone(date: Date, tz: string): CivilDate {
  const parts = new Intl.DateTimeFormat("en-US", {
    timeZone: tz,
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
    weekday: "short",
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
    hourCycle: "h23",
  }).formatToParts(date);
  const get = (type: Intl.DateTimeFormatPartTypes) => parts.find(part => part.type === type)?.value ?? "0";
  return {
    year: Number(get("year")),
    month: Number(get("month")),
    day: Number(get("day")),
    weekday: WEEKDAY[get("weekday")] ?? 0,
    hour: Number(get("hour")),
    minute: Number(get("minute")),
    second: Number(get("second")),
    ms: 0,
  };
}

export function zonedWallTime(tz: string, year: number, month: number, day: number, hour = 0, minute = 0, second = 0, ms = 0): Date {
  const wanted = Date.UTC(year, month - 1, day, hour, minute, second, ms);
  let guess = wanted;
  for (let i = 0; i < 4; i++) {
    const got = civilDateInZone(new Date(guess), tz);
    const actual = Date.UTC(got.year, got.month - 1, got.day, got.hour, got.minute, got.second, got.ms);
    guess += wanted - actual;
  }
  return new Date(guess);
}

export function addCivilDays(date: CivilDate, days: number): CivilDate {
  const next = new Date(Date.UTC(date.year, date.month - 1, date.day + days, 12));
  return { year: next.getUTCFullYear(), month: next.getUTCMonth() + 1, day: next.getUTCDate(), weekday: next.getUTCDay(), hour: 0, minute: 0, second: 0, ms: 0 };
}

export function formatCivil(date: Pick<CivilDate, "year" | "month" | "day">): string {
  return `${date.year}-${String(date.month).padStart(2, "0")}-${String(date.day).padStart(2, "0")}`;
}

export function parseCivil(value: string): CivilDate | null {
  const match = /^(\d{4})-(\d{2})-(\d{2})$/.exec(value);
  if (!match) return null;
  const year = Number(match[1]), month = Number(match[2]), day = Number(match[3]);
  const probe = new Date(Date.UTC(year, month - 1, day, 12));
  if (probe.getUTCFullYear() !== year || probe.getUTCMonth() + 1 !== month || probe.getUTCDate() !== day) return null;
  return { year, month, day, weekday: probe.getUTCDay(), hour: 0, minute: 0, second: 0, ms: 0 };
}

/** Half-open `[from, to)` window. `to` is now + 1 ms so the current instant is included, matching core `from < to`. */
export function reviewWindow(now: Date, tz: string, range: ReviewRange, weekStart: WeekStart): { from: string; to: string } {
  const civil = civilDateInZone(now, tz);
  let start = civil;
  if (range === "week") start = addCivilDays(civil, -((civil.weekday - weekStart + 7) % 7));
  else if (range === "month") start = { ...civil, day: 1, weekday: new Date(Date.UTC(civil.year, civil.month - 1, 1, 12)).getUTCDay() };
  else start = { ...civil, month: 1, day: 1, weekday: new Date(Date.UTC(civil.year, 0, 1, 12)).getUTCDay() };
  return {
    from: zonedWallTime(tz, start.year, start.month, start.day).toISOString(),
    to: new Date(now.getTime() + 1).toISOString(),
  };
}

export function characterMood(value: string | undefined): CharacterMood {
  if (value === "hopeful" || value === "happy" || value === "euphoric" || value === "sad") return value;
  return "sad";
}

export function characterAsset(kind: "streak" | "completed", mood: string | undefined): { src: string; srcSet: string; alt: string } {
  const id = characterMood(mood);
  const file = `me-${kind}-${id}`;
  return {
    src: `/characters/${file}.png`,
    srcSet: `/characters/${file}.png 1x, /characters/${file}@2x.png 2x, /characters/${file}@3x.png 3x`,
    alt: "",
  };
}

export function streakTarget(value: number): number {
  return milestoneTarget(value, SWIFT_STREAK_FIXED, 30);
}

export function completionTarget(value: number): number {
  return milestoneTarget(value, SWIFT_COMPLETED_FIXED, 100);
}

export function milestoneProgress(value: number, target: number): number {
  if (target <= 0) return 0;
  return Math.min(Math.max(value / target, 0), 1);
}

function milestoneTarget(value: number, fixed: readonly number[], repeatingStep: number): number {
  const n = Math.max(value, 0);
  const found = fixed.find(item => item >= n);
  if (found != null) return found;
  return Math.ceil(n / repeatingStep) * repeatingStep;
}

export function periodCount(day: ReviewDayJSON | undefined, period: ActivityPeriod): number {
  if (!day) return 0;
  if (period === "morning") return day.morning ?? 0;
  if (period === "afternoon") return day.afternoon ?? 0;
  if (period === "evening") return day.evening ?? 0;
  return day.night ?? 0;
}

export function isEmptyReview(report: ReviewJSON | null | undefined): boolean {
  if (!report) return true;
  const days = [...(report.days ?? []), ...(report.contribution ?? [])];
  return (report.completedTotal ?? 0) === 0
    && (report.focusMinutes ?? 0) === 0
    && !days.some(day => (day.completed ?? 0) > 0 || (day.focusMinutes ?? 0) > 0);
}

export type HeatmapCell = HeatmapDay & { inRange: boolean; completed: number; focusMinutes: number };

export function padContributionWeeks(days: readonly ReviewDayJSON[], weekStart: WeekStart): HeatmapCell[] {
  const source = days.filter(day => parseCivil(day.date ?? ""));
  if (!source.length) return [];
  const first = parseCivil(source[0]!.date!)!;
  const last = parseCivil(source[source.length - 1]!.date!)!;
  const leading = (first.weekday - weekStart + 7) % 7;
  const trailing = (weekStart + 6 - last.weekday + 7) % 7;
  const byDate = new Map(source.map(day => [day.date, day]));
  const cells: HeatmapCell[] = [];
  for (let offset = -leading; offset <= civilDiff(first, last) + trailing; offset++) {
    const civil = addCivilDays(first, offset);
    const date = formatCivil(civil);
    const day = byDate.get(date);
    const inRange = offset >= 0 && offset <= civilDiff(first, last);
    cells.push({
      date,
      value: day?.completed ?? 0,
      completed: day?.completed ?? 0,
      focusMinutes: day?.focusMinutes ?? 0,
      inRange,
    });
  }
  return cells;
}

function civilDiff(from: CivilDate, to: CivilDate): number {
  const a = Date.UTC(from.year, from.month - 1, from.day);
  const b = Date.UTC(to.year, to.month - 1, to.day);
  return Math.round((b - a) / 86_400_000);
}

export function heatmapLabel(cell: HeatmapCell): string {
  const when = formatLongDate(cell.date);
  if (!cell.inRange) return `${when}: outside the 12-month window`;
  return `${when}: ${completionPhrase(cell.completed)}`;
}

export function completionPhrase(count: number): string {
  if (count <= 0) return "No completions";
  return count === 1 ? "1 completion" : `${count} completions`;
}

export function formatLongDate(isoDate: string): string {
  const civil = parseCivil(isoDate);
  if (!civil) return isoDate;
  return new Intl.DateTimeFormat("en-US", { dateStyle: "long" }).format(new Date(Date.UTC(civil.year, civil.month - 1, civil.day, 12)));
}

export function weekdayLabels(weekStart: WeekStart): (string | null)[] {
  return Array.from({ length: 7 }, (_, row) => {
    const weekday = (weekStart + row) % 7;
    if (weekday === 1) return "Mon";
    if (weekday === 3) return "Wed";
    if (weekday === 5) return "Fri";
    return null;
  });
}

export function weekdayLetter(isoDate: string): string {
  const civil = parseCivil(isoDate);
  if (!civil) return "";
  return ["S", "M", "T", "W", "T", "F", "S"][civil.weekday] ?? "";
}

export function weekdayName(isoDate: string): string {
  const civil = parseCivil(isoDate);
  if (!civil) return isoDate;
  return ["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"][civil.weekday] ?? isoDate;
}

export type FocusWeekBucket = { id: string; label: string; minutes: number; start: string; end: string };

export function focusWeekBuckets(days: readonly ReviewDayJSON[], weekStart: WeekStart): FocusWeekBucket[] {
  const buckets = new Map<string, FocusWeekBucket>();
  for (const day of days) {
    const civil = parseCivil(day.date ?? "");
    if (!civil) continue;
    const start = addCivilDays(civil, -((civil.weekday - weekStart + 7) % 7));
    const id = formatCivil(start);
    const current = buckets.get(id) ?? { id, label: id, minutes: 0, start: id, end: id };
    current.minutes += day.focusMinutes ?? 0;
    current.end = day.date ?? id;
    buckets.set(id, current);
  }
  return [...buckets.values()].map(bucket => ({
    ...bucket,
    label: bucket.start === bucket.end ? formatLongDate(bucket.start) : `${formatLongDate(bucket.start)} – ${formatLongDate(bucket.end)}`,
  }));
}

export function activityValue(day: ReviewDayJSON): string {
  const total = day.completed ?? 0;
  if (total <= 0) return "No completions";
  const details = PERIODS.flatMap(period => {
    const count = periodCount(day, period);
    return count > 0 ? [`${PERIOD_LABEL[period]} ${count}`] : [];
  });
  return `${total} completed. ${details.join(", ")}`;
}

export function streakAccessibility(streak: number, target: number): string {
  const current = streak === 1 ? "day" : "days";
  const next = target === 1 ? "day" : "days";
  return `${streak} ${current}, next milestone ${target} ${next}`;
}

export function completedAccessibility(completed: number, target: number): string {
  const current = completed === 1 ? "completion" : "completions";
  const next = target === 1 ? "completion" : "completions";
  return `${completed} ${current} all time, next milestone ${target} ${next}`;
}

export function moveHeatmapIndex(index: number, key: string, count: number): number {
  if (count <= 0) return 0;
  const column = 7;
  if (key === "ArrowDown") return Math.min(count - 1, index + 1);
  if (key === "ArrowUp") return Math.max(0, index - 1);
  if (key === "ArrowRight") return Math.min(count - 1, index + column);
  if (key === "ArrowLeft") return Math.max(0, index - column);
  if (key === "Home") return 0;
  if (key === "End") return count - 1;
  return index;
}
