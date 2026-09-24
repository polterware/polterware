import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";
import { documentStatusTone } from "./knowledge-document-helpers";

const root = dirname(fileURLToPath(import.meta.url));
const detailSource = readFileSync(resolve(root, "document-detail.tsx"), "utf8");
const shellSource = readFileSync(resolve(root, "knowledge-document-shell.tsx"), "utf8");
const indexSource = readFileSync(resolve(root, "index.tsx"), "utf8");

describe("Document detail helpers", () => {
  it("maps document status tones", () => {
    expect(documentStatusTone("published")).toBe("healthy");
    expect(documentStatusTone("draft")).toBe("warn");
  });
});

describe("DocumentDetail composition contract", () => {
  it("uses document layout shell and Archive action", () => {
    expect(detailSource).toContain("KnowledgeDocumentShell");
    expect(detailSource).toContain("KnowledgeDocumentNotFound");
    expect(detailSource).toMatch(/OverflowItem value="archive" danger/);
    expect(detailSource).not.toContain('tone="danger"');
    expect(detailSource).toContain("Book03Icon");
    expect(shellSource).toContain('layout="document"');
    expect(shellSource).toContain("MarkdownEditor");
    expect(shellSource).toContain('panelMode() === "maximized"');
    expect(shellSource).toContain("showToolbar");
    expect(shellSource).not.toContain("DocumentProperties");
  });

  it("is wired from Knowledge index orchestration", () => {
    expect(indexSource).toContain('from "./document-detail"');
    expect(indexSource).toContain("<DocumentDetail");
    expect(indexSource).toContain("onArchive");
    expect(indexSource).toContain("<CreateKnowledgeDialog");
    expect(indexSource).not.toContain("function DocumentCanvas");
    expect(indexSource).not.toContain("function DocumentProperties");
  });
});
