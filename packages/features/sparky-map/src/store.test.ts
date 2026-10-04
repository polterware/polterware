import { describe, expect, it, vi } from "vitest";
import { createMapStore } from "./store";
import { createMapBridge } from "./bridge";
import { fakeMapBridge } from "./fake-bridge";
import { DISCLOSURE_KEY, PLACE_NOTICE, WORLD_BOUNDS } from "./model";

describe("map store", () => {
  it("paginates 1200 place titles in pages of at most 500", async () => {
    const places = Array.from({ length: 1200 }, (_, index) => ({ memoryId: `memory-${index}`, name: `Place ${index}`, lat: 0, lng: 0 }));
    const fake = fakeMapBridge({ places, memories: places.map(place => ({ id: place.memoryId, title: `Title ${place.memoryId}` })) });
    const store = createMapStore(() => createMapBridge(fake.wire));
    await store.reload();
    expect(store.loadState()).toBe("live");
    expect(store.places()).toHaveLength(1200);
    expect(store.places().at(-1)?.title).toBe("Title memory-1199");
    expect(vi.mocked(fake.wire.memories.list!).mock.calls.map(call => call[0])).toEqual([
      { hasPlace: true, limit: 500 },
      { hasPlace: true, limit: 500, cursor: "500" },
      { hasPlace: true, limit: 500, cursor: "1000" },
    ]);
    store.dispose();
  });

  it("stops stale pagination and does not overwrite newer bounds or titles", async () => {
    const fake = fakeMapBridge();
    let resolvePage!: (value: { memories: { id: string; title: string }[]; nextCursor: string }) => void;
    vi.mocked(fake.wire.memories.list!).mockImplementationOnce(() => new Promise(resolve => { resolvePage = resolve; }));
    const store = createMapStore(() => createMapBridge(fake.wire));
    const old = store.reload();
    await vi.waitFor(() => expect(resolvePage).toBeDefined());
    const next = { south: -24, west: -47, north: -23, east: -46 };
    store.setBounds(next);
    await store.flush();
    resolvePage({ memories: [{ id: "memory-1", title: "Stale" }], nextCursor: "stale-next" });
    await old;
    expect(store.places()[0]?.title).toBe("Buy milk");
    expect(store.bounds()).toEqual(next);
    expect(fake.wire.memories.list).toHaveBeenCalledTimes(2);
    store.dispose();
  });

  it("ignores an in-flight response after disposal", async () => {
    const fake = fakeMapBridge();
    let resolvePage!: (value: object) => void;
    vi.mocked(fake.wire.memories.list!).mockImplementationOnce(() => new Promise(resolve => { resolvePage = resolve as typeof resolvePage; }));
    const store = createMapStore(() => createMapBridge(fake.wire));
    const pending = store.reload();
    await vi.waitFor(() => expect(resolvePage).toBeDefined());
    store.dispose();
    resolvePage({ memories: [{ id: "memory-1", title: "Disposed" }], nextCursor: "next" });
    await pending;
    expect(store.places()).toEqual([]);
    expect(fake.wire.memories.list).toHaveBeenCalledTimes(1);
  });

  it.each(["repeat", "unbounded"])("bounds a %s cursor sequence instead of reporting partial titles as live", async mode => {
    const fake = fakeMapBridge();
    let page = 0;
    vi.mocked(fake.wire.memories.list!).mockImplementation(async () => ({ memories: [], nextCursor: mode === "repeat" ? "same" : String(++page) }));
    const store = createMapStore(() => createMapBridge(fake.wire));
    await store.reload();
    expect(store.loadState()).toBe("failed");
    expect(store.loadError()).toContain(mode === "repeat" ? "repeated cursor" : "exceeds 10000");
    expect(fake.wire.memories.list).toHaveBeenCalledTimes(mode === "repeat" ? 2 : 20);
    store.dispose();
  });

  it("lists places in bounds and keeps titles from memories", async () => {
    const fake = fakeMapBridge();
    const store = createMapStore(() => createMapBridge(fake.wire));
    const disconnect = store.connect();
    await store.flush();
    await store.reload();
    expect(store.places()).toEqual([expect.objectContaining({ memoryId: "memory-1", title: "Buy milk", name: "Home" })]);
    expect(store.mapOnline()).toBe(false);
    expect(store.tilesLive()).toBe(false);
    expect(fake.wire.places.listInBounds).toHaveBeenCalled();
    disconnect();
    store.dispose();
  });

  it("queries the current bounds through ListInBounds", async () => {
    const fake = fakeMapBridge();
    const store = createMapStore(() => createMapBridge(fake.wire));
    store.connect();
    await store.reload();
    await store.setBounds({ south: -24, west: -47, north: -23, east: -46 }, 12);
    await store.flush();
    const last = (fake.wire.places.listInBounds as ReturnType<typeof vi.fn>).mock.calls.at(-1)?.[0];
    expect(last).toMatchObject({ south: -24, west: -47, north: -23, east: -46 });
    expect(store.zoom()).toBe(12);
    store.dispose();
  });

  it("turns map.online on after the first-run disclosure", async () => {
    localStorage.clear();
    const fake = fakeMapBridge();
    const store = createMapStore(() => createMapBridge(fake.wire));
    store.connect();
    await store.reload();
    expect(store.showDisclosure()).toBe(true);
    await store.setOnline(true);
    expect(fake.wire.settings.set).toHaveBeenCalledWith({ keys: ["map.online"], settings: { mapOnline: true } });
    expect(store.mapOnline()).toBe(true);
    expect(store.disclosureSeen()).toBe(true);
    expect(localStorage.getItem(DISCLOSURE_KEY)).toBe("seen");
    expect(PLACE_NOTICE).toContain("do not trigger notifications");
    expect(WORLD_BOUNDS.south).toBeLessThan(WORLD_BOUNDS.north);
    store.dispose();
  });
});
