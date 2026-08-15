import type { ArchViewCard, ArchViewMember } from "./extra-fixtures";

/** Membership list is known only when the fixture provides `members`. */
export function hasMembershipList(
  view: Pick<ArchViewCard, "members"> | null | undefined,
): boolean {
  return Array.isArray(view?.members);
}

export function membershipRows(
  view: Pick<ArchViewCard, "members"> | null | undefined,
): ArchViewMember[] {
  return Array.isArray(view?.members) ? view.members : [];
}

export function membershipEmptyCopy(
  view: Pick<ArchViewCard, "members" | "nodeCount"> | null | undefined,
): string {
  if (!hasMembershipList(view)) {
    return "Membership list not available in mock.";
  }
  if ((view?.members?.length ?? 0) === 0) {
    return "No members in this view.";
  }
  return "";
}

export function isDescriptionEmpty(description: string | null | undefined): boolean {
  return !description || description.trim().length === 0;
}

export function emptyDescriptionCopy(): string {
  return "No description provided for this view.";
}

export function hasEnvironments(
  view: Pick<ArchViewCard, "environments"> | null | undefined,
): boolean {
  return Array.isArray(view?.environments) && view.environments.length > 0;
}

export function hasTags(view: Pick<ArchViewCard, "tags"> | null | undefined): boolean {
  return Array.isArray(view?.tags) && view.tags.length > 0;
}

/** True when fixture has enrichment beyond name/description/nodeCount/updated. */
export function isViewEnriched(view: ArchViewCard): boolean {
  return Boolean(
    view.owner ||
      hasEnvironments(view) ||
      hasTags(view) ||
      view.purpose ||
      hasMembershipList(view) ||
      view.notes ||
      view.filterSummary,
  );
}
