import { vi } from "vitest";
import type { ReviewBridge, ReviewDayJSON, ReviewJSON, SettingsJSON } from "./bridge";

export const SETTINGS: SettingsJSON = {
  calendarWeekStart: "mon",
  appearanceTheme: "dark",
  notificationsSound: "glass",
  notificationsAllDayTime: "09:00",
  notificationsSnoozeMinutes: 10,
  focusWorkMin: 25,
  focusShortBreakMin: 5,
  focusLongBreakMin: 15,
  focusCycles: 4,
  focusAutoContinue: true,
  focusSounds: true,
  focusNotifications: true,
  mapOnline: false,
  mapProvider: "",
  agentsMcpPort: 47600,
  agentsMcpEnabled: true,
  agentsAllowFileAttach: false,
  appKeepRunningInBackground: true,
  linksFetchTitles: false,
};

/** Numbers from sparky/backend/domains/review TestSwiftPeriodsAndStreak (America/Sao_Paulo, 2026-10-01..03). */
export function saoPauloFixture(): ReviewJSON {
  const days: ReviewDayJSON[] = [
    { date: "2026-10-01", completed: 1, morning: 1, afternoon: 0, evening: 0, night: 0, focusMinutes: 2 },
    { date: "2026-10-02", completed: 1, morning: 1, afternoon: 0, evening: 0, night: 0, focusMinutes: 0 },
    { date: "2026-10-03", completed: 10, morning: 2, afternoon: 2, evening: 2, night: 4, focusMinutes: 0 },
  ];
  return {
    from: "2026-10-01T03:00:00.000Z",
    to: "2026-10-04T03:00:00.000Z",
    tz: "America/Sao_Paulo",
    streak: 4,
    completedTotal: 12,
    focusMinutes: 2,
    streakMood: "hopeful",
    completedMood: "hopeful",
    days,
    contribution: [
      { date: "2026-09-28", completed: 1, morning: 0, afternoon: 0, evening: 0, night: 1, focusMinutes: 0 },
      { date: "2026-09-29", completed: 0, morning: 0, afternoon: 0, evening: 0, night: 0, focusMinutes: 0 },
      { date: "2026-09-30", completed: 0, morning: 0, afternoon: 0, evening: 0, night: 0, focusMinutes: 0 },
      ...days,
    ],
  };
}

export function emptyReview(tz = "UTC"): ReviewJSON {
  return {
    from: "2026-09-28T00:00:00.000Z",
    to: "2026-10-04T12:00:00.001Z",
    tz,
    streak: 0,
    completedTotal: 0,
    focusMinutes: 0,
    streakMood: "sad",
    completedMood: "sad",
    days: [
      { date: "2026-09-28", completed: 0, morning: 0, afternoon: 0, evening: 0, night: 0, focusMinutes: 0 },
      { date: "2026-10-04", completed: 0, morning: 0, afternoon: 0, evening: 0, night: 0, focusMinutes: 0 },
    ],
    contribution: [{ date: "2025-11-01", completed: 0, morning: 0, afternoon: 0, evening: 0, night: 0, focusMinutes: 0 }],
  };
}

export function fakeBridge(options: { report?: ReviewJSON; settings?: SettingsJSON; fail?: "review" | "settings" | "unavailable" } = {}) {
  const report = structuredClone(options.report ?? saoPauloFixture());
  const settings = { ...SETTINGS, ...options.settings };
  let listener: Parameters<ReviewBridge["onChange"]>[0] | undefined;
  const bridge: ReviewBridge = {
    review: {
      get: vi.fn(async () => {
        if (options.fail === "review") throw new Error("Review failed.");
        if (options.fail === "unavailable") throw new Error("The local core is unreachable.");
        return structuredClone(report);
      }),
    },
    settings: {
      get: vi.fn(async () => {
        if (options.fail === "settings") throw new Error("Settings unavailable.");
        return { settings: { ...settings } };
      }),
      set: vi.fn(async () => ({ settings: { ...settings } })),
    },
    onChange: vi.fn(callback => { listener = callback; return () => { listener = undefined; }; }),
  };
  return {
    bridge,
    report,
    emit: () => listener?.({ resync: false, changes: [{ entity: "memory", id: "memory-1", op: "update" }] }),
  };
}
