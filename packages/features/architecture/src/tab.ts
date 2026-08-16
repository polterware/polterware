/** Canonical Architecture tabs (docs/spec.md §8.2). */
export const ARCHITECTURE_TABS = [
  { id: "canvas", label: "Canvas" },
  { id: "components", label: "Components" },
  { id: "views", label: "Views" },
] as const;

export type ArchitectureTabId = (typeof ARCHITECTURE_TABS)[number]["id"];

const ARCHITECTURE_TAB_MAP = {
  canvas: true,
  components: true,
  views: true,
} satisfies Record<string, boolean>;

function isArchitectureTabId(val: string): val is ArchitectureTabId {
  return Object.prototype.hasOwnProperty.call(ARCHITECTURE_TAB_MAP, val);
}

/**
 * Normalize Architecture in-feature tab ids.
 * `components` MUST remain `components` — never rewrite to canvas.
 * Retired tabs may fall back to canvas.
 */
export function normalizeArchitectureTab(tab: string | undefined | null): ArchitectureTabId {
  const t = (tab ?? "canvas").trim().toLowerCase();
  if (isArchitectureTabId(t)) return t;
  // Retired architecture surfaces map to the primary canvas view.
  if (t === "dependencies" || t === "changes") return "canvas";
  return "canvas";
}
