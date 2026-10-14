import { afterEach, describe, expect, it, vi } from "vitest";
import { render } from "solid-js/web";
import { createNotesBridge, type NotesWireBridge } from "./bridge";
import { fakeBridge } from "./fake-bridge";
import { noteFixture } from "./fixtures";
import { createNotesStore } from "./store";
import { NoteFixtureWorkspace } from "./fixture-workspace";
import { orderedSections } from "./note-detail";

const cleanups: (() => void)[] = [];
afterEach(() => { cleanups.splice(0).reverse().forEach(cleanup => cleanup()); document.body.innerHTML = ""; vi.restoreAllMocks(); });
function mount(seed = [noteFixture()], debounceMs = 30, virtualized = false) {
  const fake = fakeBridge(seed), store = createNotesStore(() => createNotesBridge(fake.bridge as unknown as NotesWireBridge), debounceMs);
  const host = document.createElement("div"); document.body.append(host);
  const dispose = render(() => <NoteFixtureWorkspace store={store} virtualized={virtualized} />, host);
  cleanups.push(() => { dispose(); store.dispose(); host.remove(); });
  return { ...fake, store, host };
}
function input(element: HTMLInputElement | HTMLTextAreaElement, value: string) { element.value = value; element.dispatchEvent(new InputEvent("input", { bubbles: true })); }
function press(element: Element, key: string, options: KeyboardEventInit = {}) { element.dispatchEvent(new KeyboardEvent("keydown", { key, bubbles: true, cancelable: true, ...options })); }
function button(label: string) { return [...document.querySelectorAll<HTMLButtonElement>("button")].find(element => element.textContent?.trim() === label)!; }
async function row(host: HTMLElement, id = "note-1") { await vi.waitFor(() => expect(host.querySelector(`[data-row-id="${id}"]`)).not.toBeNull()); return host.querySelector<HTMLElement>(`[data-row-id="${id}"]`)!; }
async function open(host: HTMLElement) { press(await row(host), "Enter"); await vi.waitFor(() => expect(document.querySelector("[data-note-title]")).not.toBeNull()); }

