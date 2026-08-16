import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";
import { StatusChip, StatusDot } from "./status";

const here = dirname(fileURLToPath(import.meta.url));

describe("StatusDot", () => {
  it("exports a function component", () => {
    expect(StatusDot).toBeDefined();
  });
});

describe("StatusChip", () => {
  it("exports a function component", () => {
    expect(StatusChip).toBeDefined();
  });

  it("is always filled: no leading dot markup, filled data-appearance", () => {
    const source = readFileSync(join(here, "status.tsx"), "utf8");
    expect(source).toContain('data-appearance="filled"');
    expect(source).not.toContain("pw-status-chip__dot");
  });

  it("css has no default subtle+dot chip chrome", () => {
    const css = readFileSync(join(here, "status.css"), "utf8");
    expect(css).toContain("--pw-status-background");
    expect(css).toContain("--pw-status-foreground");
    expect(css).not.toContain("pw-status-chip__dot");
    expect(css).not.toContain("var(--color-surface-subtle)");
  });
});
