import type { SparkyFeature } from "@polterware/feature-sparky-shell/feature";
import { CalendarPage } from "./calendar-page";

const feature: SparkyFeature = {
  id: "calendar",
  routes: [{ path: "/calendar", title: "Calendar", component: CalendarPage }],
  sidebar: [{ id: "calendar", label: "Calendar", path: "/calendar", icon: "CalendarClock" }],
  commands: [{
    id: "calendar.open",
    label: "Go to Calendar",
    keywords: ["calendar", "day", "week", "month", "schedule"],
    run: async () => {
      const wire = Reflect.get(globalThis, "sparky") as { app?: { openDeepLink?: (url: string) => Promise<void> } } | undefined;
      await wire?.app?.openDeepLink?.("sparky://calendar");
    },
  }],
};

export default feature;
export { CalendarPage };
export { createCalendarBridge, browserBridge } from "./bridge";
export { createCalendarStore, calendarStore } from "./store";
export { toOneShotSchedule, moveSeriesAnchor } from "./mapping";
export { visibleRange, dayHours, weekDates, monthDates } from "./grid";
