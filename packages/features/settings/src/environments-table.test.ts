import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";

const root = dirname(fileURLToPath(import.meta.url));
const source = readFileSync(resolve(root, "index.tsx"), "utf8");

describe("Environments table composition contract", () => {
  it("uses DataTable with PrimaryCell and StatusChip for kind and protection", () => {
    expect(source).toContain("DataTable.Root");
    expect(source).toContain("DataTable.PrimaryCell");
    expect(source).toContain("StatusChip");
    expect(source).toContain("environmentKindTone");
    expect(source).toContain("environmentKindLabel");
    expect(source).toContain("protectionTone");
    expect(source).toContain("protectionLabel");
    expect(source).toContain('aria-label="Environments"');
    expect(source).toContain("environments()");
    expect(source).not.toContain("SETTINGS_ENVIRONMENT_MOCKS");
    expect(source).toContain("activateFirstRow");
    expect(source).toContain("onActiveRowChange");
  });

  it("wires environment detail activation and does not reintroduce Access/Integrations tabs", () => {
    expect(source).toContain("environmentsDetailScope");
    expect(source).toContain("environmentDetailTarget");
    expect(source).toContain("<EnvironmentDetail");
    expect(source).toContain("registerCollection");
    expect(source).toContain("activateTarget");
    expect(source).toContain("SETTINGS_TABS");
    expect(source).not.toMatch(/id:\s*"access"/);
    expect(source).not.toMatch(/id:\s*"integrations"/);
    expect(source).not.toMatch(/id:\s*"connected"/);
    expect(source).not.toMatch(/id:\s*"people"/);
    expect(source).not.toContain('label: "Access"');
    expect(source).not.toContain('label: "Integrations"');
  });

  it("uses Field primitives on General and Dialog on Advanced archive", () => {
    expect(source).toContain("<Field");
    expect(source).toContain("Save changes");
    expect(source).toContain("saveGeneralSettingsToast");
    expect(source).toContain("A11yDialog");
    expect(source).toContain("Archive project");
    expect(source).toContain("archiveProjectToast");
    expect(source).not.toContain("inputClass");
  });
});
