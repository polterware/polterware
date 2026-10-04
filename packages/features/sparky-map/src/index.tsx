import type { SparkyFeature } from "@polterware/feature-sparky-shell/feature";
import { MapPage } from "./map-page";
import { PlaceCard } from "./place-card";
import { MapSettingsPanel } from "./settings-panel";

const feature: SparkyFeature = {
  id: "map",
  routes: [{ path: "/map", title: "Map", component: MapPage }],
  sidebar: [{ id: "map", label: "Map", path: "/map", icon: "MapPin" }],
  settingsPanels: [{ id: "map", label: "Tiles", group: "Map", component: MapSettingsPanel }],
  detailSections: [{ id: "place", order: 20, component: PlaceCard }],
  commands: [{
    id: "map.open",
    label: "Go to Map",
    keywords: ["map", "places", "location"],
    run: async () => {
      const wire = Reflect.get(globalThis, "sparky") as { app?: { openDeepLink?: (url: string) => Promise<void> } } | undefined;
      await wire?.app?.openDeepLink?.("sparky://map");
    },
  }],
};

export default feature;
export { MapPage, PlaceCard, MapSettingsPanel };
export { createMapStore, mapStore } from "./store";
export { createMapBridge, browserBridge } from "./bridge";
export { PLACE_NOTICE, TILE_NOTICE } from "./model";
