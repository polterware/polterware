import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";

const root = dirname(fileURLToPath(import.meta.url));
const source = readFileSync(resolve(root, "environment-detail.tsx"), "utf8");
const indexSource = readFileSync(resolve(root, "index.tsx"), "utf8");

describe("EnvironmentDetail composition contract", () => {
  it("uses the record layout with header status chips and a properties rail", () => {
    expect(source).toContain("DetailPanel.Main");
    expect(source).toContain("DetailPanel.Rail");
    expect(source).toContain("DetailPanel.Property");
    expect(source).not.toContain("DetailPanel.Metric");
    expect(source).not.toContain("DetailPanel.Grid");
    expect(source).toContain("PageLoading");
    expect(source).toContain('layout="detail"');
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
    expect(indexSource).toContain("onSetDefault");
    expect(indexSource).toContain("onDelete");
    expect(source).toContain('label="Set as default"');
    expect(source).toContain("DetailPanel.PrimaryAction");
    expect(source).toMatch(/DetailPanel\.OverflowItem[\s\S]*?danger[\s\S]*?Delete environment/);
    expect(source).toContain("onSetDefault");
    expect(source).toContain("onDelete");
  });
});
