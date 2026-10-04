import maplibregl, { type GeoJSONSource, type Map, type Marker } from "maplibre-gl";
import { Protocol } from "pmtiles";
import Supercluster from "supercluster";
import type { MapCenter, MapMarker, MapPick, MapProviderHandle } from "../types";

const BRAND = "#01dd96";

/** Regional PMTiles sample (Florence) — small enough for spike; demonstrates offline-capable vector tiles. */
const PMTILES_URL = "https://protomaps.github.io/PMTiles/protomaps(vector)ODbL_firenze.pmtiles";
const DARK_STYLE_BASE = "https://protomaps.github.io/basemaps-assets/styles/v3/dark.json";

type ClusterProps = { cluster: boolean; cluster_id?: number; point_count?: number; id?: string };

function markersToGeoJson(markers: MapMarker[]): GeoJSON.FeatureCollection {
  return {
    type: "FeatureCollection",
    features: markers.map((m) => ({
      type: "Feature",
      properties: { id: m.id, radius: m.radius ?? 0 },
      geometry: { type: "Point", coordinates: [m.lng, m.lat] },
    })),
  };
}

async function buildPmtilesStyle(online: boolean): Promise<string | maplibregl.StyleSpecification> {
  if (!online) {
    return { version: 8, sources: {}, layers: [] };
  }

  const response = await fetch(DARK_STYLE_BASE);
  const style = (await response.json()) as maplibregl.StyleSpecification & {
    sources: Record<string, { url?: string; type?: string }>;
  };

  for (const source of Object.values(style.sources)) {
    if (source.url?.startsWith("pmtiles://")) {
      source.url = `pmtiles://${PMTILES_URL}`;
    }
  }

  return style;
}

