/** Pure icon key list — domain ids, no icon-library imports (safe for node unit tests). */
export const TYPE_ICON_KEYS = [
  "globe",
  "smartphone",
  "monitor",
  "webhook",
  "boxes",
  "cog",
  "clock",
  "zap",
  "database",
  "memory-stick",
  "list-ordered",
  "activity",
  "hard-drive",
  "search",
  "warehouse",
  "server",
  "scale",
  "route",
  "cloud",
  "network",
  "globe-2",
  "map-pin",
  "git-branch",
  "package",
  "git-merge",
  "hammer",
  "archive",
  "external-link",
  "cloud-cog",
  "plug",
  "shield",
  "credit-card",
  "link",
  "layout-dashboard",
  "square",
  "sticky-note",
  "box",
  "link-2",
  "download",
  "file-up",
] as const;

export type TypeIconKey = (typeof TYPE_ICON_KEYS)[number];

const KEY_SET = new Set<string>(TYPE_ICON_KEYS);

export function hasTypeIcon(name: string): boolean {
  return KEY_SET.has(name);
}
