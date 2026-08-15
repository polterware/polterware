import { readFileSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";

describe("Desktop fixture composition", () => {
  const source = readFileSync(join(import.meta.dirname, "App.tsx"), "utf8");

  it("composes only retained surfaces", () => {
    for (const surface of ["ArchitectureView", "ResourcesView", "KnowledgeView", "OperationsView", "TerminalPage", "IntegrationsView", "SettingsView"]) {
      expect(source).toContain(surface);
    }
  });

  it("does not initialize or query a runtime", () => {
    expect(source).not.toContain("window.polterware.core");
    expect(source).not.toContain("createPolterwareClient");
    expect(source).toContain('fixtureMode="force"');
  });
});
