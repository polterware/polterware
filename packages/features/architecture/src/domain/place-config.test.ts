import { describe, expect, it } from "vitest";
import {
  environmentIdsFromConfig,
  lifecycleFromPlaceConfig,
  metadataFromPlaceConfig,
  normalizePlaceConfig,
} from "./place-config";
import { DEFAULT_PROJECT_ENVIRONMENTS } from "./project-environments";

describe("place-config", () => {
  it("requires name and valid environment", () => {
    expect(normalizePlaceConfig({ name: "  ", environmentId: "production" })).toEqual({
      error: "Name is required",
    });
    expect(normalizePlaceConfig({ name: "API", environmentId: "nope" })).toEqual({
      error: "Environment is required",
    });
  });

  it("rejects empty project environment list", () => {
    expect(normalizePlaceConfig({ name: "API", environmentId: "production" }, [])).toEqual({
      error: "Add a project environment first",
    });
  });

  it("normalizes valid draft with only essentials", () => {
    const r = normalizePlaceConfig({ name: "  payments-db  ", environmentId: "staging" });
    expect(r).toEqual({ name: "payments-db", environmentId: "staging" });
    if ("error" in r) throw new Error("expected ok");
    expect(environmentIdsFromConfig(r)).toEqual(["staging"]);
    expect(metadataFromPlaceConfig(r)).toBeUndefined();
    expect(lifecycleFromPlaceConfig(r)).toEqual({
      state: "planned",
      linkStatus: "unlinked",
      resourceId: null,
    });
  });

  it("accepts Production label via normalize and project envs", () => {
    const r = normalizePlaceConfig(
      { name: "api", environment: "Production" },
      DEFAULT_PROJECT_ENVIRONMENTS,
    );
    expect(r).toMatchObject({ name: "api", environmentId: "production" });
  });

  it("keeps optional purpose, role, owner, and resourceId when set", () => {
    const r = normalizePlaceConfig({
      name: "payments-db",
      environmentId: "production",
      description: "  Primary OLTP for checkout  ",
      role: "primary",
      owner: "  Platform team  ",
      resourceId: "  res-pg  ",
    });
    expect(r).toEqual({
      name: "payments-db",
      environmentId: "production",
      description: "Primary OLTP for checkout",
      role: "primary",
      owner: "Platform team",
      resourceId: "res-pg",
    });
    if ("error" in r) throw new Error("expected ok");
    expect(metadataFromPlaceConfig(r)).toEqual({ role: "primary" });
    expect(lifecycleFromPlaceConfig(r)).toEqual({
      state: "linked",
      linkStatus: "linked",
      resourceId: "res-pg",
    });
  });

  it("omits empty optional fields", () => {
    const r = normalizePlaceConfig({
      name: "api",
      environmentId: "local",
      description: "   ",
      role: "",
      owner: "",
      resourceId: "",
    });
    expect(r).toEqual({ name: "api", environmentId: "local" });
  });

  it("rejects unknown role", () => {
    expect(
      normalizePlaceConfig({
        name: "api",
        environmentId: "production",
        role: "not-a-role",
      }),
    ).toEqual({ error: "Role is invalid" });
  });
});
