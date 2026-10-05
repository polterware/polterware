import type { SparkyFeature } from "@polterware/feature-sparky-shell/feature";
import { lazy } from "solid-js";
import { browserBridge } from "./bridge";
import { mapStore } from "./store";
// Register components eagerly, but load map/editor code only when the shell renders them.
const MapPage = lazy(() => import("./map-page").then(module => ({ default: module.MapPage })));
const PlaceCard = lazy(() => import("./place-card").then(module => ({ default: module.PlaceCard })));
const MapSettingsPanel = lazy(() => import("./settings-panel").then(module => ({ default: module.MapSettingsPanel })));

const feature: SparkyFeature = {
  id: "map",
  routes: [{ path: "/map", title: "Map", component: MapPage }],
  sidebar: [{ id: "map", label: "Map", path: "/map", icon: "MapPin" }],
  onDeepLink: url => {
    if (url.protocol === "sparky:" && url.hostname === "map") void mapStore.reload();
    // Returning false lets the shell navigate to the registered /map route.
    return false;
  },
  settingsPanels: [{ id: "map-tiles", label: "Tiles", group: "Map", component: MapSettingsPanel }],
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
