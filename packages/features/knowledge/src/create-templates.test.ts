import { describe, expect, it } from "vitest";
import {
  KNOWLEDGE_CREATE_TEMPLATES,
  knowledgeTemplateById,
  knowledgeTemplatesForKind,
} from "./create-templates";

describe("knowledge create templates", () => {
  it("exports unique ids", () => {
    const ids = KNOWLEDGE_CREATE_TEMPLATES.map((template) => template.id);
    expect(new Set(ids).size).toBe(ids.length);
  });

  it("groups templates by kind", () => {
    expect(knowledgeTemplatesForKind("doc")).toHaveLength(3);
    expect(knowledgeTemplatesForKind("adr")).toHaveLength(3);
    expect(knowledgeTemplatesForKind("runbook")).toHaveLength(3);
    expect(knowledgeTemplatesForKind("feature")).toHaveLength(3);
    expect(knowledgeTemplatesForKind("source")).toHaveLength(3);
  });

  it("resolves a template by id", () => {
    const template = knowledgeTemplateById("doc-architecture-overview");
    expect(template?.title).toBe("Architecture overview");
    expect(template?.body).toContain("## Purpose");
  });
});
