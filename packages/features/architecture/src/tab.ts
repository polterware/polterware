import {
  Eye,
  SchemeIcon,
  type TabItem,
} from "@polterware/frontend-design-system";

/** Canonical Architecture tabs — Canvas + Views (Components list removed). */
export const ARCHITECTURE_TABS = [
  { id: "canvas", label: "Canvas" },
  { id: "views", label: "Views" },
] as const;

export type ArchitectureTabId = (typeof ARCHITECTURE_TABS)[number]["id"];

/** PageToolbar items with descriptive hover tooltips. */
export const ARCHITECTURE_TAB_ITEMS: TabItem[] = [
  {
    id: "canvas",
    label: "Canvas",
    tooltip: {
      title: "System map",
      description:
        "Draw and arrange the project's components and relationships on a freeform canvas. This is the default landing view after you open a project.",
      icon: SchemeIcon,
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
  canvas: true,
  views: true,
} satisfies Record<string, boolean>;

function isArchitectureTabId(value: string): value is ArchitectureTabId {
  return Object.prototype.hasOwnProperty.call(ARCHITECTURE_TAB_MAP, value);
}

/**
 * Normalize Architecture in-feature tab ids.
 * Retired Surfaces (Components list, dependencies, changes) map to canvas.
 */
export function normalizeArchitectureTab(tab: string | undefined | null): ArchitectureTabId {
  const t = (tab ?? "canvas").trim().toLowerCase();
  if (isArchitectureTabId(t)) return t;
  if (t === "components" || t === "dependencies" || t === "changes") return "canvas";
  return "canvas";
}
