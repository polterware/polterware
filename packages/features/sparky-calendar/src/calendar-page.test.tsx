import { render } from "solid-js/web";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import shellFeature, { SparkyShell, createShellBridge, useSparkyShell, type ShellWireBridge, type SparkyFeature } from "@polterware/feature-sparky-shell";
import { Button } from "@polterware/frontend-design-system";
import { createCalendarBridge } from "./bridge";
import calendarFeature from "./index";
import { fakeCalendarBridge } from "./fake-bridge";
import { createCalendarStore } from "./store";
import { CalendarPage } from "./calendar-page";

let dispose: (() => void) | undefined;
beforeEach(() => {
  localStorage.clear();
  localStorage.setItem("sparky.shell.onboarding", "complete");
  vi.spyOn(HTMLElement.prototype, "getClientRects").mockImplementation(function (this: HTMLElement) {
    return this.isConnected && !this.closest("[hidden]")
      ? ([new DOMRect(0, 0, 100, 32)] as unknown as DOMRectList)
      : ([] as unknown as DOMRectList);
  });
});
afterEach(() => {
  dispose?.();
  dispose = undefined;
  Reflect.deleteProperty(globalThis, "sparky");
  document.body.replaceChildren();
  vi.restoreAllMocks();
});

function OpenCalendar(props: { store: ReturnType<typeof createCalendarStore> }) {
  const shell = useSparkyShell();
  return <>
    <Button onClick={() => shell.navigate("/calendar")}>Open calendar</Button>
    <CalendarPage store={props.store} />
  </>;
}

function mount(seed?: Parameters<typeof fakeCalendarBridge>[0]) {
  const fake = fakeCalendarBridge(seed);
  const store = createCalendarStore(() => createCalendarBridge(fake.wire));
  Reflect.set(globalThis, "sparky", {
    ...fake.wire,
    reminders: { status: async () => ({ permission: "authorized", armed: 0, planned: 0, notifierLastSeenAt: new Date().toISOString() }) },
    search: { search: async () => ({ hits: [] }) },
    app: { onDeepLink: () => () => {}, getDeepLink: async () => null, openDeepLink: async () => {} },
  });
  const opener: SparkyFeature = {
    id: "calendar-test",
    routes: [{ path: "/today", title: "Today", component: () => <OpenCalendar store={store} /> }],
  };
  const host = document.createElement("div");
  document.body.append(host);
  dispose = render(() => (
    <SparkyShell
      features={[shellFeature, opener, calendarFeature]}
      bridge={createShellBridge(Reflect.get(globalThis, "sparky") as ShellWireBridge)}
    />
  ), host);
  return { host, fake, store };
}

function pointerClick(element: HTMLElement) {
  const opts = { bubbles: true, cancelable: true, pointerId: 1, pointerType: "mouse", button: 0, buttons: 1 } as const;
  element.dispatchEvent(new PointerEvent("pointerdown", opts));
  element.dispatchEvent(new MouseEvent("mousedown", opts));
  element.dispatchEvent(new PointerEvent("pointerup", { ...opts, buttons: 0 }));
  element.dispatchEvent(new MouseEvent("mouseup", { ...opts, buttons: 0 }));
  element.click();
}

describe("calendar page through a fake preload bridge", () => {
  it("creates a memory from an empty slot and opens the detail panel", async () => {
    const { host, fake } = mount();
    const slot = await vi.waitFor(() => {
      const found = host.querySelector<HTMLButtonElement>("[data-empty-slot]");
      expect(found).not.toBeNull();
      return found!;
    });
    pointerClick(slot);
    await vi.waitFor(() => expect(fake.wire.memories.create).toHaveBeenCalledWith({ title: "Untitled memory" }));
    await vi.waitFor(() => expect(fake.schedule.set).toHaveBeenCalled());
    await vi.waitFor(() => expect(document.body.textContent).toContain("Untitled memory"));
  });

  it("opens a pill in the global DetailPanel", async () => {
    const occurrenceAt = new Date().toISOString();
    const { host } = mount({
      occurrences: [{
        memoryId: "memory-1",
        title: "Read the brief",
        occurrenceAt,
        allDay: true,
        tz: Intl.DateTimeFormat().resolvedOptions().timeZone || "UTC",
      }],
    });
    const pill = await vi.waitFor(() => {
      const found = [...host.querySelectorAll<HTMLButtonElement>("[data-occurrence='memory-1']")].find(item => item.textContent?.includes("Read the brief"));
      expect(found).toBeDefined();
      return found!;
    });
    pointerClick(pill);
    await vi.waitFor(() => expect(document.body.textContent).toContain("Read the brief"));
  });

  it("moves keyboard focus across days and keeps month fetches on the visible range", async () => {
    const { host, fake } = mount();
    const root = await vi.waitFor(() => {
      const found = host.querySelector<HTMLElement>("[data-calendar]");
      expect(found).not.toBeNull();
      return found!;
    });
    await vi.waitFor(() => expect(root.getAttribute("data-calendar-from")).toBeTruthy());
    root.focus();
    root.dispatchEvent(new KeyboardEvent("keydown", { key: "m", bubbles: true, cancelable: true }));
    await vi.waitFor(() => expect(root.getAttribute("data-calendar-view")).toBe("month"));
    const from = root.getAttribute("data-calendar-from");
    const to = root.getAttribute("data-calendar-to");
    expect(from).toBeTruthy();
    expect(to).toBeTruthy();
    const calls = fake.schedule.listOccurrences.mock.calls.map(call => call[0] as { from: string; to: string });
    expect(calls.some(call => call.from === from && call.to === to)).toBe(true);
    root.focus();
    root.dispatchEvent(new KeyboardEvent("keydown", { key: "ArrowRight", bubbles: true, cancelable: true }));
    await vi.waitFor(() => expect(host.querySelector("[data-focused]")).not.toBeNull());
  });

  it("asks to move the whole series when a recurring pill is dropped", async () => {
    const occurrenceAt = new Date().toISOString();
    const { host, fake, store } = mount({
      memories: [{
        id: "memory-1",
        title: "Standup",
        rev: "3",
        schedule: { memoryId: "memory-1", fireAt: occurrenceAt, tz: "UTC", freq: "daily", every: 1 },
      }],
      occurrences: [{ memoryId: "memory-1", title: "Standup", occurrenceAt, tz: "UTC" }],
    });
    await vi.waitFor(() => expect(store.occurrences().length).toBeGreaterThan(0));
    const occurrence = store.occurrences()[0]!;
    const slot = await vi.waitFor(() => {
      const found = host.querySelector<HTMLElement>("[data-slot-time='15:00']");
      expect(found).not.toBeNull();
      return found!;
    });
    const transfer = { getData: () => JSON.stringify(occurrence), setData: () => undefined };
    const drop = new Event("drop", { bubbles: true, cancelable: true });
    Object.defineProperty(drop, "dataTransfer", { value: transfer });
    slot.dispatchEvent(drop);
    await vi.waitFor(() => expect(document.body.textContent).toContain("This occurrence only is not supported in v1"));
    expect(fake.schedule.set).not.toHaveBeenCalled();
    const confirm = [...document.querySelectorAll("button")].find(item => item.textContent?.trim() === "Move the series");
    expect(confirm).toBeDefined();
    pointerClick(confirm!);
    await vi.waitFor(() => expect(fake.schedule.set).toHaveBeenCalled());
  });
});
