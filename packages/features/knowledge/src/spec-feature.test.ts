import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";
import { featureKeyFromTitle, uniqueFeatureKey } from "./feature-key";
import { specStatusTone } from "./knowledge-document-helpers";

const root = dirname(fileURLToPath(import.meta.url));
const detailSource = readFileSync(resolve(root, "spec-feature-detail.tsx"), "utf8");
const indexSource = readFileSync(resolve(root, "index.tsx"), "utf8");
const emptySource = readFileSync(resolve(root, "knowledge-empty.tsx"), "utf8");

describe("knowledge specs helpers", () => {
  it("maps draft and ready tones", () => {
    expect(specStatusTone("ready")).toBe("healthy");
    expect(specStatusTone("draft")).toBe("warn");
  });

  it("slugifies feature titles", () => {
    expect(featureKeyFromTitle("Local Health Checks")).toBe("local-health-checks");
    expect(featureKeyFromTitle("  ")).toBe("feature");
  });

  it("keeps feature keys unique", () => {
    expect(uniqueFeatureKey(["auth"], "Auth")).toBe("auth-2");
    expect(uniqueFeatureKey(["auth", "auth-2"], "Auth")).toBe("auth-3");
  });
});

describe("SpecFeatureDetail composition contract", () => {
  it("uses document layout shell with Spec Plan Tasks tabs", () => {
    expect(detailSource).toContain("KnowledgeDocumentShell");
    expect(detailSource).toContain('label="Archive"');
    expect(detailSource).toContain("FileText");
    expect(detailSource).toContain('{ id: "spec", label: "Spec" }');
    expect(detailSource).toContain('{ id: "plan", label: "Plan" }');
    expect(detailSource).toContain('{ id: "tasks", label: "Tasks" }');
  });

  it("is wired from Knowledge index with Specs empty state", () => {
    expect(indexSource).toContain('from "./spec-feature-detail"');
    expect(indexSource).toContain("<SpecFeatureDetail");
    expect(indexSource).toContain('{ id: "specs", label: "Specs" }');
    expect(indexSource).toContain("onCreateFeature");
    expect(indexSource).toContain("KnowledgeSpecsEmpty");
    expect(emptySource).toContain("No specs yet.");
  });
});
