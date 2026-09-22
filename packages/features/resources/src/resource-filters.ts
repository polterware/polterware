import type { Resource } from "./types";

export type ResourceListFilters = {
  statuses: readonly string[];
  types: readonly string[];
  providers: readonly string[];
  environments: readonly string[];
  query: string;
};

export const EMPTY_RESOURCE_LIST_FILTERS: ResourceListFilters = {
  statuses: [],
  types: [],
  providers: [],
  environments: [],
  query: "",
};

export function resourceListFiltersActive(filters: ResourceListFilters): boolean {
  return (
    filters.statuses.length > 0 ||
    filters.types.length > 0 ||
    filters.providers.length > 0 ||
    filters.environments.length > 0
  );
}

export function toggleFilterValue<T extends string>(
  current: readonly T[],
  id: T,
  checked: boolean,
): T[] {
  return checked ? [...current, id] : current.filter((value) => value !== id);
}

export function distinctSorted(values: readonly string[]): string[] {
  return [...new Set(values)].sort((a, b) => a.localeCompare(b));
}

export function filterResources(
  catalog: readonly Resource[],
  filters: ResourceListFilters,
): Resource[] {
  const needle = filters.query.trim().toLowerCase();
  return catalog.filter((resource) => {
    if (filters.statuses.length > 0 && !filters.statuses.includes(resource.status)) {
      return false;
    }
    if (filters.types.length > 0 && !filters.types.includes(resource.type)) {
      return false;
    }
    if (filters.providers.length > 0 && !filters.providers.includes(resource.provider)) {
      return false;
    }
    if (
      filters.environments.length > 0 &&
      !filters.environments.includes(resource.environment)
    ) {
      return false;
    }
    if (!needle) return true;
    return [resource.name, resource.type, resource.provider, ...resource.tags].some((value) =>
      value.toLowerCase().includes(needle),
    );
  });
}

export function resourceFiltersToRoute(
  filters: ResourceListFilters,
): Record<string, readonly string[]> {
  return {
    status: filters.statuses,
    type: filters.types,
    provider: filters.providers,
    environment: filters.environments,
  };
}

export function resourceFiltersFromRoute(
  filters: Readonly<Record<string, readonly string[]>> | undefined,
  query = "",
): ResourceListFilters {
  return {
    statuses: [...(filters?.status ?? [])],
    types: [...(filters?.type ?? [])],
    providers: [...(filters?.provider ?? [])],
    environments: [...(filters?.environment ?? [])],
    query,
  };
}
