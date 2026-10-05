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
function mount(extra: SparkyFeature[] = [], options: { offline?: boolean; permission?: string; armed?: number; authorize?: false | (() => Promise<boolean>); holdStatus?: boolean } = {}) {
  let permission = options.permission ?? "authorized";
  let releaseStatus: ((value: unknown) => void) | undefined;
  const status = vi.fn(async () => {
    if (options.holdStatus) await new Promise(resolve => { releaseStatus = resolve; });
    if (options.offline) throw new Error("Core disconnected");
    return { permission, armed: options.armed ?? 0, planned: 0, notifierLastSeenAt: new Date().toISOString() };
  });
  const create = vi.fn(async (_input?: unknown) => ({ memory: { id: "new-memory", title: "Buy milk", rev: "1" } }));
  const listMinds = vi.fn(async () => ({ minds: [] }));
  const listMemories = vi.fn(async () => ({ total: 0 }));
  const requestNotificationAuthorization = vi.fn(options.authorize === false ? undefined : (options.authorize ?? (async () => { permission = "authorized"; return true; })));
  let onLink!: (link: { url: string; route: string }) => void;
  const unsubscribe = vi.fn();
  const wire = {
    reminders: { status }, minds: { list: listMinds },
    memories: { list: listMemories, create, get: async () => ({ memory: { id: "new-memory", title: "Buy milk", note: "Local note", rev: "1" } }) },
    tags: { list: async () => ({ tags: [] }) }, search: { search: vi.fn(async () => ({ hits: [{ memoryId: "new-memory", title: "Buy milk", snippet: "Local <b>note</b>" }] })) },
    onChange: () => unsubscribe,
    app: {
      onDeepLink: (callback: typeof onLink) => { onLink = callback; return unsubscribe; },
      getDeepLink: async () => null,
      ...(options.authorize === false ? {} : { requestNotificationAuthorization }),
    },
  } as unknown as ShellWireBridge;
  const host = document.createElement("div"); document.body.append(host);
  dispose = render(() => <SparkyShell features={[shellFeature, ...extra]} bridge={createShellBridge(wire)} />, host);
  return { host, status, create, listMinds, listMemories, releaseStatus: () => releaseStatus?.({ permission, armed: options.armed ?? 0, planned: 0, notifierLastSeenAt: new Date().toISOString() }), onLink: (url: string) => onLink({ url, route: "" }), unsubscribe, requestNotificationAuthorization, search: wire.search.search };
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
    for (const title of ["Inbox", "Calendar", "Minds", "Map", "Focus"]) {
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
    fill("new memory");
    [...document.querySelectorAll<HTMLButtonElement>('[role="dialog"] button')].find(item => item.textContent?.trim() === "New memory")!.click();
    const title = await vi.waitFor(() => {
      const field = document.querySelector<HTMLInputElement>("#sparky-memory-title");
      expect(field).not.toBeNull();
      return field!;
    });
    title.value = "Buy milk tomorrow 9am !!"; title.dispatchEvent(new Event("input", { bubbles: true }));
    await vi.waitFor(() => expect(button("Create memory").disabled).toBe(false));
    button("Create memory").click();
    await vi.waitFor(() => expect(create).toHaveBeenCalledOnce());
    expect(create.mock.calls[0]?.[0]).toMatchObject({ title: "Buy milk", priority: 2 });
    await vi.waitFor(() => expect(host.textContent).toContain("Local note"));
    expect(document.querySelector('[role="dialog"]')).toBeNull();
    key("j"); await vi.waitFor(() => expect(host.textContent).not.toContain("Local note"));
  });
  it("routes Start focus and memory-bound focus links to the Focus page", async () => {
    const onDeepLink = vi.fn((url: URL) => url.searchParams.get("action") === "focus");
    const { host, onLink } = mount([
      { id: "focus", routes: [{ path: "/focus", title: "Focus", component: () => <p>Live focus</p> }], onDeepLink },
    ]);
    await vi.waitFor(() => expect(host.textContent).toContain("Today is unavailable"));
    onLink("sparky://focus");
    await vi.waitFor(() => expect(host.textContent).toContain("Live focus"));
    onLink("sparky://memory/01K6N3QWJ8P9A7B6C5D4E3F2G1?action=focus");
    await vi.waitFor(() => expect(host.textContent).toContain("Live focus"));
    expect(onDeepLink).toHaveBeenCalled();
    await vi.waitFor(() => expect(host.textContent).toContain("Local note"));
  });
  it("opens memory search results and handles a warm deep link", async () => {
    const { host, onLink } = mount(); await vi.waitFor(() => expect(host.textContent).toContain("Today is unavailable"));
    onLink("sparky://calendar?date=2026-10-04"); await vi.waitFor(() => expect(host.textContent).toContain("Calendar is unavailable"));
    key("k"); await vi.waitFor(() => expect(document.querySelector("#sparky-command-input")).not.toBeNull());
    fill("milk"); await vi.waitFor(() => expect(document.body.textContent).toContain("Buy milk"));
    expect(document.body.textContent).not.toContain("<b>");
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
it("leaves native controls, editable ancestors and overlays out of collection shortcuts", async () => {
  const action = vi.fn();
  const Page = () => {
    const shell = useSparkyShell();
    onMount(() => { onCleanup(shell.registerCollectionShortcuts(action)); });
    return <div data-keyboard-fixture><button><span>Native button</span></button><a href="#">Link</a><input /><div contenteditable="true"><span>Editable</span></div><div role="dialog"><span>Dialog</span></div><div role="menu"><span>Menu</span></div><div role="listbox"><span>Option</span></div><div role="row" tabindex="0">Row</div><p>Other content</p></div>;
  };
  const { host } = mount([{ id: "memories", routes: [{ path: "/today", title: "Today", component: Page }] }]);
  await vi.waitFor(() => expect(host.querySelector("[data-keyboard-fixture]")).not.toBeNull());
  host.querySelector("[contenteditable]")!.setAttribute("contenteditable", "");
  for (const target of host.querySelectorAll('[data-keyboard-fixture] button span, a, input, [contenteditable] span, [role="dialog"] span, [role="menu"] span, [role="listbox"] span, [data-keyboard-fixture] p')) {
    for (const value of ["Enter", " ", "e", "p"]) {
      const event = new KeyboardEvent("keydown", { key: value, bubbles: true, cancelable: true });
      target.dispatchEvent(event);
      expect(event.defaultPrevented).toBe(false);
    }
  }
  expect(action).not.toHaveBeenCalled();
  const row = host.querySelector('[role="row"]')!;
  const handled = new KeyboardEvent("keydown", { key: "Enter", bubbles: true, cancelable: true });
  row.dispatchEvent(handled);
  expect(handled.defaultPrevented).toBe(true);
  expect(action).toHaveBeenCalledWith("open");
  const prevented = new KeyboardEvent("keydown", { key: " ", bubbles: true, cancelable: true });
  prevented.preventDefault(); row.dispatchEvent(prevented);
  expect(action).toHaveBeenCalledOnce();
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
it("shows skippable onboarding without requesting notification authorization on mount", async () => {
  localStorage.removeItem("sparky.shell.onboarding");
  const { requestNotificationAuthorization } = mount([], { permission: "not-determined" });
  await vi.waitFor(() => expect(document.querySelector('[role="dialog"]')?.textContent).toContain("Welcome to Sparky"));
  expect(requestNotificationAuthorization).not.toHaveBeenCalled();
  button("Continue").click();
  await vi.waitFor(() => expect(document.querySelector('[role="dialog"]')?.textContent).toContain("Reminders: permission required"));
  expect(document.querySelector('[role="dialog"]')?.textContent).toContain("Enable notifications");
  expect(document.querySelector('[role="dialog"]')?.textContent).toContain("does not arm a reminder");
  expect(document.querySelector('[role="dialog"]')?.textContent).not.toMatch(/Reminders: \d+ armed/);
  button("Skip onboarding").click();
  await vi.waitFor(() => expect(document.querySelector('[role="dialog"]')).toBeNull());
  expect(localStorage.getItem("sparky.shell.onboarding")).toBe("complete");
  expect(requestNotificationAuthorization).not.toHaveBeenCalled();
});
it("enables notifications from onboarding and reports real reminder status", async () => {
  localStorage.removeItem("sparky.shell.onboarding");
  const { requestNotificationAuthorization } = mount([], { permission: "not-determined" });
  await vi.waitFor(() => expect(document.querySelector('[role="dialog"]')?.textContent).toContain("Welcome to Sparky"));
  button("Continue").click();
  await vi.waitFor(() => expect(button("Enable notifications").disabled).toBe(false));
  expect(requestNotificationAuthorization).not.toHaveBeenCalled();
  button("Enable notifications").click();
  await vi.waitFor(() => expect(requestNotificationAuthorization).toHaveBeenCalledOnce());
  await vi.waitFor(() => expect(document.querySelector('[role="dialog"]')?.textContent).toContain("Reminders refresh when Sparky runs"));
  expect(document.querySelector('[role="dialog"]')?.textContent).not.toMatch(/Reminders: \d+ armed/);
  expect(button("Enable notifications").disabled).toBe(true);
});
it("renders registered Commander sources and strips FTS markers from snippets", async () => {
  const run = vi.fn();
  const { host, search } = mount([{
    id: "search",
    routes: [],
    commandSources: [{ id: "memories", search: async query => ({ hits: [{ id: "hit-1", label: `Found ${query}`, snippet: "see <b>milk</b> here", run }] }) }],
  }]);
  await vi.waitFor(() => expect(host.textContent).toContain("Today is unavailable"));
  key("k"); await vi.waitFor(() => expect(document.querySelector("#sparky-command-input")).not.toBeNull());
  fill("milk");
  await vi.waitFor(() => expect(document.body.textContent).toContain("Found milk"));
  expect(search).not.toHaveBeenCalled();
  expect(document.body.textContent).toContain("see milk here");
  expect(document.body.textContent).not.toContain("<b>");
  expect(document.querySelector('[role="dialog"]')?.innerHTML).not.toContain("<b>milk</b>");
  button("Found milksee milk here").click();
  await vi.waitFor(() => expect(run).toHaveBeenCalledOnce());
});
it("renders registered memoryDetail, memoryCollection, and sidebarTree", async () => {
  const Detail = (props: { memoryId: string; detailSections: { id: string }[] }) => (
    <DetailPanel.Body><DetailPanel.Main>Memory detail {props.memoryId} {props.detailSections.map(section => section.id).join(",")}</DetailPanel.Main></DetailPanel.Body>
  );
  const Collection = (props: { mindId: string }) => {
    const shell = useSparkyShell();
    return <><p>Mind collection {props.mindId}/{shell.params().id}</p><Button onClick={() => shell.openMemory("mem-1")}>Open memory</Button></>;
  };
  const { host, onLink } = mount([
    {
      id: "memories",
      routes: [],
      memoryCollection: Collection,
      memoryDetail: Detail,
      detailSections: [{ id: "schedule", order: 1, component: () => <p>Schedule card</p> }],
    },
    { id: "map", routes: [], detailSections: [{ id: "place", order: 2, component: () => <p>Place card</p> }] },
    { id: "minds", routes: [], sidebarTree: { id: "minds-tree", component: () => <p>Registered minds tree</p> } },
  ]);
  await vi.waitFor(() => expect(host.textContent).toContain("Registered minds tree"));
  onLink("sparky://mind/home");
  await vi.waitFor(() => expect(host.textContent).toContain("Mind collection home/home"));
  expect(host.textContent).not.toContain("Mind is unavailable");
  button("Open memory").click();
  await vi.waitFor(() => expect(host.textContent).toContain("Memory detail mem-1 schedule,place"));
});
it("starts independent queries with the reminder probe", async () => {
  const delayed = mount([], { holdStatus: true });
  await vi.waitFor(() => expect(delayed.status).toHaveBeenCalled());
  await vi.waitFor(() => expect(delayed.listMemories).toHaveBeenCalled());
  await vi.waitFor(() => expect(delayed.listMinds).toHaveBeenCalled());
  expect(delayed.host.textContent).toContain("Starting Sparky");
  delayed.releaseStatus();
  await vi.waitFor(() => expect(delayed.host.textContent).toContain("Today is unavailable"));
});
it("skips duplicate Mind reads when a sidebar tree is registered", async () => {
  const withTree = mount([
    { id: "minds", routes: [], sidebarTree: { id: "minds-tree", component: () => <p>Registered minds tree</p> } },
  ], { holdStatus: true });
  await vi.waitFor(() => expect(withTree.listMemories).toHaveBeenCalled());
  expect(withTree.listMinds).not.toHaveBeenCalled();
  withTree.releaseStatus();
  await vi.waitFor(() => expect(withTree.host.textContent).toContain("Registered minds tree"));
  expect(withTree.listMinds).not.toHaveBeenCalled();
});
