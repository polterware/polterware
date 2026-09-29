import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";

const root = dirname(fileURLToPath(import.meta.url));
const source = readFileSync(resolve(root, "index.tsx"), "utf8");
const rows = readFileSync(resolve(root, "environment-rows.tsx"), "utf8");

describe("Settings dialog composition contract", () => {
  it("renders inside a flush A11yDialog with a tab nav and section cards", () => {
    expect(source).toContain("<A11yDialog");
    expect(source).toContain('chrome="flush"');
    expect(source).toContain("SETTINGS_TABS");
    expect(source).toContain("<SettingsSection");
    expect(source).toContain("<SettingsRow");
    expect(source).toContain("onClose");
  });

  it("lists environments as expandable rows with StatusChip kind and protection", () => {
    expect(source).toContain("<EnvironmentRows");
    expect(source).toContain("Add environment");
    expect(source).toContain("<CreateEnvironmentDialog");
    expect(source).toContain("onCreateEnvironment");
    expect(source).toContain("onSetDefaultEnvironment");
    expect(rows).toContain("aria-expanded");
    expect(rows).toContain("StatusChip");
    expect(rows).toContain("environmentKindTone");
    expect(rows).toContain("protectionTone");
    expect(rows).toContain("Set as default");
    expect(rows).toContain("Delete environment");
    expect(source).not.toContain("DetailPanel");
    expect(source).not.toContain("DataTable");
    expect(source).not.toMatch(/id:\s*"access"/);
    expect(source).not.toMatch(/id:\s*"integrations"/);
  });

  it("uses the shared inputs on General and a dialog for the archive confirm", () => {
    expect(source).toContain("<Input");
    expect(source).toContain("Save changes");
    expect(source).toContain("saveGeneralSettingsToast");
    expect(source).toContain("Archive project");
    expect(source).toContain("archiveProjectToast");
    expect(source).not.toContain("inputClass");
  });
});
