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
    const persist = readFileSync(join(import.meta.dirname, "workspace/persist.ts"), "utf8");
    expect(source).toContain("loadWorkspace");
    expect(source).toContain("persistView");
    expect(source).toContain("persistKnowledgeCreate");
    expect(source).toContain("persistResource");
    expect(source).toContain("persistNode");
    expect(source).toContain("persistStartProcess");
    expect(source).toContain("persistStopProcess");
    expect(source).toContain("persistEnvironment");
    expect(source).toContain("persistEnvironmentDefault");
    expect(source).toContain("persistWorkflow");
    expect(source).toContain("persistSchedule");
    expect(source).toContain("persistIntegrationSync");
    expect(source).not.toContain("Integration metadata refreshed locally.");
    expect(persist).toContain("syncConnection");
    expect(persist).not.toContain("Metadata refreshed from the local store.");
    expect(source).toContain("projectId={currentProjectId()}");
    expect(source).toContain("draft.name");
    expect(source).toContain("draft.type");
    expect(source).not.toContain('name: "New resource"');
    expect(source).toContain('fixtureMode="off"');
    expect(source).not.toContain("PROJECT_SWITCHER_FIXTURES");
    expect(source).not.toContain("RESOURCE_FIXTURES");
    expect(source).not.toContain('fixtureMode="force"');
  });
});
