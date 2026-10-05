# w0-map status

- Result: done
- Branch / last commit: `sparky/w0-map` (pending commit)
- Acceptance criteria:
  - Recommendation + evidence table → **pass** (see below)
  - Working minimal `MapView` API prototype matching 05 → **pass** (`spikes/map/src/renderer/map/MapView.tsx` + `types.ts`)
  - Screenshot of dark map → **pass** (`spikes/map/evidence/dark-map-maplibre.png`)
  - Explicit viewport-leak statement + setting control → **pass** (see Privacy / `map.online` below)
  - O4 recorded in `00-decisions.md` → **pass**
- Deviations from the plan (and why):
  - PMTiles spike uses the public Florence sample archive (`protomaps.github.io`) rather than bundling a regional file; sufficient to prove protocol + dark style wiring. Full offline bundling is a W2-MAP / settings follow-up.
  - Spike is self-contained under `spikes/map/` (not wired into `apps/sparky-desktop`) per throwaway spike scope.
- Requests filed: none
- Follow-ups for other WS:
  - **W1-ELECTRON**: apply CSP entries from decision O4 in Main `session.webRequest` when `map.online` is true.
  - **W2-MAP**: implement production `MapView` in `packages/frontend/map` by porting spike providers; lazy-load `maplibre-gl` (~1 MB); keep Leaflet path behind a dev/fallback flag only if needed.
  - **W2-SETTINGS**: first-run disclosure copy for `map.online`; geocoding opt-in (disabled by default).

## Recommendation

**Ship MapLibre GL JS + OpenFreeMap vector tiles** (`https://tiles.openfreemap.org/styles/dark`) as the v1 default. Use **supercluster** for marker clustering (>200 markers). Treat **PMTiles** as the offline/region path (user-downloaded or bundled region file under Sparky data dir). Keep **Leaflet + OSM raster** as a documented fallback only — OSM tile policy is a poor fit for a installed desktop app with unpredictable usage.

## Evidence table

| Criterion | MapLibre + OpenFreeMap | Leaflet + OSM raster | MapLibre + PMTiles |
|-----------|------------------------|----------------------|---------------------|
| Marker + radius circle | Yes (`circle` layer + draggable pin) | Yes (`L.circle` + `L.circleMarker`) | Yes (same as MapLibre) |
| Pin drop / drag | Shift+click + draggable pin | Shift+click + draggable pin | Shift+click + draggable pin |
| Cluster 500 markers | Yes (`supercluster`) | Yes (`leaflet.markercluster`) | Yes (`supercluster`) |
| Dark style | Native vector dark style URL | Raster only; dark = invert/filter hack | Protomaps dark JSON + PMTiles source |
| Offline (`map.online` off) | Empty style + honest UI state | Tiles removed; grey canvas | Local/bundled `.pmtiles` possible (not in spike) |
| CSP (`connect-src` / `img-src`) | `connect-src https://tiles.openfreemap.org`; `worker-src blob:`; `img-src data: blob:` | `img-src https://tile.openstreetmap.org`; `connect-src` same | `connect-src` PMTiles host or `file:`; `worker-src blob:` |
| Prod bundle (main entry, node_modules) | **maplibre-gl ~1.03 MB** | **leaflet ~450 KB** + markercluster ~81 KB | maplibre-gl + **pmtiles ~15 KB** |
| Built spike renderer (all three, no code-split) | **2.13 MB JS** + 142 KB CSS (`evidence/bundle-sizes.json`) | (included above) | (included above) |
| Attribution | Auto via MapLibre; required: © OpenMapTiles / OSM ([OpenFreeMap](https://openfreemap.org/), 2026-10-04) | Must show © OSM; valid **User-Agent** required ([OSM tile policy](https://operations.osmfoundation.org/policies/tiles/), 2026-10-04) | © OSM (ODbL) + Protomaps ([protomaps.com](https://protomaps.com/docs/open-source), 2026-10-04) |
| Desktop ToS / commercial | Free public instance; **commercial allowed**; no API key; no SLA ([OpenFreeMap FAQ](https://openfreemap.org/), 2026-10-04) | **Heavy use discouraged**; no offline prefetch; generic library User-Agent blocked; best-effort only ([OSM tiles](https://operations.osmfoundation.org/policies/tiles/), 2026-10-04) | Open source (BSD + ODbL); self-host or use public samples; planet PMTiles for offline |
| Electron spike | **Pass** — dark São Paulo map, clustering, attribution visible | **Pass** — raster tiles load in spike | **Pass** — Florence sample PMTiles + dark style |

## MapView prototype API

Matches `05-search-attachments-map.md`:

```ts
type MapViewProps = {
  provider: "maplibre" | "leaflet" | "pmtiles";
  center: { lat: number; lng: number };
  zoom: number;
  markers: { id: string; lat: number; lng: number; radius?: number }[];
  selectedId?: string;
  online: boolean;
  onSelect?: (id: string) => void;
  onPick?: (pick: { lat: number; lng: number }) => void;
  class?: string;
};
```

Implementation: `spikes/map/src/renderer/map/MapView.tsx` with provider factories under `providers/`.

## Privacy / viewport leak (`map.online`)

When **`map.online` is on** (default after first-run disclosure), every pan/zoom fetches vector tiles (and glyph resources) for the **visible bounding box** from `tiles.openfreemap.org` (and any style font endpoints). The tile provider sees approximate geographic viewport, client IP, and request timing — not memory titles or place names.

When **`map.online` is off**, the spike shows an honest empty state listing place IDs; **no map network requests** are made. Production should mirror this (list of places + pin coordinates without tiles).

## Geocoding (opt-in, off by default)

| Provider | Terms (2026-10-04) | Fit for Sparky |
|----------|-------------------|----------------|
| [Nominatim](https://operations.osmfoundation.org/policies/nominatim/) | Max **1 req/s**; no autocomplete; valid User-Agent; end-user triggered only | Acceptable as **opt-in** manual search (not as-you-type) |
| OSM raster tiles | Not a geocoder | N/A |
| OpenFreeMap | Tiles only | N/A |

**Decision:** no geocoder in v1 default; manual lat/lng + pin drop. Optional Nominatim behind Settings with rate limit + disclosure.

## O4 decision (also in `00-decisions.md`)

- **Library:** MapLibre GL JS ^5.6
- **Tile provider:** OpenFreeMap dark style `https://tiles.openfreemap.org/styles/dark`
- **Clustering:** supercluster (MapLibre GeoJSON layers)
- **Attribution (UI):** rely on MapLibre control; footer may add “Map tiles from OpenFreeMap”
- **CSP (when `map.online`):** `connect-src 'self' https://tiles.openfreemap.org`; `worker-src 'self' blob:`; `img-src 'self' data: blob:`

## Verify

```bash
cd spikes/map && pnpm install && pnpm build && pnpm measure && pnpm screenshot
# Root gate (unchanged by spike — no workspace wiring):
bash tooling/verify-sparky.sh
```

All passed in this worktree on 2026-10-04.
