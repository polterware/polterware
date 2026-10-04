import { createSignal } from "solid-js";
import { DEFAULT_MAP_CENTER } from "@polterware/frontend-map";
import { browserBridge, type MapBridge } from "./bridge";
import {
  DISCLOSURE_KEY, WORLD_BOUNDS, errorText, geocodingEnabled, readDisclosureSeen, writeDisclosureSeen,
  type Honesty, type MapPlaceRow, type PlaceJSON, type SettingsJSON,
} from "./model";

export type MapBoundsQuery = { south: number; west: number; north: number; east: number };

export function createMapStore(getBridge: () => MapBridge = browserBridge) {
  const [places, setPlaces] = createSignal<MapPlaceRow[]>([]);
  const [bounds, setBounds] = createSignal<MapBoundsQuery>(WORLD_BOUNDS);
  const [center, setCenter] = createSignal(DEFAULT_MAP_CENTER);
  const [zoom, setZoom] = createSignal(11);
  const [selectedId, setSelectedId] = createSignal<string | null>(null);
  const [mapOnline, setMapOnline] = createSignal(false);
  const [provider, setProvider] = createSignal<"maplibre" | "pmtiles">("maplibre");
  const [geocoding, setGeocoding] = createSignal(false);
  const [disclosureSeen, setDisclosureSeen] = createSignal(readDisclosureSeen());
  const [loadState, setLoadState] = createSignal<Honesty>("queued");
  const [saveState, setSaveState] = createSignal<Honesty>("live");
  const [loadError, setLoadError] = createSignal("");
  const [saveError, setSaveError] = createSignal("");
  let unsubscribe: (() => void) | undefined;
  let users = 0;
  let io = Promise.resolve();
  let fitted = false;

  async function loadSettings(bridge: MapBridge): Promise<SettingsJSON | undefined> {
    const result = await bridge.settings.get({}) as { settings?: SettingsJSON };
    const settings = result.settings;
    setMapOnline(Boolean(settings?.mapOnline));
    setProvider(settings?.mapProvider === "pmtiles" ? "pmtiles" : "maplibre");
    setGeocoding(geocodingEnabled(settings));
    return settings;
  }

  async function loadPlaces(next = bounds()) {
    setLoadState("queued");
    setLoadError("");
    try {
      const bridge = getBridge();
      await loadSettings(bridge);
      const [placeResponse, memoryResponse] = await Promise.all([
        bridge.places.listInBounds(next) as Promise<{ places?: PlaceJSON[] }>,
        bridge.memories.list({ hasPlace: true, limit: 1000 }) as Promise<{ memories?: { id?: string; title?: string }[] }>,
      ]);
      const titles = new Map((memoryResponse.memories ?? []).map(row => [row.id ?? "", row.title ?? ""]));
      const rows = (placeResponse.places ?? []).map(place => ({
        memoryId: place.memoryId ?? "",
        title: titles.get(place.memoryId ?? "") || place.name || "Untitled place",
        name: place.name ?? "",
        lat: place.lat ?? 0,
        lng: place.lng ?? 0,
        radiusM: place.radiusM,
        event: place.event || "enter",
      })).filter(row => row.memoryId);
      setPlaces(rows);
      setBounds(next);
      if (!fitted && rows[0]) {
        setCenter({ lat: rows[0].lat, lng: rows[0].lng });
        fitted = true;
      }
      setLoadState("live");
    } catch (error) {
      const message = errorText(error);
      setLoadError(message);
      setLoadState(message.toLowerCase().includes("unavailable") || message.toLowerCase().includes("unreachable") ? "unavailable" : "failed");
    }
  }

  const store = {
    places, bounds, center, zoom, selectedId, mapOnline, provider, geocoding, disclosureSeen, loadState, saveState, loadError, saveError,
    tilesLive: () => mapOnline() && provider() !== "pmtiles",
    showDisclosure: () => !disclosureSeen(),
    markers: () => places().map(place => ({ id: place.memoryId, lat: place.lat, lng: place.lng, radius: place.radiusM })),
    connect() {
      users += 1;
      if (users === 1) {
        try {
          unsubscribe = getBridge().onChange(event => {
            if (event.resync || event.changes.some(change => ["place", "places", "memory", "memories", "settings"].includes(change.entity))) {
              io = io.catch(() => undefined).then(() => loadPlaces(bounds()));
            }
          });
        } catch (error) {
          setLoadState("unavailable");
          setLoadError(errorText(error));
        }
        void loadPlaces(WORLD_BOUNDS);
      }
      return () => {
        users = Math.max(0, users - 1);
        if (users === 0) { unsubscribe?.(); unsubscribe = undefined; }
      };
    },
    reload: () => loadPlaces(bounds()),
    setBounds(next: MapBoundsQuery, nextZoom?: number) {
      if (nextZoom !== undefined) setZoom(nextZoom);
      io = io.catch(() => undefined).then(() => loadPlaces(next));
    },
    select(id: string | null) {
      setSelectedId(id);
      const row = places().find(place => place.memoryId === id);
      if (row) setCenter({ lat: row.lat, lng: row.lng });
    },
    markDisclosureSeen() {
      writeDisclosureSeen();
      setDisclosureSeen(true);
    },
    async setOnline(online: boolean) {
      setSaveState("queued");
      setSaveError("");
      try {
        const result = await getBridge().settings.set({ keys: ["map.online"], settings: { mapOnline: online } }) as { settings?: SettingsJSON };
        setMapOnline(Boolean(result.settings?.mapOnline));
        writeDisclosureSeen();
        setDisclosureSeen(true);
        setSaveState("live");
      } catch (error) {
        setSaveError(errorText(error));
        setSaveState("failed");
      }
    },
    flush: () => io,
    dispose() { unsubscribe?.(); unsubscribe = undefined; users = 0; },
    disclosureKey: DISCLOSURE_KEY,
  };
  return store;
}

export type MapStore = ReturnType<typeof createMapStore>;
export const mapStore = createMapStore();
