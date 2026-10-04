import { render } from "solid-js/web";
import { afterEach, describe, expect, it, vi } from "vitest";
import shellFeature, { SparkyShell, createShellBridge, type ShellWireBridge, type SparkyFeature } from "@polterware/feature-sparky-shell";
import { emptyReview, fakeBridge } from "./fake-bridge";
import { createReviewStore } from "./store";
import { ReviewPage } from "./review-page";
import reviewFeature from "./index";

let dispose: (() => void) | undefined;
afterEach(() => {
  dispose?.();
  dispose = undefined;
  document.body.replaceChildren();
  vi.restoreAllMocks();
});

function mount(options: { empty?: boolean; fail?: "review" | "unavailable" } = {}) {
  localStorage.setItem("sparky.shell.onboarding", "complete");
  vi.spyOn(HTMLElement.prototype, "getClientRects").mockImplementation(function (this: HTMLElement) {
    return this.isConnected && !this.closest("[hidden]")
      ? ([new DOMRect(0, 0, 100, 32)] as unknown as DOMRectList)
      : ([] as unknown as DOMRectList);
  });
  const fake = fakeBridge({ report: options.empty ? emptyReview() : undefined, fail: options.fail });
  const store = createReviewStore(() => fake.bridge, { now: () => Date.parse("2026-10-04T12:00:00.000Z"), tz: "UTC" });
  const extra: SparkyFeature = { ...reviewFeature, routes: [{ path: "/review", title: "Review", component: () => <ReviewPage store={store} /> }] };
  const wire = {
    ...fake.bridge,
    minds: { list: async () => ({ minds: [] }) },
    memories: { list: async () => ({ memories: [], total: 0 }), get: async () => ({}) },
    tags: { list: async () => ({ tags: [] }) },
    reminders: { status: async () => ({ planned: 0, armed: 0, permission: "authorized" }) },
    search: { search: async () => ({ hits: [] }) },
    app: { onDeepLink: () => () => {}, getDeepLink: async () => null, requestNotificationAuthorization: async () => true },
  };
  const host = document.createElement("div");
  document.body.append(host);
  dispose = render(() => <SparkyShell features={[shellFeature, extra]} bridge={createShellBridge(wire as unknown as ShellWireBridge)} />, host);
  return { host, store, fake };
}

async function openReview(host: HTMLElement) {
  await vi.waitFor(() => expect(host.textContent).toContain("Today is unavailable"));
  const nav = await vi.waitFor(() => {
    const button = [...host.querySelectorAll("button")].find(element => element.textContent?.trim() === "Review");
    expect(button).not.toBeUndefined();
    return button!;
  });
  nav.click();
}

describe("Review route", () => {
  it("renders core streak, completions, heatmap, and chart tables", async () => {
    const { host, store } = mount();
    await openReview(host);
    await vi.waitFor(() => expect(store.loadState()).toBe("live"));
    await vi.waitFor(() => expect(document.body.textContent).toContain("Day streak"));
    expect(document.body.textContent).toContain("12");
    expect(document.body.textContent).toContain("4");
    expect(document.body.textContent).toContain("4/7 days");
    expect(document.body.textContent).toContain("12/25 completed");
    expect(document.body.querySelector('[aria-label="12-month contribution calendar"]')).not.toBeNull();
    expect(document.body.querySelector('[aria-label="Weekly activity as a table"]')).not.toBeNull();
    expect(document.body.querySelector('[aria-label="Focus minutes as a table"]')).not.toBeNull();
    expect(document.body.textContent).toContain("Morning 2");
  });

  it("moves heatmap focus with the keyboard and updates the details", async () => {
    const { host, store } = mount();
    await openReview(host);
    await vi.waitFor(() => expect(store.loadState()).toBe("live"));
    const heatmap = await vi.waitFor(() => {
      const node = document.body.querySelector<HTMLElement>(".sparky-review-heatmap");
      expect(node).not.toBeNull();
      return node!;
    });
    heatmap.focus();
    heatmap.dispatchEvent(new KeyboardEvent("keydown", { key: "Home", bubbles: true, cancelable: true }));
    await vi.waitFor(() => expect(document.body.querySelector('[data-review-focus="true"]')).not.toBeNull());
    expect(document.body.textContent).toContain("September 28, 2026");
  });

  it("shows the first-run empty state when the core returns zeros", async () => {
    const { host, store } = mount({ empty: true });
    await openReview(host);
    await vi.waitFor(() => expect(store.loadState()).toBe("live"));
    await vi.waitFor(() => expect(document.body.textContent).toContain("No completions yet"));
    expect(document.body.textContent).toContain("Complete a memory to start a streak");
  });

  it("shows an honest unavailable state", async () => {
    const { host, store } = mount({ fail: "unavailable" });
    await openReview(host);
    await vi.waitFor(() => expect(store.loadState()).toBe("unavailable"));
    expect(document.body.textContent).toContain("Review unavailable");
  });

  it("registers the Review route", () => {
    expect(reviewFeature.routes[0]?.path).toBe("/review");
    expect(reviewFeature.sidebar?.[0]?.path).toBe("/review");
  });
});
