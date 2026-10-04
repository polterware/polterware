import { heatmapLevel } from "@polterware/frontend-design-system";
import { describe, expect, it } from "vitest";
import {
  CORE_COMPLETED_MOOD,
  CORE_STREAK_MOOD,
  HEATMAP_THRESHOLDS,
  completionTarget,
  focusWeekBuckets,
  heatmapLabel,
  moveHeatmapIndex,
  padContributionWeeks,
  reviewWindow,
  streakTarget,
  weekStartFromSettings,
  weekdayLabels,
  type ActivityPeriod,
} from "./model";

/** Swift MeCharacterMood.streak(for:) */
function swiftStreakMood(value: number): string {
  const n = Math.max(value, 0);
  if (n === 0) return "sad";
  if (n < 7) return "hopeful";
  if (n < 30) return "happy";
  return "euphoric";
}

/** Swift MeCharacterMood.completed(for:) */
function swiftCompletedMood(value: number): string {
  const n = Math.max(value, 0);
  if (n === 0) return "sad";
  if (n < 25) return "hopeful";
  if (n < 100) return "happy";
  return "euphoric";
}

/** Core review.StreakMood / CompletedMood (types.go). */
function coreStreakMood(n: number): string {
  if (n >= CORE_STREAK_MOOD.euphoricMin) return "euphoric";
  if (n >= CORE_STREAK_MOOD.happyMin) return "happy";
  if (n >= CORE_STREAK_MOOD.hopefulMin) return "hopeful";
  return "sad";
}
function coreCompletedMood(n: number): string {
  if (n >= CORE_COMPLETED_MOOD.euphoricMin) return "euphoric";
  if (n >= CORE_COMPLETED_MOOD.happyMin) return "happy";
  if (n >= CORE_COMPLETED_MOOD.hopefulMin) return "hopeful";
  return "sad";
}

/** Swift ContributionCalendarLayout.intensityLevel */
function swiftIntensity(count: number): number {
  if (count <= 0) return 0;
  if (count === 1) return 1;
  if (count === 2) return 2;
  if (count <= 4) return 3;
  return 4;
}

describe("Swift thresholds against the core", () => {
  it.each([-1, 0, 1, 6, 7, 24, 25, 29, 30, 99, 100])("pins streak mood for %s", n => {
    expect(swiftStreakMood(n)).toBe(coreStreakMood(n));
  });

  it.each([-1, 0, 1, 6, 7, 24, 25, 29, 30, 99, 100])("pins completed mood for %s", n => {
    expect(swiftCompletedMood(n)).toBe(coreCompletedMood(n));
  });

  it.each([
    [0, 3], [1, 3], [3, 3], [4, 7], [7, 7], [14, 14], [15, 30], [30, 30], [31, 60], [100, 120],
  ])("pins Swift streak milestone %s → %s", (value, target) => {
    expect(streakTarget(value)).toBe(target);
  });

  it.each([
    [0, 7], [7, 7], [8, 25], [25, 25], [26, 50], [50, 50], [51, 100], [100, 100], [101, 200],
  ])("pins Swift completion milestone %s → %s", (value, target) => {
    expect(completionTarget(value)).toBe(target);
  });

  it.each([0, 1, 2, 3, 4, 5, 14])("pins Swift heatmap intensity %s onto heatmapLevel", count => {
    expect(heatmapLevel(count, HEATMAP_THRESHOLDS)).toBe(swiftIntensity(count));
  });
});

