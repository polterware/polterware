import { onCleanup, onMount } from "solid-js";
import { render } from "solid-js/web";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import shellFeature, { createShellBridge, SparkyShell, useSparkyShell, type ShellWireBridge, type SparkyFeature } from "./index";
import { Button, DetailPanel } from "@polterware/frontend-design-system";

let dispose: (() => void) | undefined;
beforeEach(() => {
  localStorage.clear(); localStorage.setItem("sparky.shell.onboarding", "complete");
  // jsdom has no layout; Ark must see connected controls as visible to model browser focus.
  vi.spyOn(HTMLElement.prototype, "getClientRects").mockImplementation(function (this: HTMLElement) {
    return (this.isConnected && !this.closest("[hidden]")) ? [new DOMRect(0, 0, 100, 32)] as unknown as DOMRectList : [] as unknown as DOMRectList;
  });
});
afterEach(() => { dispose?.(); dispose = undefined; document.body.replaceChildren(); vi.restoreAllMocks(); });
function mount(extra: SparkyFeature[] = [], options: { offline?: boolean } = {}) {
  const status = vi.fn(async () => {
    if (options.offline) throw new Error("Core disconnected");
    return { permission: "authorized", armed: 0, planned: 0, notifierLastSeenAt: new Date().toISOString() };
  });
  const create = vi.fn(async (_input?: unknown) => ({ memory: { id: "new-memory", title: "Buy milk", rev: "1" } }));
  let onLink!: (link: { url: string; route: string }) => void;
  const unsubscribe = vi.fn();
  const wire = {
    reminders: { status }, minds: { list: async () => ({ minds: [] }) },
    memories: { list: async () => ({ total: 0 }), create, get: async () => ({ memory: { id: "new-memory", title: "Buy milk", note: "Local note", rev: "1" } }) },
    tags: { list: async () => ({ tags: [] }) }, search: { search: async () => ({ hits: [{ memoryId: "new-memory", title: "Buy milk", snippet: "Local note" }] }) },
    onChange: () => unsubscribe,
    app: { onDeepLink: (callback: typeof onLink) => { onLink = callback; return unsubscribe; }, getDeepLink: async () => null },
  } as unknown as ShellWireBridge;
  const host = document.createElement("div"); document.body.append(host);
  dispose = render(() => <SparkyShell features={[shellFeature, ...extra]} bridge={createShellBridge(wire)} />, host);
  return { host, status, create, onLink: (url: string) => onLink({ url, route: "" }), unsubscribe };
}
function button(label: string): HTMLButtonElement {
  const found = Array.from(document.querySelectorAll<HTMLButtonElement>("button")).find(item => item.getAttribute("aria-label") === label || item.textContent?.trim() === label);
  if (!found) throw new Error(`Button ${label} missing: ${document.body.textContent}`);
  return found;
}
function key(key: string, metaKey = true) { document.dispatchEvent(new KeyboardEvent("keydown", { key, metaKey, bubbles: true, cancelable: true })); }
function fill(value: string) {
  const input = document.querySelector<HTMLInputElement>("#sparky-command-input")!;
  input.value = value; input.dispatchEvent(new Event("input", { bubbles: true })); return input;
}
describe("Sparky shell behavior", () => {
  it("renders every primary registry stub, landmarks and active navigation", async () => {
    const { host } = mount();
    await vi.waitFor(() => expect(host.textContent).toContain("Today is unavailable"));
    expect(host.querySelector("header")).not.toBeNull();
    expect(host.querySelector('nav[aria-label="Main navigation"]')).not.toBeNull();
    expect(host.querySelector("main")).not.toBeNull();
    for (const title of ["Inbox", "Today", "Calendar", "Minds", "Map", "Focus", "Review"]) {
      button(title).click();
      await vi.waitFor(() => expect(host.textContent).toContain(`${title} is unavailable`));
      expect(host.querySelector('[aria-current="page"]')?.textContent).toContain(title);
      await vi.waitFor(() => expect(document.activeElement?.tagName).toBe("MAIN"));
    }
  });
  it("renders registered routes and runs registered Commander actions", async () => {
    const run = vi.fn();
    const { host } = mount([{ id: "calendar", routes: [{ path: "/calendar", title: "Calendar", component: () => <p>Live calendar</p> }], commands: [{ id: "custom", label: "Custom action", keywords: ["favorite"], run }] }]);
    await vi.waitFor(() => expect(host.textContent).toContain("Today is unavailable"));
    key("3"); await vi.waitFor(() => expect(host.textContent).toContain("Live calendar"));
    key("k"); await vi.waitFor(() => expect(document.querySelector('[role="dialog"]')).not.toBeNull());
    await vi.waitFor(() => expect(document.activeElement?.id).toBe("sparky-command-input"));
    fill("favorite"); button("Custom action").click();
    await vi.waitFor(() => expect(run).toHaveBeenCalledOnce());
    await vi.waitFor(() => expect(document.querySelector('[role="dialog"]')).toBeNull());
  });
  it("recovers from offline via reconnect UI", async () => {
    const options = { offline: true }; const { host, status } = mount([], options);
    await vi.waitFor(() => expect(host.textContent).toContain("Sparky cannot reach the local core"));
    options.offline = false; button("Retry connection").click();
    await vi.waitFor(() => expect(host.textContent).toContain("Today is unavailable"));
    expect(status).toHaveBeenCalledTimes(2);
  });
  it("navigates history and opens Settings by keyboard", async () => {
    const { host } = mount(); await vi.waitFor(() => expect(host.textContent).toContain("Today is unavailable"));
    key("1"); await vi.waitFor(() => expect(host.textContent).toContain("Inbox is unavailable"));
    key("["); await vi.waitFor(() => expect(host.textContent).toContain("Today is unavailable"));
    key("]"); await vi.waitFor(() => expect(host.textContent).toContain("Inbox is unavailable"));
    key(","); await vi.waitFor(() => expect(document.body.textContent).toContain("Settings unavailable"));
  });
  it("creates through Commander and opens global memory detail", async () => {
    const { host, create } = mount(); await vi.waitFor(() => expect(host.textContent).toContain("Today is unavailable"));
    key("k"); await vi.waitFor(() => expect(document.querySelector("#sparky-command-input")).not.toBeNull());
    fill("new memory"); button("New memory").click();
    await vi.waitFor(() => expect(document.querySelector('[role="dialog"]')?.textContent).toContain("New memory"));
    fill("Buy milk tomorrow 9am !!");
    await vi.waitFor(() => expect(button("Create memory").disabled).toBe(false));
    button("Create memory").click();
    await vi.waitFor(() => expect(create).toHaveBeenCalledOnce());
    expect(create.mock.calls[0]?.[0]).toMatchObject({ title: "Buy milk", priority: 2 });
    await vi.waitFor(() => expect(host.textContent).toContain("Local note"));
    expect(document.querySelector('[role="dialog"]')).toBeNull();
    key("j"); await vi.waitFor(() => expect(host.textContent).not.toContain("Local note"));
  });
  it("opens memory search results and handles a warm deep link", async () => {
    const { host, onLink } = mount(); await vi.waitFor(() => expect(host.textContent).toContain("Today is unavailable"));
    onLink("sparky://calendar?date=2026-10-04"); await vi.waitFor(() => expect(host.textContent).toContain("Calendar is unavailable"));
    key("k"); await vi.waitFor(() => expect(document.querySelector("#sparky-command-input")).not.toBeNull());
    fill("milk"); await vi.waitFor(() => expect(document.body.textContent).toContain("Buy milk"));
    button("Buy milkLocal note").click(); await vi.waitFor(() => expect(host.textContent).toContain("Local note"));
  });
  it("exposes detail sections and parameterized routes without feature imports", async () => {
    const Detail = () => { const shell = useSparkyShell(); return <DetailPanel.Body><DetailPanel.Main>Editor {shell.selectedMemoryId()}</DetailPanel.Main></DetailPanel.Body>; };
    const Page = () => { const shell = useSparkyShell(); return <Button onClick={() => shell.openMemory("selected")}>Open selected</Button>; };
    const { host } = mount([{ id: "memories", routes: [{ path: "/today", title: "Today", component: Page }, { path: "/memory/:id", title: "Memory", component: Detail }] }]);
    await vi.waitFor(() => expect(host.textContent).toContain("Open selected"));
    button("Open selected").click(); await vi.waitFor(() => expect(host.textContent).toContain("Editor selected"));
    key("j"); await vi.waitFor(() => expect(host.textContent).not.toContain("Editor selected"));
  });
  it("cleans up bridge subscriptions when unmounted", async () => {
    const { host, unsubscribe } = mount(); await vi.waitFor(() => expect(host.textContent).toContain("Today is unavailable"));
    dispose?.(); dispose = undefined; expect(unsubscribe).toHaveBeenCalledTimes(2);
  });
});

