import { describe, expect, it } from "vitest";
import { SETTINGS_ENVIRONMENT_MOCKS } from "./fixtures";
import {
  environmentDetailTarget,
  environmentSelectOptions,
  environmentsDetailScope,
  normalizeSettingsTab,
  SETTINGS_TABS,
} from "./settings-helpers";

describe("settings helpers", () => {
  it("exposes only General · Environments · Advanced tabs", () => {
    expect(SETTINGS_TABS.map((tab) => tab.id)).toEqual([
      "general",
      "environments",
      "advanced",
    ]);
    const labels = SETTINGS_TABS.map((tab) => tab.label).join(" ");
    expect(labels).not.toMatch(/Access|Integrations|Knowledge/i);
  });

  it("normalizes legacy Access/Integrations tab ids to general", () => {
    expect(normalizeSettingsTab("unknown")).toBe("general");
    expect(normalizeSettingsTab("environments")).toBe("environments");
    expect(normalizeSettingsTab(undefined)).toBe("general");
  });

  it("builds select options and detail targets from fixtures", () => {
    const options = environmentSelectOptions();
    expect(options.length).toBe(SETTINGS_ENVIRONMENT_MOCKS.length);
    expect(options[0]).toEqual({
      value: SETTINGS_ENVIRONMENT_MOCKS[0]!.id,
      label: SETTINGS_ENVIRONMENT_MOCKS[0]!.name,
    });
    expect(environmentsDetailScope()).toBe("settings:environments");
    expect(environmentDetailTarget("env-prod")).toEqual({
      scopeId: "settings:environments",
      entityId: "env-prod",
      entityKind: "environment",
    });
  });
});

describe("settings environment fixtures", () => {
  it("covers kind and protection postures with honest dataSource", () => {
    expect(SETTINGS_ENVIRONMENT_MOCKS.length).toBeGreaterThanOrEqual(3);
    expect(
      SETTINGS_ENVIRONMENT_MOCKS.every((environment) => environment.dataSource === "fixture"),
    ).toBe(true);
    const kinds = new Set(SETTINGS_ENVIRONMENT_MOCKS.map((e) => e.kind));
    const protections = new Set(SETTINGS_ENVIRONMENT_MOCKS.map((e) => e.protection));
    expect(kinds.has("production")).toBe(true);
    expect(kinds.has("staging")).toBe(true);
    expect(kinds.has("development")).toBe(true);
    expect(protections.has("strict")).toBe(true);
    expect(protections.has("standard")).toBe(true);
    expect(protections.has("relaxed")).toBe(true);
    const defaults = SETTINGS_ENVIRONMENT_MOCKS.filter((e) => e.isDefault);
    expect(defaults).toHaveLength(1);
    const production = SETTINGS_ENVIRONMENT_MOCKS.find((e) => e.kind === "production");
    expect(production?.protectionRules.length).toBeGreaterThanOrEqual(2);
  });
});
