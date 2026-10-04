/** Prototype MapView API — matches plans/sparky-next/05-search-attachments-map.md §Map */
export type MapMarker = {
  id: string;
  lat: number;
  lng: number;
  radius?: number;
};

export type MapCenter = {
  lat: number;
  lng: number;
};

export type MapPick = {
  lat: number;
  lng: number;
};

export type MapProviderId = "maplibre" | "leaflet" | "pmtiles";

export type MapViewProps = {
  provider: MapProviderId;
  center: MapCenter;
  zoom: number;
  markers: MapMarker[];
  selectedId?: string;
  online: boolean;
  onSelect?: (id: string) => void;
  onPick?: (pick: MapPick) => void;
  class?: string;
};

export type MapProviderHandle = {
  destroy: () => void;
  setCenter: (center: MapCenter, zoom: number) => void;
  setMarkers: (markers: MapMarker[], selectedId?: string) => void;
  setOnline: (online: boolean) => void;
};