export function createPmtilesProvider(
  container: HTMLElement,
  options: {
    center: MapCenter;
    zoom: number;
    markers: MapMarker[];
    selectedId?: string;
    online: boolean;
    onSelect?: (id: string) => void;
    onPick?: (pick: MapPick) => void;
  },
): MapProviderHandle {
  const protocol = new Protocol();
  maplibregl.addProtocol("pmtiles", protocol.tile);

  const clusterIndex = new Supercluster<{ id: string; radius: number }>({
    radius: 50,
    maxZoom: 16,
  });

  let map: Map | null = null;
  let pickMarker: Marker | null = null;
  let radiusLayers: string[] = [];
  let destroyed = false;

  const renderRadiusCircles = (markers: MapMarker[]) => {
    if (!map) return;
    for (const id of radiusLayers) {
      if (map.getLayer(id)) map.removeLayer(id);
      if (map.getSource(id)) map.removeSource(id);
    }
    radiusLayers = [];

    for (const marker of markers) {
      if (!marker.radius) continue;
      const sourceId = `radius-${marker.id}`;
      radiusLayers.push(sourceId);
      map.addSource(sourceId, {
        type: "geojson",
        data: {
          type: "Feature",
          geometry: { type: "Point", coordinates: [marker.lng, marker.lat] },
          properties: {},
        },
      });
      map.addLayer({
        id: sourceId,
        type: "circle",
        source: sourceId,
        paint: {
          "circle-radius": {
            stops: [
              [0, 0],
              [20, marker.radius / 0.075],
            ],
            base: 2,
          },
          "circle-color": BRAND,
          "circle-opacity": 0.12,
          "circle-stroke-color": BRAND,
          "circle-stroke-width": 1,
          "circle-stroke-opacity": 0.5,
        },
      });
    }
  };

  const updateClusters = (markers: MapMarker[], selectedId?: string) => {
    if (!map) return;
    const geojson = markersToGeoJson(markers);
    clusterIndex.load(
      geojson.features.map((f) => ({
        type: "Feature" as const,
        properties: {
          id: String(f.properties?.id ?? ""),
          radius: Number(f.properties?.radius ?? 0),
        },
        geometry: f.geometry as GeoJSON.Point,
      })),
    );

    const bounds = map.getBounds();
    const zoom = Math.floor(map.getZoom());
    const clusters = clusterIndex.getClusters(
      [bounds.getWest(), bounds.getSouth(), bounds.getEast(), bounds.getNorth()],
      zoom,
    );

    const source = map.getSource("markers") as GeoJSONSource | undefined;
    if (source) {
      source.setData({
        type: "FeatureCollection",
        features: clusters.map((c) => ({
          type: "Feature",
          properties: c.properties as ClusterProps & { id: string },
          geometry: c.geometry,
        })),
      });
    }

    renderRadiusCircles(markers);

    if (selectedId) {
      const selected = markers.find((m) => m.id === selectedId);
      if (selected && pickMarker) {
        pickMarker.setLngLat([selected.lng, selected.lat]);
      }
    }
  };

  const wireMap = () => {
    if (!map) return;

    map.addSource("markers", {
      type: "geojson",
      data: { type: "FeatureCollection", features: [] },
    });

    map.addLayer({
      id: "clusters",
      type: "circle",
      source: "markers",
      filter: ["has", "point_count"],
      paint: {
        "circle-color": BRAND,
        "circle-radius": ["step", ["get", "point_count"], 16, 10, 20, 50, 26],
        "circle-opacity": 0.85,
      },
    });

    map.addLayer({
      id: "cluster-count",
      type: "symbol",
      source: "markers",
      filter: ["has", "point_count"],
      layout: {
        "text-field": "{point_count_abbreviated}",
        "text-size": 11,
      },
      paint: { "text-color": "#070707" },
    });

    map.addLayer({
      id: "unclustered",
      type: "circle",
      source: "markers",
      filter: ["!", ["has", "point_count"]],
      paint: {
        "circle-color": BRAND,
        "circle-radius": 7,
        "circle-stroke-color": "#070707",
        "circle-stroke-width": 2,
      },
    });

    pickMarker = new maplibregl.Marker({ color: BRAND, draggable: true })
      .setLngLat([options.center.lng, options.center.lat])
      .addTo(map);

    pickMarker.on("dragend", () => {
      const lngLat = pickMarker?.getLngLat();
      if (lngLat) options.onPick?.({ lat: lngLat.lat, lng: lngLat.lng });
    });

    map.on("click", "clusters", (e) => {
      const feature = e.features?.[0];
      if (!feature || !map) return;
      const clusterId = feature.properties?.cluster_id as number;
      const expansionZoom = clusterIndex.getClusterExpansionZoom(clusterId);
      const [lng, lat] = (feature.geometry as GeoJSON.Point).coordinates;
      map.easeTo({ center: [lng, lat], zoom: expansionZoom });
    });

    map.on("click", "unclustered", (e) => {
      const id = e.features?.[0]?.properties?.id as string | undefined;
      if (id) options.onSelect?.(id);
    });

    map.on("click", (e) => {
      if (e.originalEvent.shiftKey) {
        pickMarker?.setLngLat(e.lngLat);
        options.onPick?.({ lat: e.lngLat.lat, lng: e.lngLat.lng });
      }
    });

    map.on("moveend", () => updateClusters(options.markers, options.selectedId));
    updateClusters(options.markers, options.selectedId);
  };

  const init = async () => {
    if (destroyed) return;

    const style = await buildPmtilesStyle(options.online);

    map = new maplibregl.Map({
      container,
      style,
      center: [options.center.lng, options.center.lat],
      zoom: options.zoom,
      attributionControl: { compact: true },
    });

    map.addControl(new maplibregl.NavigationControl({ showCompass: false }), "top-right");
    map.on("load", wireMap);
  };

  void init();

  return {
    destroy: () => {
      destroyed = true;
      pickMarker?.remove();
      map?.remove();
      map = null;
      maplibregl.removeProtocol("pmtiles");
    },
    setCenter: (center, zoom) => {
      map?.easeTo({ center: [center.lng, center.lat], zoom });
    },
    setMarkers: (markers, selectedId) => {
      options.markers = markers;
      updateClusters(markers, selectedId);
    },
    setOnline: async (online) => {
      if (!map || options.online === online) return;
      options.online = online;
      const style = await buildPmtilesStyle(online);
      map.setStyle(style);
      map.once("style.load", wireMap);
    },
  };
}
