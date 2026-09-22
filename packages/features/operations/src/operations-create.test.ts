import { describe, expect, it } from "vitest";

describe("Operations create dialog contracts", () => {
  it("wires workflow create draft fields", async () => {
    const { readFileSync } = await import("node:fs");
    const { dirname, resolve } = await import("node:path");
    const { fileURLToPath } = await import("node:url");
    const root = dirname(fileURLToPath(import.meta.url));
    const source = readFileSync(resolve(root, "create-workflow-dialog.tsx"), "utf8");

    expect(source).toContain("CreateWorkflowDraft");
    expect(source).toContain('title="Add workflow"');
    expect(source).toContain('label="Name"');
    expect(source).toContain('label="Description"');
    expect(source).toContain("local SQLite database");
  });

  it("wires schedule create against existing workflows and header environment note", async () => {
    const { readFileSync } = await import("node:fs");
    const { dirname, resolve } = await import("node:path");
    const { fileURLToPath } = await import("node:url");
    const root = dirname(fileURLToPath(import.meta.url));
    const source = readFileSync(resolve(root, "create-schedule-dialog.tsx"), "utf8");

    expect(source).toContain("CreateScheduleDraft");
    expect(source).toContain('title="Add schedule"');
    expect(source).toContain('label="Workflow"');
    expect(source).toContain('label="Expression"');
    expect(source).toContain("Create a workflow first");
    expect(source).toContain("currently selected environment from the header switcher");
  });

  it("shows tab-scoped create actions from OperationsView", async () => {
    const { readFileSync } = await import("node:fs");
    const { dirname, resolve } = await import("node:path");
    const { fileURLToPath } = await import("node:url");
    const root = dirname(fileURLToPath(import.meta.url));
    const source = readFileSync(resolve(root, "index.tsx"), "utf8");

    expect(source).toContain('label="Add workflow"');
    expect(source).toContain('label="Add schedule"');
    expect(source).toContain("ToolbarIconButton");
    expect(source).toContain("<CreateWorkflowDialog");
    expect(source).toContain("<CreateScheduleDialog");
    expect(source).toContain("onCreateWorkflow");
    expect(source).toContain("onCreateSchedule");
  });
});
