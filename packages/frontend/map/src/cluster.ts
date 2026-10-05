/// <reference path="./supercluster.d.ts" />
import Supercluster from "supercluster";
import type { MapBounds, MapMarker } from "./types";

export type ClusterPoint = {
  kind: "point";
  id: string;
  lat: number;
  lng: number;
  radius?: number;
};

export type ClusterGroup = {
  kind: "cluster";
  id: number;
  lat: number;
  lng: number;
  count: number;
  expansionZoom: number;
};

export type ClusterFeature = ClusterPoint | ClusterGroup;

type PointProps = { id: string; radius: number };

const DEFAULT_RADIUS = 50;
const DEFAULT_MAX_ZOOM = 16;

export function bboxFromBounds(bounds: MapBounds): [number, number, number, number] {
  return [bounds.west, bounds.south, bounds.east, bounds.north];
}

export function createClusterIndex(markers: readonly MapMarker[], options?: { radius?: number; maxZoom?: number }) {
  const index = new Supercluster<PointProps>({
    radius: options?.radius ?? DEFAULT_RADIUS,
    maxZoom: options?.maxZoom ?? DEFAULT_MAX_ZOOM,
  });
  index.load(markers.map(marker => ({
    type: "Feature" as const,
    properties: { id: marker.id, radius: marker.radius ?? 0 },
    geometry: { type: "Point" as const, coordinates: [marker.lng, marker.lat] },
  })));
  return index;
}

export function clusterMarkers(
  markers: readonly MapMarker[],
  bounds: MapBounds,
  zoom: number,
  options?: { radius?: number; maxZoom?: number },
): ClusterFeature[] {
  const index = createClusterIndex(markers, options);
  const level = Math.max(0, Math.floor(zoom));
  return index.getClusters(bboxFromBounds(bounds), level).map(feature => {
    const [lng, lat] = feature.geometry.coordinates;
    const props = feature.properties;
    if ("cluster" in props && props.cluster) {
      const clusterId = props.cluster_id;
      return {
        kind: "cluster",
        id: clusterId,
        lat,
        lng,
        count: props.point_count,
        expansionZoom: index.getClusterExpansionZoom(clusterId),
      };
    }
    const point = props as { id: string; radius: number };
    return {
      kind: "point",
      id: point.id,
      lat,
      lng,
      radius: point.radius > 0 ? point.radius : undefined,
    };
  });
}

/** Synthetic cluster fixture around São Paulo — the W0-MAP evidence set. */
export function generateClusterMarkers(count = 500): MapMarker[] {
  const centerLat = -23.5505;
  const centerLng = -46.6333;
  const markers: MapMarker[] = [];
  for (let i = 0; i < count; i++) {
    const angle = (i / count) * Math.PI * 2 * 7;
    const spread = 0.02 + (i % 50) * 0.0008;
    markers.push({
      id: `m-${i}`,
      lat: centerLat + Math.sin(angle) * spread,
      lng: centerLng + Math.cos(angle) * spread,
      radius: i % 17 === 0 ? 200 + (i % 5) * 100 : undefined,
    });
  }
  return markers;
}

export const DEFAULT_MAP_CENTER = { lat: -23.5505, lng: -46.6333 };