describe("reviewWindow", () => {
  const utcNoon = new Date("2026-10-04T12:00:00.000Z");

  it("uses a Monday-start week in UTC, matching MCP get_review", () => {
    const window = reviewWindow(utcNoon, "UTC", "week", 1);
    expect(window.from).toBe("2026-09-28T00:00:00.000Z");
    expect(window.to).toBe("2026-10-04T12:00:00.001Z");
  });

  it("uses a Sunday-start week when week start is Sunday", () => {
    const window = reviewWindow(utcNoon, "UTC", "week", 0);
    expect(window.from).toBe("2026-10-04T00:00:00.000Z");
  });

  it("starts the month and year at local midnight", () => {
    expect(reviewWindow(utcNoon, "UTC", "month", 1).from).toBe("2026-10-01T00:00:00.000Z");
    expect(reviewWindow(utcNoon, "UTC", "year", 1).from).toBe("2026-01-01T00:00:00.000Z");
  });

  it("buckets the civil day in America/Sao_Paulo, not UTC", () => {
    const late = new Date("2026-10-04T02:00:00.000Z");
    const window = reviewWindow(late, "America/Sao_Paulo", "week", 1);
    expect(window.from).toBe("2026-09-28T03:00:00.000Z");
    expect(reviewWindow(late, "America/Sao_Paulo", "month", 1).from).toBe("2026-10-01T03:00:00.000Z");
  });
});

describe("week start and heatmap padding", () => {
  it("defaults to Monday because Settings has no week-start field", () => {
    expect(weekStartFromSettings({})).toBe(1);
    expect(weekStartFromSettings({}, 7)).toBe(0);
  });

  it("pads complete weeks without inventing completions", () => {
    const days = [
      { date: "2026-09-29", completed: 2, morning: 2, afternoon: 0, evening: 0, night: 0, focusMinutes: 0 },
      { date: "2026-09-30", completed: 0, morning: 0, afternoon: 0, evening: 0, night: 0, focusMinutes: 0 },
    ];
    const monday = padContributionWeeks(days, 1);
    expect(monday[0]?.date).toBe("2026-09-28");
    expect(monday[0]?.inRange).toBe(false);
    expect(monday[0]?.completed).toBe(0);
    expect(monday[1]?.date).toBe("2026-09-29");
    expect(monday[1]?.inRange).toBe(true);
    expect(monday[1]?.value).toBe(2);
    expect(monday.length % 7).toBe(0);
    expect(weekdayLabels(1)[0]).toBe("Mon");
  });

  it("moves heatmap focus along GitHub-style columns", () => {
    expect(moveHeatmapIndex(0, "ArrowRight", 21)).toBe(7);
    expect(moveHeatmapIndex(8, "ArrowUp", 21)).toBe(7);
    expect(moveHeatmapIndex(3, "Home", 21)).toBe(0);
  });

  it("keeps heatmap labels honest for padding cells", () => {
    const [pad] = padContributionWeeks([{ date: "2026-09-29", completed: 1, morning: 1, afternoon: 0, evening: 0, night: 0, focusMinutes: 0 }], 1);
    expect(heatmapLabel(pad!)).toContain("outside the 12-month window");
  });

  it("sums core focus minutes into week buckets without changing the total", () => {
    const days = [
      { date: "2026-09-28", completed: 0, morning: 0, afternoon: 0, evening: 0, night: 0, focusMinutes: 10 },
      { date: "2026-09-29", completed: 0, morning: 0, afternoon: 0, evening: 0, night: 0, focusMinutes: 5 },
      { date: "2026-10-05", completed: 0, morning: 0, afternoon: 0, evening: 0, night: 0, focusMinutes: 7 },
    ];
    const buckets = focusWeekBuckets(days, 1);
    expect(buckets).toHaveLength(2);
    expect(buckets.reduce((sum, bucket) => sum + bucket.minutes, 0)).toBe(22);
  });
});

describe("core review fixtures (not recomputed)", () => {
  it("documents TestSwiftPeriodsAndStreak totals and period buckets", () => {
    const fixture = {
      completedTotal: 12,
      streak: 4,
      focusMinutes: 2,
      dayCount: 3,
      periods: { morning: 2, afternoon: 2, evening: 2, night: 4 } satisfies Record<ActivityPeriod, number>,
      contributionFirst: "2025-11-01",
      contributionLast: "2026-10-03",
    };
    expect(fixture.completedTotal).toBe(12);
    expect(coreStreakMood(fixture.streak)).toBe("hopeful");
    expect(coreCompletedMood(fixture.completedTotal)).toBe("hopeful");
    expect(fixture.periods.morning + fixture.periods.afternoon + fixture.periods.evening + fixture.periods.night).toBe(10);
  });
});
