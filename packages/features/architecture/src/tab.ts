import {
  Eye,
  ListChecks,
  SchemeIcon,
  type TabItem,
} from "@polterware/frontend-design-system";

/** Canonical Atlas tabs: the draggable graph, the node hierarchy, and saved views. */
export const ARCHITECTURE_TABS = [
  { id: "map", label: "Map" },
  { id: "hierarchy", label: "Hierarchy" },
  { id: "views", label: "Views" },
] as const;

export type ArchitectureTabId = (typeof ARCHITECTURE_TABS)[number]["id"];

/** PageToolbar items with descriptive hover tooltips. */
export const ARCHITECTURE_TAB_ITEMS: TabItem[] = [
  {
    id: "map",
    label: "Map",
    tooltip: {
      title: "System map",
      description:
        "Draw and arrange the project's components and relationships on a freeform canvas.",
      icon: SchemeIcon,
    },
  },
  {
    id: "hierarchy",
    label: "Hierarchy",
    tooltip: {
      title: "Hierarchy",
      description:
        "Zoom the project from system to domain to component. Status, contract, and work sit in the side panel.",
      icon: ListChecks,
    },
  },
  {
    id: "views",
    label: "Views",
    tooltip: {
      title: "Saved viewpoints",
      description:
        "Named slices of the architecture so you can jump between focused maps without rebuilding the canvas.",
      icon: Eye,
    },
  },
];

const ARCHITECTURE_TAB_MAP = {
  map: true,
  hierarchy: true,
  views: true,
} satisfies Record<string, boolean>;

function isArchitectureTabId(value: string): value is ArchitectureTabId {
  return Object.prototype.hasOwnProperty.call(ARCHITECTURE_TAB_MAP, value);
}

/**
 * Normalize Architecture in-feature tab ids.
 * Retired ids (canvas, Components list, dependencies, changes) map to the map.
 */
export function normalizeArchitectureTab(tab: string | undefined | null): ArchitectureTabId {
  const t = (tab ?? "map").trim().toLowerCase();
  if (isArchitectureTabId(t)) return t;
  // "canvas" is the retired id of the graph, before the hierarchy took the name.
  return "map";
}
