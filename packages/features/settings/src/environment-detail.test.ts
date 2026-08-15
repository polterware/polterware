import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";

const root = dirname(fileURLToPath(import.meta.url));
const source = readFileSync(resolve(root, "environment-detail.tsx"), "utf8");
const indexSource = readFileSync(resolve(root, "index.tsx"), "utf8");

describe("EnvironmentDetail composition contract", () => {
  it("uses production DetailPanel sections layout and StatusChip overview", () => {
    expect(source).toContain('layout="sections"');
    expect(source).toContain("DetailPanel.Grid");
    expect(source).toContain("DetailPanel.Metric");
    expect(source).toContain("DetailPanel.Metadata");
    expect(source).toContain("PageLoading");
    expect(source).toContain("PageError");
    expect(source).toContain("Environment not found");
    expect(source).toContain("StatusChip");
    expect(source).toContain("Protection rules");
    expect(source).toContain('aria-label="Protection rules"');
    expect(source).not.toContain("description=");
  });

  it("is wired from Settings Environments activation", () => {
    expect(indexSource).toContain('from "./environment-detail"');
    expect(indexSource).toContain("<EnvironmentDetail");
    expect(indexSource).toContain("registerCollection");
    expect(indexSource).toContain("activateTarget");
  });
});
