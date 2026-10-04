import { afterEach, describe, expect, it, vi } from "vitest";
import { render } from "solid-js/web";
import { fakeBridge } from "./fake-bridge";
import { memoryFixture } from "./fixtures";
import { createMemoriesStore } from "./store";
import { MemoryFixtureWorkspace } from "./fixture-workspace";
import { orderedSections } from "./memory-detail";

const cleanups: (() => void)[] = [];
afterEach(() => { cleanups.splice(0).reverse().forEach(cleanup => cleanup()); document.body.innerHTML = ""; vi.restoreAllMocks(); });
function mount(seed = [memoryFixture()], debounceMs = 30) {
  const fake = fakeBridge(seed), store = createMemoriesStore(() => fake.bridge, debounceMs);
  const host = document.createElement("div"); document.body.append(host);
  const dispose = render(() => <MemoryFixtureWorkspace store={store} virtualized={false} />, host);
  cleanups.push(() => { dispose(); store.dispose(); host.remove(); });
  return { ...fake, store, host };
}
function input(element: HTMLInputElement | HTMLTextAreaElement, value: string) { element.value = value; element.dispatchEvent(new InputEvent("input", { bubbles: true })); }
function press(element: Element, key: string, options: KeyboardEventInit = {}) { element.dispatchEvent(new KeyboardEvent("keydown", { key, bubbles: true, cancelable: true, ...options })); }
function button(label: string) { return [...document.querySelectorAll<HTMLButtonElement>("button")].find(element => element.textContent?.trim() === label)!; }
async function row(host: HTMLElement, id = "memory-1") { await vi.waitFor(() => expect(host.querySelector(`[data-row-id="${id}"]`)).not.toBeNull()); return host.querySelector<HTMLElement>(`[data-row-id="${id}"]`)!; }
async function open(host: HTMLElement) { press(await row(host), "Enter"); await vi.waitFor(() => expect(document.querySelector("[data-memory-title]")).not.toBeNull()); }

describe("memory page behavior through a fake preload bridge", () => {
  it("supports keyboard create, edit, pin, complete and delete confirmation with undo", async () => {
    const { host, bridge, store } = mount();
    const first = await row(host); first.focus(); press(first, "p");
    await vi.waitFor(() => expect(bridge.memories.setPinned).toHaveBeenCalled());
    press(await row(host), "e"); await vi.waitFor(() => expect(document.querySelector("[data-memory-title]")).not.toBeNull());
    const title = document.querySelector<HTMLInputElement>("[data-memory-title]")!; title.focus(); input(title, "Edited using keyboard"); press(title, "s", { metaKey: true });
    await vi.waitFor(() => expect(store.record("memory-1")!.title).toBe("Edited using keyboard")); await store.flush("memory-1");
    button("Close details")?.click();
    const selected = await row(host); press(selected, "Backspace", { metaKey: true });
    await vi.waitFor(() => expect(document.querySelector('[role="dialog"]')).not.toBeNull());
    const deleteButton = button("Delete"); deleteButton.focus(); deleteButton.click();
    await vi.waitFor(() => expect(store.deleting().has("memory-1")).toBe(true));
    button("Undo").click(); await vi.waitFor(() => expect(store.deleting().size).toBe(0));
    press(await row(host), " "); await vi.waitFor(() => expect(bridge.memories.setStatus).toHaveBeenCalled());
    button("Undo").click(); await vi.waitFor(() => expect(store.record("memory-1")!.status).toBe("active"));
    const current = await row(host); press(current, "n", { metaKey: true });
    await vi.waitFor(() => expect(document.querySelector("#new-memory-title")).not.toBeNull());
    input(document.querySelector<HTMLInputElement>("#new-memory-title")!, "Keyboard creation");
    document.querySelector("form")!.dispatchEvent(new Event("submit", { bubbles: true, cancelable: true }));
    await vi.waitFor(() => expect(bridge.memories.create).toHaveBeenCalledWith({ title: "Keyboard creation", mindId: "" }));
  });
  it("debounces rapid title edits without flushing on every reactive detail update", async () => {
    const { host, store, bridge } = mount([memoryFixture()], 1000); await open(host);
    const title = document.querySelector<HTMLInputElement>("[data-memory-title]")!;
    input(title, "A"); input(title, "AB"); input(title, "ABC");
    await new Promise(done => setTimeout(done, 20));
    expect(bridge.memories.update).not.toHaveBeenCalled();
    await store.flush("memory-1"); expect(bridge.memories.update).toHaveBeenCalledTimes(1);
    expect(store.record("memory-1")!.title).toBe("ABC");
  });
  it("shows Reload and Overwrite on conflict and retains the Markdown draft", async () => {
    const { host, store, data } = mount(); await open(host);
    data.set("memory-1", { ...data.get("memory-1")!, note: "Remote", rev: "2" });
    store.edit("memory-1", { note: "Local Markdown **draft**" }); await store.flush("memory-1");
    await vi.waitFor(() => expect([...document.querySelectorAll('[role="alert"]')].some(element => element.textContent?.includes("Edit conflict"))).toBe(true));
    expect(button("Reload")).toBeDefined(); expect(button("Overwrite")).toBeDefined();
    button("Overwrite").click(); await vi.waitFor(() => expect(data.get("memory-1")!.note).toBe("Local Markdown **draft**"));
    expect(store.state("memory-1")).toBe("live");
  });
  it("keeps checklist input focus during rapid editing and preserves detail on reorder", async () => {
    const memory = memoryFixture(); memory.checklist!.push({ id: "check-2", title: "Second", detail: "Second detail" });
    const { host, store, data } = mount([memory]); await open(host);
    const title = document.querySelector<HTMLInputElement>('[aria-label="Checklist title 1"]')!; title.focus();
    input(title, "A"); input(title, "AB"); input(title, "ABC");
    expect(document.activeElement).toBe(title); expect(document.querySelector('[aria-label="Checklist title 1"]')).toBe(title);
    button("Move down").click(); await store.flush("memory-1");
    expect(data.get("memory-1")!.checklist![1]!.title).toBe("ABC"); expect(data.get("memory-1")!.checklist![1]!.detail).toBe("Keep details");
  });
  it("invalidates Inbox membership after moving a memory through the detail Mind picker", async () => {
    const { host, store } = mount(); await open(host);
    store.edit("memory-1", { mindId: "mind-1" }); await store.flush("memory-1");
    await vi.waitFor(() => expect(host.textContent).toContain("No memories here"));
    expect(store.record("memory-1")!.mindId).toBe("mind-1");
  });
  it("renders offline recovery and empty collections honestly", async () => {
    const { host, bridge, store } = mount([]);
    await vi.waitFor(() => expect(host.textContent).toContain("No memories here"));
    vi.mocked(bridge.memories.list).mockRejectedValueOnce(new Error("Core offline")); await store.refresh();
    expect(host.textContent).toContain("Memories unavailable"); expect(host.textContent).toContain("Retry connection");
    button("Retry connection").click(); await vi.waitFor(() => expect(store.loadError()).toBe(""));
  });
  it("orders visible contributed detail sections without importing their owners", () => {
    const component = () => <p>Contribution</p>;
    expect(orderedSections([{ id: "attachments", order: 30, component }, { id: "place", order: 20, component, visible: id => id === "other" }, { id: "schedule", order: 10, component }], "memory-1").map(section => section.id)).toEqual(["schedule", "attachments"]);
  });
});
