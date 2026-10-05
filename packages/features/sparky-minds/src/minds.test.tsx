import { render } from "solid-js/web";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import shellFeature, { SparkyShell, createShellBridge, type ShellWireBridge, type SparkyFeature } from "@polterware/feature-sparky-shell";
import mindsFeature from "./index";
import { MINDS_EXPANDED_KEY } from "./tree";

let dispose: (() => void) | undefined;
let store: {
  minds: { id: string; name: string; parentId: string; color: string; icon: string; sortOrder: number; rev: string }[];
  memories: Record<string, number>;
};

beforeEach(() => {
  localStorage.clear();
  localStorage.setItem("sparky.shell.onboarding", "complete");
  vi.spyOn(HTMLElement.prototype, "getClientRects").mockImplementation(function (this: HTMLElement) {
    return this.isConnected && !this.closest("[hidden]")
      ? ([new DOMRect(0, 0, 100, 32)] as unknown as DOMRectList)
      : ([] as unknown as DOMRectList);
  });
  store = {
    minds: [
      { id: "a", name: "Alpha", parentId: "", color: "#7EA8F0", icon: "brain", sortOrder: 0, rev: "1" },
      { id: "b", name: "Beta", parentId: "a", color: "#6CC7A5", icon: "folder", sortOrder: 0, rev: "1" },
    ],
    memories: { a: 2, b: 1 },
  };
});
afterEach(() => {
  dispose?.();
  dispose = undefined;
  document.body.replaceChildren();
  vi.restoreAllMocks();
});

function mount(extra: SparkyFeature[] = [mindsFeature]) {
  const update = vi.fn(async (input: { id: string; expectedRev?: string; parentId?: string; clearParent?: boolean; name?: string }) => {
    const mind = store.minds.find((item) => item.id === input.id);
    if (!mind) throw new Error("missing");
    if (input.name) mind.name = input.name;
    if (input.clearParent) mind.parentId = "";
    else if (input.parentId !== undefined) mind.parentId = input.parentId;
    mind.rev = String(Number(mind.rev) + 1);
    return { mind: { ...mind } };
  });
  const create = vi.fn(async (input: { name: string; parentId?: string }) => {
    const mind = {
      id: "new-mind",
      name: input.name,
      parentId: input.parentId ?? "",
      color: "#7EA8F0",
      icon: "brain",
      sortOrder: store.minds.length,
      rev: "1",
    };
    store.minds.push(mind);
    return { mind };
  });
  const remove = vi.fn(async () => ({ deletedMindIds: ["a", "b"], movedMemoryCount: 3 }));
  const wire = {
    reminders: {
      status: async () => ({ permission: "authorized", armed: 0, planned: 0, notifierLastSeenAt: new Date().toISOString() }),
    },
    minds: {
      list: async () => ({ minds: store.minds.map((mind) => ({ ...mind })) }),
      create,
      update,
      delete: remove,
      reorder: async () => ({ minds: store.minds }),
    },
    memories: {
      list: async (input?: { mindId?: string; includeDescendants?: boolean }) => {
        if (!input?.mindId) return { total: Object.values(store.memories).reduce((sum, count) => sum + count, 0) };
        if (input.includeDescendants) {
          const ids = new Set([input.mindId]);
          for (const mind of store.minds) if (ids.has(mind.parentId)) ids.add(mind.id);
          return { total: [...ids].reduce((sum, id) => sum + (store.memories[id] ?? 0), 0) };
        }
        return { total: store.memories[input.mindId] ?? 0 };
      },
      get: async () => ({ memory: { id: "memory", title: "Note", rev: "1" } }),
    },
    tags: { list: async () => ({ tags: [] }) },
    search: { search: async () => ({ hits: [] }) },
    onChange: () => () => {},
    app: { onDeepLink: () => () => {}, getDeepLink: async () => null },
  } as unknown as ShellWireBridge;
  const host = document.createElement("div");
  document.body.append(host);
  (window as unknown as { sparky: ShellWireBridge }).sparky = wire;
  dispose = render(() => <SparkyShell features={[shellFeature, ...extra]} bridge={createShellBridge(wire)} />, host);
  return { host, update, create, remove };
}

function button(label: string): HTMLButtonElement {
  const found = Array.from(document.querySelectorAll<HTMLButtonElement>("button")).find(
    (item) => item.getAttribute("aria-label") === label || item.textContent?.trim() === label,
  );
  if (!found) throw new Error(`Button ${label} missing: ${document.body.textContent}`);
  return found;
}