it("delegates collection keyboard actions and unregisters them on navigation", async () => {
  const action = vi.fn();
  const Page = () => {
    const shell = useSparkyShell();
    onMount(() => { const unregister = shell.registerCollectionShortcuts(action); onCleanup(unregister); });
    return <p>Keyboard collection</p>;
  };
  const { host } = mount([{ id: "memories", routes: [{ path: "/today", title: "Today", component: Page }] }]);
  await vi.waitFor(() => expect(host.textContent).toContain("Keyboard collection"));
  for (const shortcut of ["Enter", " ", "e", "p"]) key(shortcut, false);
  key("Backspace");
  expect(action.mock.calls.map(call => call[0])).toEqual(["open", "toggle-done", "edit-title", "pin", "delete"]);
  key("1"); await vi.waitFor(() => expect(host.textContent).toContain("Inbox is unavailable"));
  key("Enter", false); expect(action).toHaveBeenCalledTimes(5);
});
it("contains a feature render failure and permits navigation away", async () => {
  const { host } = mount([{ id: "calendar", routes: [{ path: "/calendar", title: "Calendar", component: () => { throw new Error("Calendar render failed"); } }] }]);
  await vi.waitFor(() => expect(host.textContent).toContain("Today is unavailable"));
  key("3"); await vi.waitFor(() => expect(host.querySelector('[role="alert"]')?.textContent).toContain("Calendar render failed"));
  key("1"); await vi.waitFor(() => expect(host.textContent).toContain("Inbox is unavailable"));
});
it("keeps failed registered actions in Commander with honest error text", async () => {
  const { host } = mount([{ id: "actions", routes: [], commands: [{ id: "fail", label: "Fail action", run: async () => { throw new Error("Action failed"); } }] }]);
  await vi.waitFor(() => expect(host.textContent).toContain("Today is unavailable"));
  key("k"); await vi.waitFor(() => expect(document.querySelector("#sparky-command-input")).not.toBeNull());
  fill("fail action"); button("Fail action").click();
  await vi.waitFor(() => expect(document.querySelector('[role="dialog"] [role="alert"]')?.textContent).toContain("Action failed"));
});
it("shows skippable onboarding and honest authorization-unavailable state", async () => {
  localStorage.removeItem("sparky.shell.onboarding");
  mount(); await vi.waitFor(() => expect(document.querySelector('[role="dialog"]')?.textContent).toContain("Welcome to Sparky"));
  button("Continue").click();
  await vi.waitFor(() => expect(document.querySelector('[role="dialog"]')?.textContent).toContain("Notification authorization is unavailable"));
  button("Skip onboarding").click();
  await vi.waitFor(() => expect(document.querySelector('[role="dialog"]')).toBeNull());
  expect(localStorage.getItem("sparky.shell.onboarding")).toBe("complete");
});
