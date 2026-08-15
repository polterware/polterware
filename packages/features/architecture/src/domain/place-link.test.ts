import { describe, expect, it } from "vitest";
import {
  filterLinkCandidates,
  suggestPlaceFromResource,
  typesCompatible,
  validateLink,
  type LinkableResource,
} from "./place-link";

const samples: LinkableResource[] = [
  {
    id: "res-pg",
    name: "prod-postgres",
    type: "Database",
    environmentId: "production",
    dataSource: "fixture",
  },
  {
    id: "res-stg",
    name: "staging-api",
    type: "Service",
    environmentId: "staging",
    dataSource: "fixture",
  },
  {
    id: "res-redis",
    name: "redis-cache",
    type: "Cache",
    environmentId: "production",
    dataSource: "fixture",
  },
];

describe("place-link", () => {
  it("filters by environment and type family", () => {
    const list = filterLinkCandidates(samples, {
      environmentId: "production",
      componentType: "database",
    });
    expect(list.map((r) => r.id)).toEqual(["res-pg"]);
  });

  it("rejects cross-env link", () => {
    expect(validateLink("staging", samples[0])).toEqual({
      error: "Resource environment does not match",
    });
    expect(validateLink("production", samples[0])).toEqual({ resourceId: "res-pg" });
  });

  it("returns empty when no candidates", () => {
    expect(
      filterLinkCandidates(samples, { environmentId: "local", componentType: "database" }),
    ).toEqual([]);
  });

  it("typesCompatible maps service family", () => {
    expect(typesCompatible("service", "API")).toBe(true);
    expect(typesCompatible("database", "Queue")).toBe(false);
  });

  it("suggestPlaceFromResource prefills identity", () => {
    expect(suggestPlaceFromResource(samples[0]!)).toMatchObject({
      name: "prod-postgres",
      environmentId: "production",
      resourceId: "res-pg",
      suggestedComponentType: "database",
    });
  });
});
