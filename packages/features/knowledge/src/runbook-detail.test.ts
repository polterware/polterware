import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";
import { runbookSeverityTone } from "./knowledge-document-helpers";

const root = dirname(fileURLToPath(import.meta.url));
const detailSource = readFileSync(resolve(root, "runbook-detail.tsx"), "utf8");
const shellSource = readFileSync(resolve(root, "knowledge-document-shell.tsx"), "utf8");
const indexSource = readFileSync(resolve(root, "index.tsx"), "utf8");

describe("Runbook detail helpers", () => {
  it("maps severity tones without inventing live health", () => {
    expect(runbookSeverityTone("critical")).toBe("danger");
    expect(runbookSeverityTone("high")).toBe("danger");
    expect(runbookSeverityTone("medium")).toBe("warn");
    expect(runbookSeverityTone("low")).toBe("muted");
  });
});

describe("RunbookDetail composition contract", () => {
  it("uses document layout without condemned execution actions", () => {
    expect(detailSource).toContain("KnowledgeDocumentShell");
    expect(detailSource).toContain("metadataActions=");
    expect(detailSource).toMatch(/Button variant="ghost"[\s\S]*?Duplicate/);
    expect(detailSource).toContain("footerActions=");
    expect(detailSource).toMatch(/Button variant="danger"[\s\S]*?Archive/);
    expect(detailSource).not.toContain('label="Run"');
    expect(detailSource).not.toContain('label="Open terminal"');
    expect(detailSource).toContain("KnowledgeDocumentNotFound");
    expect(detailSource).not.toContain("live execution");
    expect(shellSource).toContain('layout="document"');
    expect(shellSource).toContain("state.expanded");
    expect(shellSource).not.toContain("DocumentProperties");
  });

  it("is wired from Knowledge index orchestration", () => {
    expect(indexSource).toContain('from "./runbook-detail"');
    expect(indexSource).toContain("<RunbookDetail");
    expect(indexSource).not.toContain("onOpenTerminal");
    expect(indexSource).not.toContain("onOpenOperation");
    expect(indexSource).not.toContain("onRun={");
    expect(indexSource).not.toContain("onRun?:");
    expect(indexSource).toContain("onDuplicate");
    expect(indexSource).toContain("onArchive");
  });
});
