import { describe, expect, it } from "vitest";

describe("Desktop core composition", () => {
  it("composes only retained surfaces", async () => {
    const { readFileSync } = await import("node:fs");
    const { join } = await import("node:path");
    const source = readFileSync(join(import.meta.dirname, "App.tsx"), "utf8");
    for (const surface of ["ArchitectureView", "ResourcesView", "KnowledgeView", "OperationsView", "TerminalPage", "IntegrationsView", "SettingsView"]) {
      expect(source).toContain(surface);
    }
  });

  it("loads workspace data through the typed core bridge", async () => {
    const { readFileSync } = await import("node:fs");
    const { join } = await import("node:path");
    const source = readFileSync(join(import.meta.dirname, "App.tsx"), "utf8");
    expect(source).toContain("loadWorkspace");
    expect(source).toContain('fixtureMode="off"');
    expect(source).not.toContain("PROJECT_SWITCHER_FIXTURES");
    expect(source).not.toContain("RESOURCE_FIXTURES");
    expect(source).not.toContain('fixtureMode="force"');
  });
});
