import { createSignal, Show } from "solid-js";
import { MapView } from "./map/MapView";
import { CSP_ENTRIES, cspForProvider } from "./map/csp";
import { DEFAULT_MARKERS, generateClusterMarkers } from "./map/markers";
import type { MapProviderId } from "./map/types";

const PROVIDERS: { id: MapProviderId; label: string }[] = [
  { id: "maplibre", label: "MapLibre + OpenFreeMap" },
  { id: "leaflet", label: "Leaflet + OSM raster" },
  { id: "pmtiles", label: "MapLibre + PMTiles" },
];

export function App() {
  const [provider, setProvider] = createSignal<MapProviderId>("maplibre");
  const [online, setOnline] = createSignal(true);
  const [clusterMode, setClusterMode] = createSignal(false);
  const [selectedId, setSelectedId] = createSignal<string | undefined>("sparky-hq");
  const [lastPick, setLastPick] = createSignal<string>("—");
  const [markers, setMarkers] = createSignal(DEFAULT_MARKERS);

  const center = () =>
    provider() === "pmtiles"
      ? { lat: 43.7696, lng: 11.2558 }
      : { lat: -23.5505, lng: -46.6333 };

  return (
    <div class="flex h-full flex-col">
      <header class="flex flex-wrap items-center gap-3 border-b border-[var(--color-border-default)] px-4 py-3">
        <h1 class="text-sm font-medium text-[var(--color-text-primary)]">W0-MAP spike</h1>
        <div class="flex flex-wrap gap-1">
          {PROVIDERS.map((p) => (
            <button
              type="button"
              class={`rounded-md px-2.5 py-1 text-xs ${
                provider() === p.id
                  ? "bg-[var(--color-surface-selected)] text-[var(--color-text-primary)]"
                  : "text-[var(--color-text-secondary)] hover:bg-[var(--color-surface-hover)]"
              }`}
              onClick={() => setProvider(p.id)}
            >
              {p.label}
            </button>
          ))}
        </div>
        <label class="flex items-center gap-2 text-xs text-[var(--color-text-secondary)]">
          <input
            type="checkbox"
            checked={online()}
            onChange={(e) => setOnline(e.currentTarget.checked)}
          />
          map.online
        </label>
        <button
          type="button"
          class="rounded-md px-2.5 py-1 text-xs text-[var(--color-text-secondary)] hover:bg-[var(--color-surface-hover)]"
          onClick={() => {
            setClusterMode(!clusterMode());
            setMarkers(clusterMode() ? DEFAULT_MARKERS : generateClusterMarkers(500));
          }}
        >
          {clusterMode() ? "3 markers" : "500 markers (cluster)"}
        </button>
        <span class="text-xs text-[var(--color-text-muted)]">
          Shift+click or drag pin to drop · selected: {selectedId() ?? "none"} · pick: {lastPick()}
        </span>
      </header>

      <div class="relative min-h-0 flex-1">
        <Show
          when={online()}
          fallback={
            <div class="absolute inset-0 z-10 flex items-center justify-center bg-[var(--color-app-background)] p-8">
              <div class="max-w-md text-center">
                <p class="text-sm font-medium text-[var(--color-text-primary)]">
                  Map tiles are off
                </p>
                <p class="mt-2 text-xs text-[var(--color-text-secondary)]">
                  When <code class="text-[var(--color-text-muted)]">map.online</code> is disabled,
                  Sparky shows an honest empty state with the list of places instead of implying
                  live tiles.
                </p>
                <p class="mt-3 text-xs text-[var(--color-text-muted)]">
                  Places in this spike: {markers().map((m) => m.id).join(", ")}
                </p>
              </div>
            </div>
          }
        >
          <MapView
            provider={provider()}
            center={center()}
            zoom={provider() === "pmtiles" ? 12 : 11}
            markers={markers()}
            selectedId={selectedId()}
            online={online()}
            onSelect={setSelectedId}
            onPick={(pick) => setLastPick(`${pick.lat.toFixed(5)}, ${pick.lng.toFixed(5)}`)}
            class="h-full w-full"
          />
        </Show>
      </div>

      <footer class="border-t border-[var(--color-border-default)] px-4 py-2 text-[10px] text-[var(--color-text-muted)]">
        CSP ({provider()}): {cspForProvider(provider())} · connect-src:{" "}
        {CSP_ENTRIES[provider()].connectSrc.join(", ")}
      </footer>
    </div>
  );
}