describe("note page behavior through a fake preload bridge", () => {
  it("refreshes virtualized row labels after a detail title save and closure", async () => {
    vi.spyOn(HTMLElement.prototype, "offsetHeight", "get").mockReturnValue(480);
    vi.spyOn(HTMLElement.prototype, "offsetWidth", "get").mockReturnValue(800);
    const previous = HTMLElement.prototype.scrollTo;
    HTMLElement.prototype.scrollTo = () => {};
    cleanups.push(() => { HTMLElement.prototype.scrollTo = previous; });
    const { host, store, data, bridge, emit } = mount([noteFixture()], 30, true);
    await open(host);
    const title = document.querySelector<HTMLInputElement>("[data-note-title]")!;
    input(title, "Acknowledged title");
    await store.flush("note-1");
    expect(data.get("note-1")!.title).toBe("Acknowledged title");
    expect(store.state("note-1")).toBe("live");
    // The same entity remains in the virtual range while the stream refreshes it.
    emit();
    await vi.waitFor(() => expect(bridge.notes.list).toHaveBeenCalledTimes(2));
    document.querySelector<HTMLButtonElement>('[aria-label="Close details"]')!.click();
    await vi.waitFor(() => {
      const savedRow = host.querySelector('[data-row-id="note-1"]')!;
      expect(savedRow.getAttribute("aria-label")).toBe("Acknowledged title");
      expect(savedRow.textContent).toContain("Acknowledged title");
      expect(savedRow.querySelector('[aria-label="Pin Acknowledged title"]')).not.toBeNull();
      expect(savedRow.querySelector('[aria-label="Complete Acknowledged title"]')).not.toBeNull();
    });
  });

  it("renders a nonempty virtualized subset after asynchronous fake-bridge loading", async () => {
    vi.spyOn(HTMLElement.prototype, "offsetHeight", "get").mockReturnValue(480);
    vi.spyOn(HTMLElement.prototype, "offsetWidth", "get").mockReturnValue(800);
    const previous = HTMLElement.prototype.scrollTo;
    HTMLElement.prototype.scrollTo = () => {};
    cleanups.push(() => { HTMLElement.prototype.scrollTo = previous; });
    const seed = Array.from({ length: 1200 }, (_, index) => noteFixture(index));
    const { host } = mount(seed, 30, true);
    await vi.waitFor(() => expect(host.textContent).toContain("Live · 1200 notes"));
    await vi.waitFor(() => expect(host.querySelectorAll("[data-row-id]").length).toBeGreaterThan(0));
    expect(host.querySelectorAll("[data-row-id]").length).toBeLessThan(40);
    expect(host.querySelector('.pw-data-table-scroll[data-virtualized="true"]')).not.toBeNull();
  });

  it("completes a note through the native row checkbox without opening details", async () => {
    const { host, bridge } = mount();
    const first = await row(host);
    const checkbox = first.querySelector<HTMLInputElement>('input[aria-label^="Complete "]')!;
    checkbox.focus();
    checkbox.click();
    await vi.waitFor(() => expect(bridge.notes.setStatus).toHaveBeenCalled());
    expect(document.querySelector("[data-note-title]")).toBeNull();
  });

  it("supports keyboard create, edit, pin, complete and delete confirmation with undo", async () => {
    const { host, bridge, store } = mount();
    const first = await row(host); first.focus(); press(first, "p");
    await vi.waitFor(() => expect(bridge.notes.setPinned).toHaveBeenCalled());
    press(await row(host), "e"); await vi.waitFor(() => expect(document.querySelector("[data-note-title]")).not.toBeNull());
    const title = document.querySelector<HTMLInputElement>("[data-note-title]")!; title.focus(); input(title, "Edited using keyboard"); press(title, "s", { metaKey: true });
    await vi.waitFor(() => expect(store.record("note-1")!.title).toBe("Edited using keyboard")); await store.flush("note-1");
    button("Close details")?.click();
    const selected = await row(host); press(selected, "Backspace", { metaKey: true });
    await vi.waitFor(() => expect(document.querySelector('[role="dialog"]')).not.toBeNull());
    const deleteButton = button("Delete"); deleteButton.focus(); deleteButton.click();
    await vi.waitFor(() => expect(store.deleting().has("note-1")).toBe(true));
    button("Undo").click(); await vi.waitFor(() => expect(store.deleting().size).toBe(0));
    press(await row(host), " "); await vi.waitFor(() => expect(bridge.notes.setStatus).toHaveBeenCalled());
    button("Undo").click(); await vi.waitFor(() => expect(store.record("note-1")!.status).toBe("active"));
    const current = await row(host); press(current, "n", { metaKey: true });
    await vi.waitFor(() => expect(document.querySelector("#new-note-title")).not.toBeNull());
    input(document.querySelector<HTMLInputElement>("#new-note-title")!, "Keyboard creation");
    document.querySelector("form")!.dispatchEvent(new Event("submit", { bubbles: true, cancelable: true }));
    await vi.waitFor(() => expect(bridge.notes.create).toHaveBeenCalledWith({ title: "Keyboard creation", spaceId: "" }));
  });
  it("debounces rapid title edits without flushing on every reactive detail update", async () => {
    const { host, store, bridge } = mount([noteFixture()], 1000); await open(host);
    const title = document.querySelector<HTMLInputElement>("[data-note-title]")!;
    input(title, "A"); input(title, "AB"); input(title, "ABC");
    await new Promise(done => setTimeout(done, 20));
    expect(bridge.notes.update).not.toHaveBeenCalled();
    await store.flush("note-1"); expect(bridge.notes.update).toHaveBeenCalledTimes(1);
    expect(store.record("note-1")!.title).toBe("ABC");
  });
  it("shows Reload and Overwrite on conflict and retains the Markdown draft", async () => {
    const { host, store, data } = mount(); await open(host);
    data.set("note-1", { ...data.get("note-1")!, content: "Remote", rev: "2" });
    store.edit("note-1", { content: "Local Markdown **draft**" }); await store.flush("note-1");
    await vi.waitFor(() => expect([...document.querySelectorAll('[role="alert"]')].some(element => element.textContent?.includes("Edit conflict"))).toBe(true));
    expect(button("Reload")).toBeDefined(); expect(button("Overwrite")).toBeDefined();
    button("Overwrite").click(); await vi.waitFor(() => expect(data.get("note-1")!.content).toBe("Local Markdown **draft**"));
    expect(store.state("note-1")).toBe("live");
  });
  it("keeps checklist input focus during rapid editing and preserves detail on reorder", async () => {
    const note = noteFixture(); note.checklist!.push({ id: "check-2", title: "Second", detail: "Second detail" });
    const { host, store, data } = mount([note]); await open(host);
    const title = document.querySelector<HTMLInputElement>('[aria-label="Checklist title 1"]')!; title.focus();
    input(title, "A"); input(title, "AB"); input(title, "ABC");
    expect(document.activeElement).toBe(title); expect(document.querySelector('[aria-label="Checklist title 1"]')).toBe(title);
    button("Move down").click(); await store.flush("note-1");
    expect(data.get("note-1")!.checklist![1]!.title).toBe("ABC"); expect(data.get("note-1")!.checklist![1]!.detail).toBe("Keep details");
  });
  it("invalidates Inbox membership after moving a note through the detail Space picker", async () => {
    const { host, store } = mount(); await open(host);
    store.edit("note-1", { spaceId: "space-1" }); await store.flush("note-1");
    await vi.waitFor(() => expect(host.textContent).toContain("Start from a template"));
    expect(store.record("note-1")!.spaceId).toBe("space-1");
  });
  it("renders offline recovery and empty collections honestly", async () => {
    const { host, bridge, store } = mount([]);
    await vi.waitFor(() => expect(host.textContent).toContain("Start from a template"));
    vi.mocked(bridge.notes.list).mockRejectedValueOnce(new Error("Core offline")); await store.refresh();
    expect(host.textContent).toContain("Notes unavailable"); expect(host.textContent).toContain("Retry connection");
    button("Retry connection").click(); await vi.waitFor(() => expect(store.loadError()).toBe(""));
  });
  it("toggles table columns for due date and priority", async () => {
    const { host } = mount();
    await row(host);
    expect(host.textContent).not.toContain("Priority");
    document.querySelector<HTMLButtonElement>('[aria-label="Filters"]')!.click();
    const layout = () => [...document.querySelectorAll<HTMLButtonElement>("button")].find(element => element.textContent?.includes("Layout"));
    await vi.waitFor(() => expect(layout()).toBeTruthy());
    layout()!.click();
    await vi.waitFor(() => expect([...document.querySelectorAll(".pw-switch-row")].some(element => element.textContent?.includes("Table"))).toBe(true));
    [...document.querySelectorAll<HTMLElement>(".pw-switch-row")].find(element => element.textContent?.includes("Table"))!.click();
    await vi.waitFor(() => expect(host.textContent).toContain("Priority"));
    expect(host.textContent).toContain("Due");
  });
  it("orders visible contributed detail sections without importing their owners", () => {
    const component = () => <p>Contribution</p>;
    expect(orderedSections([{ id: "attachments", order: 30, component }, { id: "place", order: 20, component, visible: id => id === "other" }, { id: "schedule", order: 10, component }], "note-1").map(section => section.id)).toEqual(["schedule", "attachments"]);
  });
});
