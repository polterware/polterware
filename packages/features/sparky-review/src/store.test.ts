import { afterEach, describe, expect, it, vi } from "vitest";
import { fakeBridge } from "./fake-bridge";
import { createReviewStore } from "./store";

const cleanups: Array<() => void> = [];
afterEach(() => { cleanups.splice(0).reverse().forEach(run => run()); });

function mount(fail?: "review" | "unavailable") {
  const fake = fakeBridge({ fail });
  const store = createReviewStore(() => fake.bridge, { now: () => Date.parse("2026-10-04T12:00:00.000Z"), tz: "UTC" });
  cleanups.push(() => store.dispose());
  return { fake, store };
}

describe("review store", () => {
  it("loads ReviewService.Get with a timezone-correct week window and does not invent totals", async () => {
    const { fake, store } = mount();
    store.connect();
    await vi.waitFor(() => expect(store.loadState()).toBe("live"));
    expect(fake.bridge.settings.get).toHaveBeenCalled();
    expect(fake.bridge.review.get).toHaveBeenCalledWith({
      from: "2026-09-28T00:00:00.000Z",
      to: "2026-10-04T12:00:00.001Z",
      tz: "UTC",
    });
    expect(store.report()?.streak).toBe(4);
    expect(store.report()?.completedTotal).toBe(12);
    expect(store.report()?.focusMinutes).toBe(2);
    expect(store.report()?.streakMood).toBe("hopeful");
    expect(store.weekStart()).toBe(1);
  });

  it("reloads when the range changes", async () => {
    const { fake, store } = mount();
    store.connect();
    await vi.waitFor(() => expect(store.loadState()).toBe("live"));
    store.setRange("month");
    await vi.waitFor(() => expect(fake.bridge.review.get).toHaveBeenCalledTimes(2));
    expect(fake.bridge.review.get).toHaveBeenLastCalledWith({
      from: "2026-10-01T00:00:00.000Z",
      to: "2026-10-04T12:00:00.001Z",
      tz: "UTC",
    });
  });

  it("marks the core unreachable as unavailable", async () => {
    const { store } = mount("unavailable");
    store.connect();
    await vi.waitFor(() => expect(store.loadState()).toBe("unavailable"));
    expect(store.loadError()).toContain("unreachable");
  });
  it("uses Sunday from SettingsService for the review window", async () => {
    const fake = fakeBridge({ settings: { calendarWeekStart: "sun" } });
    const store = createReviewStore(() => fake.bridge, { now: () => Date.parse("2026-10-04T12:00:00.000Z"), tz: "UTC" });
    cleanups.push(() => store.dispose());
    store.connect();
    await vi.waitFor(() => expect(store.loadState()).toBe("live"));
    expect(store.weekStart()).toBe(0);
    expect(fake.bridge.review.get).toHaveBeenCalledWith({ from: "2026-10-04T00:00:00.000Z", to: "2026-10-04T12:00:00.001Z", tz: "UTC" });
  });

});
