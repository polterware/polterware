import L from "leaflet";
import "leaflet/dist/leaflet.css";
import "leaflet.markercluster";
import "leaflet.markercluster/dist/MarkerCluster.css";
import "leaflet.markercluster/dist/MarkerCluster.Default.css";
import type { MapCenter, MapMarker, MapPick, MapProviderHandle } from "../types";

const BRAND = "#01dd96";
const OSM_TILE_URL = "https://tile.openstreetmap.org/{z}/{x}/{y}.png";
const OSM_ATTRIBUTION =
  '&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a> contributors';

export function createLeafletProvider(
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
  let map: L.Map | null = null;
  let tileLayer: L.TileLayer | null = null;
  let clusterGroup: L.MarkerClusterGroup | null = null;
  let pickMarker: L.Marker | null = null;
  const radiusCircles = new Map<string, L.Circle>();

  const renderMarkers = (markers: MapMarker[], selectedId?: string) => {
    if (!map || !clusterGroup) return;

    clusterGroup.clearLayers();
    radiusCircles.forEach((circle) => circle.remove());
    radiusCircles.clear();

    for (const marker of markers) {
      const m = L.circleMarker([marker.lat, marker.lng], {
        radius: marker.id === selectedId ? 9 : 7,
        color: "#070707",
        weight: 2,
        fillColor: BRAND,
        fillOpacity: 0.9,
      });
      m.on("click", () => options.onSelect?.(marker.id));
      clusterGroup.addLayer(m);

      if (marker.radius) {
        const circle = L.circle([marker.lat, marker.lng], {
          radius: marker.radius,
          color: BRAND,
          fillColor: BRAND,
          fillOpacity: 0.12,
          weight: 1,
        }).addTo(map);
        radiusCircles.set(marker.id, circle);
      }
    }

    if (selectedId) {
      const selected = markers.find((m) => m.id === selectedId);
      if (selected && pickMarker) {
        pickMarker.setLatLng([selected.lat, selected.lng]);
      }
    }
  };

  map = L.map(container, {
    center: [options.center.lat, options.center.lng],
    zoom: options.zoom,
    zoomControl: true,
  });

  if (options.online) {
    tileLayer = L.tileLayer(OSM_TILE_URL, {
      attribution: OSM_ATTRIBUTION,
      maxZoom: 19,
    }).addTo(map);
  }

  clusterGroup = L.markerClusterGroup({ maxClusterRadius: 50, disableClusteringAtZoom: 16 });
  map.addLayer(clusterGroup);

  pickMarker = L.marker([options.center.lat, options.center.lng], { draggable: true }).addTo(map);
  pickMarker.on("dragend", () => {
    const latLng = pickMarker?.getLatLng();
    if (latLng) options.onPick?.({ lat: latLng.lat, lng: latLng.lng });
  });

  map.on("click", (e) => {
    if (e.originalEvent.shiftKey) {
      pickMarker?.setLatLng(e.latlng);
      options.onPick?.({ lat: e.latlng.lat, lng: e.latlng.lng });
    }
  });

  renderMarkers(options.markers, options.selectedId);

  return {
    destroy: () => {
      radiusCircles.forEach((circle) => circle.remove());
      map?.remove();
      map = null;
    },
    setCenter: (center, zoom) => {
      map?.setView([center.lat, center.lng], zoom);
    },
    setMarkers: (markers, selectedId) => {
      options.markers = markers;
      renderMarkers(markers, selectedId);
    },
    setOnline: (online) => {
      if (!map || options.online === online) return;
      options.online = online;
      if (online && !tileLayer) {
        tileLayer = L.tileLayer(OSM_TILE_URL, {
          attribution: OSM_ATTRIBUTION,
          maxZoom: 19,
        }).addTo(map);
      } else if (!online && tileLayer) {
        tileLayer.remove();
        tileLayer = null;
      }
    },
  };
}
