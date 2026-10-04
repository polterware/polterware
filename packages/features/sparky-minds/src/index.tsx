import type { Component } from "solid-js";
import type { SparkyFeature } from "@polterware/feature-sparky-shell/feature";
import { MindPage } from "./mind-page";
import { MindsSidebarTree } from "./minds-tree";

type MindsFeature = SparkyFeature & {
  sidebarTree?: { id: string; component: Component };
};

const feature: MindsFeature = {
  id: "minds",
  routes: [
    { path: "/minds", component: MindPage, title: "Minds" },
    { path: "/mind/:id", component: MindPage, title: "Mind" },
  ],
  sidebar: [{ id: "minds", label: "Minds", path: "/minds", icon: "brain" }],
  sidebarTree: { id: "minds", component: MindsSidebarTree },
};

export default feature;
export { MindPage, MindsSidebarTree };
