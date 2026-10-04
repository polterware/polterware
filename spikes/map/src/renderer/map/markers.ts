import type { MapMarker } from "./types";

/** Synthetic cluster test set — 500 markers around São Paulo. */
export function generateClusterMarkers(count = 500): MapMarker[] {
  const centerLat = -23.5505;
  const centerLng = -46.6333;
  const markers: MapMarker[] = [];

  for (let i = 0; i < count; i++) {
    const angle = (i / count) * Math.PI * 2 * 7;
    const radius = 0.02 + (i % 50) * 0.0008;
    markers.push({
      id: `m-${i}`,
      lat: centerLat + Math.sin(angle) * radius,
      lng: centerLng + Math.cos(angle) * radius,
      radius: i % 17 === 0 ? 200 + (i % 5) * 100 : undefined,
    });
  }

  return markers;
}

export const DEFAULT_MARKERS: MapMarker[] = [
  { id: "sparky-hq", lat: -23.5505, lng: -46.6333, radius: 500 },
  { id: "pin-drop", lat: -23.561, lng: -46.655, radius: 250 },
  { id: "memory-3", lat: -23.545, lng: -46.62 },
];
