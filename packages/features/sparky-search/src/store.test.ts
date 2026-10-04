import { afterEach, describe, expect, it, vi } from "vitest";
import { fakeBridge, SEARCH_FIXTURES, filterFixtures } from "./fake-bridge";
import { createSearchStore } from "./store";
import { EMPTY_FILTERS } from "./model";

const cleanups: Array<() => void> = [];
afterEach(() => { cleanups.splice(0).reverse().forEach(run => run()); vi.useRealTimers(); });

function mount(fail?: "search" | "status" | "unavailable", extra?: { notice?: string; ftsDocuments?: number; storage?: Storage }) {
  const memory = extra?.storage ?? window.localStorage;
  memory.clear();
  const fake = fakeBridge({ fail, notice: extra?.notice, ftsDocuments: extra?.ftsDocuments });
  const store = createSearchStore(() => fake.bridge, { debounceMs: 20, now: () => new Date(2026, 9, 4, 15, 0, 0), storage: memory });
  cleanups.push(() => store.dispose());
  return { fake, store };
}

describe("search store against SearchService fixtures", () => {
  it("returns the same hits as the SearchService fixture for milk + active + Home", async () => {
    const { fake, store } = mount();
    store.connect();
    await vi.waitFor(() => expect(store.loadState()).toBe("live"));
    expect(fake.bridge.search.status).toHaveBeenCalled();
    store.setQuery("milk");
    await store.flush();
    const expected = filterFixtures(SEARCH_FIXTURES, { query: "milk", status: "active", includeDescendants: true, limit: 50 });
    expect(store.hits().map(hit => hit.memoryId)).toEqual(expected.map(hit => hit.memoryId));
    expect(fake.bridge.search.search).toHaveBeenCalledWith(expect.objectContaining({ query: "milk", status: "active", limit: 50 }));
  });

  it("cancels stale requests and keeps the latest query", async () => {
    let resolveFirst: (value: { hits: { memoryId: string }[] }) => void = () => {};
    const fake = fakeBridge();
    fake.bridge.search.search = vi.fn()
      .mockImplementationOnce(() => new Promise(resolve => { resolveFirst = resolve; }))
      .mockImplementationOnce(async () => ({ hits: [{ memoryId: "memory-eggs", title: "Boil eggs", snippet: "Breakfast <b>eggs</b>" }], notice: "" }));
    const store = createSearchStore(() => fake.bridge, { debounceMs: 5, now: () => new Date(2026, 9, 4), storage: window.localStorage });
    cleanups.push(() => store.dispose());
    store.connect();
    store.setQuery("milk");
    await new Promise(done => setTimeout(done, 15));
    store.setQuery("eggs");
    await vi.waitFor(() => expect(store.visibleHits().map(hit => hit.memoryId)).toEqual(["memory-eggs"]));
    resolveFirst({ hits: [{ memoryId: "memory-milk" }] });
    await Promise.resolve();
    expect(store.visibleHits().map(hit => hit.memoryId)).toEqual(["memory-eggs"]);
  });

  it("stores recent searches locally and restores saved filter chips", async () => {
    const { store } = mount();
    store.connect();
    await vi.waitFor(() => expect(store.loadState()).toBe("live"));
    store.setQuery("milk");
    await store.flush();
    expect(store.recent()).toEqual(["milk"]);
    expect(JSON.parse(localStorage.getItem("sparky.search.recent") ?? "[]")).toEqual(["milk"]);
    store.setFilters({ status: "completed", mindId: "mind-2" });
    store.saveFilters("Done in Work");
    expect(store.saved()[0]?.name).toBe("Done in Work");
    store.clearFilters();
    store.applySaved(store.saved()[0]!.id);
    expect(store.filters()).toMatchObject({ status: "completed", mindId: "mind-2" });
  });

  it("reports an empty index as degraded FTS5 copy", async () => {
    const { store } = mount(undefined, { ftsDocuments: 0 });
    store.connect();
    await vi.waitFor(() => expect(store.loadState()).toBe("live"));
    expect(store.honesty()).toMatchObject({ label: "Index empty", tone: "degraded" });
  });

  it("marks the core unreachable as unavailable", async () => {
    const { store } = mount("unavailable");
    store.connect();
    await vi.waitFor(() => expect(store.loadState()).toBe("unavailable"));
    expect(store.loadError()).toContain("unreachable");
  });
});

describe("empty filters", () => {
  it("does not search until the query has text", async () => {
    const { fake, store } = mount();
    store.connect();
    await vi.waitFor(() => expect(store.loadState()).toBe("live"));
    store.setFilters({ ...EMPTY_FILTERS, status: "completed" });
    await store.flush();
    expect(fake.bridge.search.search).not.toHaveBeenCalled();
    expect(store.emptyQuery()).toBe(true);
  });
});
