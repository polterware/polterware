import type { SparkyFeature } from "@polterware/feature-sparky-shell/feature";
import { browserBridge } from "./bridge";
import { mapStore } from "./store";
import { MapPage } from "./map-page";
import { PlaceCard } from "./place-card";
import { MapSettingsPanel } from "./settings-panel";

const feature: SparkyFeature = {
  id: "map",
  routes: [{ path: "/map", title: "Map", component: MapPage }],
  sidebar: [{ id: "map", label: "Map", path: "/map", icon: "MapPin" }],
  onDeepLink: url => {
    if (url.protocol === "sparky:" && url.hostname === "map") void mapStore.reload();
    // Returning false lets the shell navigate to the registered /map route.
    return false;
  },
  settingsPanels: [{ id: "map", label: "Tiles", group: "Map", component: MapSettingsPanel }],
  detailSections: [{ id: "place", order: 20, component: PlaceCard }],
  commands: [{
    id: "map.open",
    label: "Go to Map",
    keywords: ["map", "places", "location"],
    run: () => browserBridge().app.openDeepLink("sparky://map"),
  }],
};

export default feature;
export { MapPage, PlaceCard, MapSettingsPanel };
export { createMapStore, mapStore } from "./store";
export { createMapBridge, browserBridge } from "./bridge";
export { PLACE_NOTICE, TILE_NOTICE } from "./model";
