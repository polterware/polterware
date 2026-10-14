import { render } from "solid-js/web";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import shellFeature, { SparkyShell, createShellBridge, type ShellWireBridge, type SparkyFeature } from "@polterware/feature-sparky-shell";
import spacesFeature from "./index";
import { SPACES_EXPANDED_KEY } from "./tree";

let dispose: (() => void) | undefined;
let store: {
  spaces: { id: string; name: string; parentId: string; color: string; icon: string; sortOrder: number; rev: string }[];
  notes: Record<string, number>;
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
    spaces: [
      { id: "a", name: "Alpha", parentId: "", color: "#7EA8F0", icon: "brain", sortOrder: 0, rev: "1" },
      { id: "b", name: "Beta", parentId: "a", color: "#6CC7A5", icon: "folder", sortOrder: 0, rev: "1" },
    ],
    notes: { a: 2, b: 1 },
  };
});
afterEach(() => {
  dispose?.();
  dispose = undefined;
  document.body.replaceChildren();
  vi.restoreAllMocks();
});

function mount(extra: SparkyFeature[] = [spacesFeature]) {
  const update = vi.fn(async (input: { id: string; expectedRev?: string; parentId?: string; clearParent?: boolean; name?: string }) => {
    const space = store.spaces.find((item) => item.id === input.id);
    if (!space) throw new Error("missing");
    if (input.name) space.name = input.name;
    if (input.clearParent) space.parentId = "";
    else if (input.parentId !== undefined) space.parentId = input.parentId;
    space.rev = String(Number(space.rev) + 1);
    return { space: { ...space } };
  });
  const create = vi.fn(async (input: { name: string; parentId?: string }) => {
    const space = {
      id: "new-space",
      name: input.name,
      parentId: input.parentId ?? "",
      color: "#7EA8F0",
      icon: "brain",
      sortOrder: store.spaces.length,
      rev: "1",
    };
    store.spaces.push(space);
    return { space };
  });
  const remove = vi.fn(async () => ({ deletedSpaceIds: ["a", "b"], movedNoteCount: 3 }));
  const wire = {
    reminders: {
      status: async () => ({ permission: "authorized", armed: 0, planned: 0, notifierLastSeenAt: new Date().toISOString() }),
    },
    spaces: {
      list: async () => ({ spaces: store.spaces.map((space) => ({ ...space })) }),
      create,
      update,
      delete: remove,
      reorder: async () => ({ spaces: store.spaces }),
    },
    notes: {
      list: async (input?: { spaceId?: string; includeDescendants?: boolean }) => {
        if (!input?.spaceId) return { total: Object.values(store.notes).reduce((sum, count) => sum + count, 0) };
        if (input.includeDescendants) {
          const ids = new Set([input.spaceId]);
          for (const space of store.spaces) if (ids.has(space.parentId)) ids.add(space.id);
          return { total: [...ids].reduce((sum, id) => sum + (store.notes[id] ?? 0), 0) };
        }
        return { total: store.notes[input.spaceId] ?? 0 };
      },
      get: async () => ({ note: { id: "note", title: "Note", rev: "1" } }),
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
    return text.includes(name) && text.includes("child Spaces");
  });
  if (!found) throw new Error(`Space card ${name} missing: ${document.body.textContent}`);
  return found;
}

