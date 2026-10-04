import { vi } from "vitest";
import type { MapWireBridge } from "./bridge";
import { DEFAULT_RADIUS_M, type MemoryJSON, type PlaceJSON, type SettingsJSON } from "./model";

export function fakeMapBridge(seed?: {
  places?: PlaceJSON[];
  memories?: MemoryJSON[];
  settings?: SettingsJSON;
}) {
  const places = new Map((seed?.places ?? [{
    memoryId: "memory-1", name: "Home", lat: -23.5505, lng: -46.6333, radiusM: DEFAULT_RADIUS_M, event: "enter", active: true, updatedAt: "2026-10-04T12:00:00.000Z",
  }]).map(place => [place.memoryId ?? "", structuredClone(place)]));
  const memories = new Map((seed?.memories ?? [{
    id: "memory-1", title: "Buy milk", note: "", status: "active", rev: "3", place: places.get("memory-1"),
  }]).map(memory => [memory.id ?? "", structuredClone(memory)]));
  let settings: SettingsJSON = { mapOnline: false, mapProvider: "openfreemap", ...seed?.settings };
  let listener: Parameters<MapWireBridge["onChange"]>[0] | undefined;

  const wire = {
    places: {
      listInBounds: vi.fn(async (input?: { south?: number; west?: number; north?: number; east?: number }) => {
        const south = input?.south ?? -90, west = input?.west ?? -180, north = input?.north ?? 90, east = input?.east ?? 180;
        return { places: [...places.values()].filter(place => (place.lat ?? 0) >= south && (place.lat ?? 0) <= north && (place.lng ?? 0) >= west && (place.lng ?? 0) <= east) };
      }),
      set: vi.fn(async (input?: { expectedRev?: string; place?: PlaceJSON }) => {
        const memory = memories.get(input?.place?.memoryId ?? "");
        if (input?.expectedRev && memory && input.expectedRev !== memory.rev) throw Object.assign(new Error("memory rev conflict"), { code: "conflict" });
        const place = { ...input?.place, radiusM: input?.place?.radiusM ?? DEFAULT_RADIUS_M, event: input?.place?.event || "enter", active: input?.place?.active ?? true, updatedAt: new Date().toISOString() };
        places.set(place.memoryId ?? "", place);
        if (memory) {
          memory.place = place;
          memory.rev = String(BigInt(memory.rev ?? "0") + 1n);
        }
        return { place };
      }),
      clear: vi.fn(async (input?: { memoryId?: string; expectedRev?: string }) => {
        const memory = memories.get(input?.memoryId ?? "");
        if (input?.expectedRev && memory && input.expectedRev !== memory.rev) throw Object.assign(new Error("memory rev conflict"), { code: "conflict" });
        places.delete(input?.memoryId ?? "");
        if (memory) {
          memory.place = undefined;
          memory.rev = String(BigInt(memory.rev ?? "0") + 1n);
        }
        return {};
      }),
    },
    memories: {
      get: vi.fn(async (input?: { id?: string }) => {
        const memory = memories.get(input?.id ?? "");
        if (!memory) throw new Error("Memory not found");
        return { memory: structuredClone(memory) };
      }),
      list: vi.fn(async (input?: { hasPlace?: boolean }) => {
        const rows = [...memories.values()].filter(memory => !input?.hasPlace || Boolean(memory.place ?? places.get(memory.id ?? "")));
        return { memories: rows.map(memory => ({ id: memory.id, title: memory.title, snippet: memory.note, rev: memory.rev })), total: rows.length };
      }),
    },
    settings: {
      get: vi.fn(async () => ({ settings: structuredClone(settings) })),
      set: vi.fn(async (input?: { keys?: string[]; settings?: SettingsJSON }) => {
        settings = { ...settings, ...input?.settings };
        return { settings: structuredClone(settings) };
      }),
    },
    reminders: {
      status: vi.fn(async () => ({ permission: "authorized", armed: 0, planned: 0, notifierLastSeenAt: new Date().toISOString() })),
    },
    minds: { list: vi.fn(async () => ({ minds: [] })) },
    tags: { list: vi.fn(async () => ({ tags: [] })) },
    search: { search: vi.fn(async () => ({ hits: [] })) },
    app: {
      onDeepLink: vi.fn(() => () => {}),
      getDeepLink: vi.fn(async () => null),
      requestNotificationAuthorization: vi.fn(async () => true),
      openDeepLink: vi.fn(async () => {}),
    },
    onChange: vi.fn(callback => {
      listener = callback;
      return () => { listener = undefined; };
    }),
  } as unknown as MapWireBridge;

  return {
    wire,
    places,
    memories,
    emit: (entity = "place", id = "memory-1") => listener?.({ resync: false, changes: [{ entity, id, op: "update" }] }),
  };
}
