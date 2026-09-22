import { describe, expect, it } from "vitest";
import {
  EMPTY_RESOURCE_LIST_FILTERS,
  filterResources,
  resourceFiltersFromRoute,
  resourceFiltersToRoute,
  resourceListFiltersActive,
} from "./resource-filters";
import type { Resource } from "./types";

const resource = {
  id: "r1",
  name: "api",
  type: "Service",
  provider: "local",
  environment: "dev",
  status: "healthy",
  region: "local",
  tags: ["core"],
  description: "",
  dataSource: "core",
} satisfies Resource;

describe("resource-filters", () => {
  it("filters by status type provider and environment", () => {
    expect(
      filterResources([resource], {
        ...EMPTY_RESOURCE_LIST_FILTERS,
        statuses: ["offline"],
      }),
    ).toHaveLength(0);
    expect(
      filterResources([resource], {
        ...EMPTY_RESOURCE_LIST_FILTERS,
        types: ["Service"],
        providers: ["local"],
        environments: ["dev"],
      }),
    ).toHaveLength(1);
  });

  it("round-trips route filter maps", () => {
    const filters = {
      ...EMPTY_RESOURCE_LIST_FILTERS,
      statuses: ["healthy"],
      types: ["Service"],
      query: "api",
    };
    expect(resourceListFiltersActive(filters)).toBe(true);
    expect(resourceFiltersFromRoute(resourceFiltersToRoute(filters), "api")).toEqual(filters);
  });
});
