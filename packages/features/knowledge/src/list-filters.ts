import type { WorkspaceSource, WorkspaceSpecFeature } from "@polterware/feature-shell";
import { checklistHasOpenItems } from "./spec-templates";

export type SpecChecklistFilter = "any" | "open" | "clear";
export type LinkedFilter = "any" | "linked" | "unlinked";

export type SpecListFilters = {
  statuses: readonly string[];
  checklist: SpecChecklistFilter;
  query: string;
};

export type SourceListFilters = {
  linked: LinkedFilter;
  pathPrefixes: readonly string[];
  query: string;
};

export const EMPTY_SPEC_LIST_FILTERS: SpecListFilters = {
  statuses: [],
  checklist: "any",
  query: "",
};

export const EMPTY_SOURCE_LIST_FILTERS: SourceListFilters = {
  linked: "any",
  pathPrefixes: [],
  query: "",
};

export function specListFiltersActive(filters: SpecListFilters): boolean {
  return filters.statuses.length > 0 || filters.checklist !== "any";
}

export function sourceListFiltersActive(filters: SourceListFilters): boolean {
  return filters.linked !== "any" || filters.pathPrefixes.length > 0;
}

export function sourcePathPrefix(path: string): string {
  const normalized = path.replace(/\\/g, "/").replace(/^\.\//, "");
  const parts = normalized.split("/").filter(Boolean);
  return parts[0] ?? (normalized || "/");
}

export function distinctSourcePathPrefixes(sources: readonly WorkspaceSource[]): string[] {
  const values = new Set<string>();
  for (const source of sources) {
    values.add(sourcePathPrefix(source.path));
  }
  return [...values].sort((a, b) => a.localeCompare(b));
}

export function filterSpecFeatures(
  features: readonly WorkspaceSpecFeature[],
  filters: SpecListFilters,
): WorkspaceSpecFeature[] {
  const needle = filters.query.trim().toLowerCase();
  return features.filter((feature) => {
    if (filters.statuses.length > 0 && !filters.statuses.includes(feature.status)) {
      return false;
    }
    if (filters.checklist === "open" && !checklistHasOpenItems(feature.checklist.bodyMarkdown)) {
      return false;
    }
    if (filters.checklist === "clear" && checklistHasOpenItems(feature.checklist.bodyMarkdown)) {
      return false;
    }
    if (!needle) return true;
    return [feature.title, feature.featureKey, feature.summary, feature.owner].some((value) =>
      value.toLowerCase().includes(needle),
    );
  });
}

export function filterSources(
  sources: readonly WorkspaceSource[],
  filters: SourceListFilters,
): WorkspaceSource[] {
  const needle = filters.query.trim().toLowerCase();
  return sources.filter((source) => {
    if (filters.linked === "linked" && source.entityLinks.length === 0) return false;
    if (filters.linked === "unlinked" && source.entityLinks.length > 0) return false;
    if (
      filters.pathPrefixes.length > 0 &&
      !filters.pathPrefixes.includes(sourcePathPrefix(source.path))
    ) {
      return false;
    }
    if (!needle) return true;
    return [source.title, source.path, source.summary, source.owner].some((value) =>
      value.toLowerCase().includes(needle),
    );
  });
}
