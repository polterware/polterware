import { createSignal } from "solid-js";
import { browserBridge, type ReviewBridge, type ReviewJSON } from "./bridge";
import {
  honestyLabel,
  isEmptyReview,
  resolveTimezone,
  reviewWindow,
  weekStartFromSettings,
  type Honesty,
  type ReviewRange,
  type WeekStart,
} from "./model";

export function createReviewStore(getBridge: () => ReviewBridge = browserBridge, options: { now?: () => number; tz?: string } = {}) {
  const [range, setRange] = createSignal<ReviewRange>("week");
  const [weekStart, setWeekStart] = createSignal<WeekStart>(1);
  const [report, setReport] = createSignal<ReviewJSON | null>(null);
  const [loadState, setLoadState] = createSignal<Honesty>("queued");
  const [loadError, setLoadError] = createSignal("");
  const [tz, setTz] = createSignal(resolveTimezone(options.tz));
  let unsubscribe: (() => void) | undefined;
  let users = 0;
  let io = Promise.resolve();

  const now = () => options.now?.() ?? Date.now();

  async function load(): Promise<void> {
    setLoadState("queued");
    setLoadError("");
    try {
      const settings = await getBridge().settings.get({}).catch(() => ({ settings: undefined }));
      const start = weekStartFromSettings(settings.settings);
      setWeekStart(start);
      const zone = resolveTimezone(options.tz);
      setTz(zone);
      const window = reviewWindow(new Date(now()), zone, range(), start);
      const next = await getBridge().review.get({ from: window.from, to: window.to, tz: zone });
      setReport(next);
      setLoadState("live");
    } catch (error) {
      const message = error instanceof Error ? error.message : "Review is unavailable.";
      setLoadError(message);
      setReport(null);
      setLoadState(message.toLowerCase().includes("unreachable") ? "unavailable" : "failed");
    }
  }

  const store = {
    range, weekStart, report, loadState, loadError, tz,
    empty: () => isEmptyReview(report()),
    statusLabel: () => honestyLabel(loadState()),
    setRange(next: ReviewRange) {
      if (next === range()) return;
      setRange(next);
      void load();
    },
    connect() {
      users += 1;
      if (users === 1) {
        try {
          unsubscribe = getBridge().onChange(event => {
            if (event.resync || event.changes.some(change => change.entity === "memory" || change.entity === "focus" || change.entity === "settings")) {
              io = io.catch(() => undefined).then(load);
            }
          });
        } catch {
          setLoadState("unavailable");
          setLoadError("Sparky bridge unavailable. Open this page in Sparky Desktop.");
        }
        void load();
      }
      return () => {
        users = Math.max(0, users - 1);
        if (users === 0) { unsubscribe?.(); unsubscribe = undefined; }
      };
    },
    reload: () => load(),
    flush: () => io,
    dispose() { unsubscribe?.(); unsubscribe = undefined; users = 0; },
  };
  return store;
}

export const reviewStore = createReviewStore();
export type ReviewStore = ReturnType<typeof createReviewStore>;