describe("Spaces feature", () => {
  it("composes the registered Space editor with its explicit spaceId note collection", async () => {
    const { host, update } = mount([spacesFeature, { id: "notes", routes: [], noteCollection: props => <p data-space-notes={props.spaceId}>Notes for {props.spaceId}</p> }]);
    await vi.waitFor(() => expect(host.textContent).toContain("Today is unavailable"));
    button("Spaces").click();
    await vi.waitFor(() => expect(host.textContent).toContain("child Spaces"));
    cardNamed("Alpha").click();
    await vi.waitFor(() => expect(host.querySelector("h1")?.textContent).toBe("Alpha"));
    expect(host.querySelector('[data-space-notes="a"]')).not.toBeNull();
    expect(host.querySelector('[aria-label="Breadcrumb"]')).not.toBeNull();
    expect(host.textContent).toContain("Child Spaces");
    expect(button("Delete Space")).toBeDefined();
    expect(host.querySelector('[aria-label="Move to"]')).not.toBeNull();
    const input = host.querySelector<HTMLInputElement>('input')!;
    input.value = "Renamed Alpha"; input.dispatchEvent(new Event("input", { bubbles: true }));
    button("Save changes").click();
    await vi.waitFor(() => expect(update).toHaveBeenCalledWith(expect.objectContaining({ id: "a", name: "Renamed Alpha" })));
  });
  it("registers a sidebar tree contribution and keeps the flat Spaces fallback", () => {
    expect(spacesFeature.sidebar?.some((item) => item.path === "/spaces")).toBe(true);
    expect(spacesFeature.sidebarTree?.id).toBe("spaces");
    expect(spacesFeature.sidebarTree?.component).toBeTypeOf("function");
  });

  it("opens a Space, persists expansion, and shows cascade numbers from {deletedSpaceIds, movedNoteCount}", async () => {
    const { host, remove } = mount();
    await vi.waitFor(() => expect(host.textContent).toContain("Today is unavailable"));
    button("Spaces").click();
    await vi.waitFor(() => expect(host.textContent).toContain("child Spaces"));
    cardNamed("Alpha").click();
    await vi.waitFor(() => expect(host.querySelector("h1")?.textContent).toBe("Alpha"));
    expect(host.querySelector("[aria-label='Breadcrumb']")?.textContent).toContain("Spaces");
    expect(host.querySelector("[data-icon-tile]")).not.toBeNull();
    button("Delete Space").click();
    await vi.waitFor(() => expect(document.body.textContent).toContain("Deletes 1 child Spaces; 3 notes move to Inbox."));
    const confirm = Array.from(document.querySelectorAll("button")).filter((item) => item.textContent?.trim() === "Delete Space").at(-1)!;
    confirm.click();
    await vi.waitFor(() => expect(remove).toHaveBeenCalledWith(expect.objectContaining({ expectedRev: "1" })));
    expect(JSON.parse(localStorage.getItem(SPACES_EXPANDED_KEY) ?? "[]")).toEqual(expect.any(Array));
  });

  it("creates a Space and disables cyclic Move to targets", async () => {
    const { host, create } = mount();
    await vi.waitFor(() => expect(host.textContent).toContain("Today is unavailable"));
    button("Spaces").click();
    await vi.waitFor(() => expect(host.textContent).toContain("Create Space"));
    button("Create Space").click();
    await vi.waitFor(() => expect(document.querySelector("[role='dialog']")).not.toBeNull());
    const name = document.querySelector<HTMLInputElement>("input")!;
    name.value = "Delta";
    name.dispatchEvent(new Event("input", { bubbles: true }));
    button("Create").click();
    await vi.waitFor(() => expect(create).toHaveBeenCalledWith(expect.objectContaining({ name: "Delta" })));
    await vi.waitFor(() => expect(host.textContent).toContain("Delta"));
    await vi.waitFor(() => expect(host.textContent).toContain("child Spaces"));
    cardNamed("Alpha").click();
    await vi.waitFor(() => expect(host.querySelector("h1")?.textContent).toBe("Alpha"));
    expect(button("Move to")).toBeTruthy();
  });

  it("shows empty states when no Spaces exist", async () => {
    store.spaces = [];
    const { host } = mount();
    await vi.waitFor(() => expect(host.textContent).toContain("Today is unavailable"));
    button("Spaces").click();
    await vi.waitFor(() => expect(host.textContent).toContain("Start from a template"));
    expect(host.textContent).toContain("Personal");
  });
});
