import { describe, expect, it } from "vitest";
import { lifecycleFromPlaceConfig, normalizePlaceConfig } from "./place-config";

describe("place-commit lifecycle", () => {
  it("builds planned/unlinked without resourceId", () => {
    const cfg = normalizePlaceConfig({ name: "payments-db", environmentId: "staging" });
    if ("error" in cfg) throw new Error(cfg.error);
    expect(lifecycleFromPlaceConfig(cfg)).toEqual({
      state: "planned",
      linkStatus: "unlinked",
      resourceId: null,
    });
  });

  it("builds linked when resourceId present", () => {
    const cfg = normalizePlaceConfig({
      name: "payments-db",
      environmentId: "production",
      resourceId: "res-pg",
    });
    if ("error" in cfg) throw new Error(cfg.error);
    expect(lifecycleFromPlaceConfig(cfg)).toEqual({
      state: "linked",
      linkStatus: "linked",
      resourceId: "res-pg",
    });
  });
});
