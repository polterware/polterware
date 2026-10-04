import { describe, expect, it, vi } from "vitest";
import { createMapStore } from "./store";
import { createMapBridge } from "./bridge";
import { fakeMapBridge } from "./fake-bridge";
import { DISCLOSURE_KEY, PLACE_NOTICE, WORLD_BOUNDS } from "./model";

describe("map store", () => {
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
