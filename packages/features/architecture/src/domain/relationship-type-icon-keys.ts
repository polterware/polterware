/**
 * Semantic icon keys for relationship types (resolved to Hugeicons in UI).
 * Keys reuse the architecture type-icon vocabulary where possible.
 */
export const RELATIONSHIP_TYPE_ICON_KEYS = {
  // Creatable (focused catalog)
  depends_on: "git-branch",
  calls: "webhook",
  reads_from: "download",
  writes_to: "file-up",
  publishes_to: "zap",
  consumes_from: "inbox",
  routes_to: "route",
  deployed_on: "cloud",
  connects_to: "plug",

  // Full / legacy catalog
  authenticates_with: "key-round",
  deploys_to: "package",
  stores_in: "database",
  monitors: "activity",
  hosted_on: "server",
  replicates_to: "copy",
  synchronizes_with: "refresh-cw",
  exposes: "external-link",
  contains: "box",
  related: "link",
  custom: "pencil",
} as const;

export type RelationshipTypeIconKey =
  (typeof RELATIONSHIP_TYPE_ICON_KEYS)[keyof typeof RELATIONSHIP_TYPE_ICON_KEYS];

function isRelationshipTypeIconKey(
  type: string,
): type is keyof typeof RELATIONSHIP_TYPE_ICON_KEYS {
  return Object.prototype.hasOwnProperty.call(RELATIONSHIP_TYPE_ICON_KEYS, type);
}

export function relationshipTypeIconKey(type: string): string {
  return (
    isRelationshipTypeIconKey(type) ? RELATIONSHIP_TYPE_ICON_KEYS[type] :
    "link"
  );
}

export function hasRelationshipTypeIcon(type: string): boolean {
  return isRelationshipTypeIconKey(type);
}
