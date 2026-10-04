import type { MapProviderId } from "./types";

/** CSP fragments required per provider when map.online is enabled. */
export function cspForProvider(provider: MapProviderId): string {
  const shared = [
    "default-src 'self'",
    "script-src 'self'",
    "style-src 'self' 'unsafe-inline'",
    "font-src 'self' data:",
    "worker-src 'self' blob:",
  ];

  const connect: string[] = ["'self'"];
  const img: string[] = ["'self' data: blob:"];

  switch (provider) {
    case "maplibre":
      connect.push("https://tiles.openfreemap.org", "https://*.openfreemap.org");
      break;
    case "leaflet":
      img.push("https://tile.openstreetmap.org", "https://*.tile.openstreetmap.org");
      connect.push("https://tile.openstreetmap.org");
      break;
    case "pmtiles":
      connect.push(
        "https://protomaps.github.io",
        "https://data.source.coop",
        "https://*.source.coop",
      );
      break;
  }

  return [...shared, `connect-src ${connect.join(" ")}`, `img-src ${img.join(" ")}`].join("; ");
}

export const CSP_ENTRIES: Record<MapProviderId, { connectSrc: string[]; imgSrc: string[] }> = {
  maplibre: {
    connectSrc: ["https://tiles.openfreemap.org"],
    imgSrc: ["data:", "blob:"],
  },
  leaflet: {
    connectSrc: ["https://tile.openstreetmap.org"],
    imgSrc: ["https://tile.openstreetmap.org", "https://*.tile.openstreetmap.org"],
  },
  pmtiles: {
    connectSrc: ["https://data.source.coop", "https://protomaps.github.io"],
    imgSrc: ["data:", "blob:"],
  },
};
