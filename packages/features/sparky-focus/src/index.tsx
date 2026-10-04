import type { SparkyFeature } from "@polterware/feature-sparky-shell/feature";
import { FocusPage } from "./focus-page";
import { FocusRecipeSection } from "./focus-recipe-section";
import { focusStore } from "./store";

function memoryIdFromFocusLink(url: URL): string | null {
  if (url.protocol !== "sparky:" || url.hostname !== "memory" || url.searchParams.get("action") !== "focus") return null;
  const id = url.pathname.replace(/^\//, "");
  return id || null;
}

const feature: SparkyFeature = {
  id: "focus",
  routes: [{ path: "/focus", title: "Focus", component: FocusPage }],
  sidebar: [{ id: "focus", label: "Focus", path: "/focus", icon: "Clock" }],
  commands: [{
    id: "focus.start",
    label: "Start focus",
    keywords: ["pomodoro", "timer", "focus", "start"],
    run: () => { void focusStore.startCommand(); },
  }],
  detailSections: [{ id: "focus-recipe", order: 40, component: FocusRecipeSection }],
  onDeepLink(url) {
    const id = memoryIdFromFocusLink(url);
    if (!id) return false;
    void focusStore.startMemory(id).catch(() => {});
    return false;
  },
};

export default feature;
export { FocusPage, FocusRecipeSection };
export { createFocusStore, focusStore } from "./store";
export { createFocusBridge, browserBridge } from "./bridge";
export { createFocusTimer } from "./timer";
