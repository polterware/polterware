import {
  ListChecks,
  SchemeIcon,
  type TabItem,
} from "@polterware/frontend-design-system";

/** Canonical Atlas tabs: the canvas, and the component list grouped by domain. */
export const ARCHITECTURE_TABS = [
  { id: "map", label: "Map" },
  { id: "components", label: "Components" },
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
        "Domains frame their components. Lines are the relationships between those components.",
      icon: SchemeIcon,
    },
  },
  {
    id: "components",
    label: "Components",
    tooltip: {
      title: "Components",
      description:
        "Every component, grouped by the domain it belongs to. Work cards sit in the side panel.",
      icon: ListChecks,
    },
  },
];

const ARCHITECTURE_TAB_MAP = {
  map: true,
  components: true,
} satisfies Record<string, boolean>;

function isArchitectureTabId(value: string): value is ArchitectureTabId {
  return Object.prototype.hasOwnProperty.call(ARCHITECTURE_TAB_MAP, value);
}

/**
 * Normalize Architecture in-feature tab ids.
 * Retired ids (canvas, hierarchy, views, dependencies, changes) map to the map.
 */
export function normalizeArchitectureTab(tab: string | undefined | null): ArchitectureTabId {
  const t = (tab ?? "map").trim().toLowerCase();
  if (isArchitectureTabId(t)) return t;
  return "map";
}