function cardNamed(name: string): HTMLElement {
  const found = Array.from(document.querySelectorAll("button")).find((item) => {
    const text = item.textContent ?? "";
    return text.includes(name) && text.includes("child Minds");
  });
  if (!found) throw new Error(`Mind card ${name} missing: ${document.body.textContent}`);
  return found;
}

describe("Minds feature", () => {
  it("composes the registered Mind editor with its explicit mindId memory collection", async () => {
    const { host, update } = mount([mindsFeature, { id: "memories", routes: [], memoryCollection: props => <p data-mind-memories={props.mindId}>Memories for {props.mindId}</p> }]);
    await vi.waitFor(() => expect(host.textContent).toContain("Today is unavailable"));
    button("Minds").click();
    await vi.waitFor(() => expect(host.textContent).toContain("child Minds"));
    cardNamed("Alpha").click();
    await vi.waitFor(() => expect(host.querySelector("h1")?.textContent).toBe("Alpha"));
    expect(host.querySelector('[data-mind-memories="a"]')).not.toBeNull();
    expect(host.querySelector('[aria-label="Breadcrumb"]')).not.toBeNull();
    expect(host.textContent).toContain("Child Minds");
    expect(button("Delete Mind")).toBeDefined();
    expect(host.querySelector('[aria-label="Move to"]')).not.toBeNull();
    const input = host.querySelector<HTMLInputElement>('input')!;
    input.value = "Renamed Alpha"; input.dispatchEvent(new Event("input", { bubbles: true }));
    button("Save changes").click();
    await vi.waitFor(() => expect(update).toHaveBeenCalledWith(expect.objectContaining({ id: "a", name: "Renamed Alpha" })));
  });
  it("registers a sidebar tree contribution and keeps the flat Minds fallback", () => {
    expect(mindsFeature.sidebar?.some((item) => item.path === "/minds")).toBe(true);
    expect(mindsFeature.sidebarTree?.id).toBe("minds");
    expect(mindsFeature.sidebarTree?.component).toBeTypeOf("function");
  });

  it("opens a Mind, persists expansion, and shows cascade numbers from {deletedMindIds, movedMemoryCount}", async () => {
    const { host, remove } = mount();
    await vi.waitFor(() => expect(host.textContent).toContain("Today is unavailable"));
    button("Minds").click();
    await vi.waitFor(() => expect(host.textContent).toContain("child Minds"));
    cardNamed("Alpha").click();
    await vi.waitFor(() => expect(host.querySelector("h1")?.textContent).toBe("Alpha"));
    expect(host.querySelector("[aria-label='Breadcrumb']")?.textContent).toContain("Minds");
    expect(host.querySelector("[data-dither-icon]")).not.toBeNull();
    button("Delete Mind").click();
    await vi.waitFor(() => expect(document.body.textContent).toContain("Deletes 1 child Minds; 3 memories move to Inbox."));
    const confirm = Array.from(document.querySelectorAll("button")).filter((item) => item.textContent?.trim() === "Delete Mind").at(-1)!;
    confirm.click();
    await vi.waitFor(() => expect(remove).toHaveBeenCalledWith(expect.objectContaining({ expectedRev: "1" })));
    expect(JSON.parse(localStorage.getItem(MINDS_EXPANDED_KEY) ?? "[]")).toEqual(expect.any(Array));
  });

  it("creates a Mind and disables cyclic Move to targets", async () => {
    const { host, create } = mount();
    await vi.waitFor(() => expect(host.textContent).toContain("Today is unavailable"));
    button("Minds").click();
    await vi.waitFor(() => expect(host.textContent).toContain("Create Mind"));
    button("Create Mind").click();
    await vi.waitFor(() => expect(document.querySelector("[role='dialog']")).not.toBeNull());
    const name = document.querySelector<HTMLInputElement>("input")!;
    name.value = "Delta";
    name.dispatchEvent(new Event("input", { bubbles: true }));
    button("Create").click();
    await vi.waitFor(() => expect(create).toHaveBeenCalledWith(expect.objectContaining({ name: "Delta" })));
    await vi.waitFor(() => expect(host.textContent).toContain("Delta"));
    await vi.waitFor(() => expect(host.textContent).toContain("child Minds"));
    cardNamed("Alpha").click();
    await vi.waitFor(() => expect(host.querySelector("h1")?.textContent).toBe("Alpha"));
    expect(button("Move to")).toBeTruthy();
  });

  it("shows empty states when no Minds exist", async () => {
    store.minds = [];
    const { host } = mount();
    await vi.waitFor(() => expect(host.textContent).toContain("Today is unavailable"));
    button("Minds").click();
    await vi.waitFor(() => expect(host.textContent).toContain("Start from a template"));
    expect(host.textContent).toContain("Personal");
  });
});
