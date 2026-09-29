import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import { Popover } from "./popover";
import { repoPath } from "../test/repo-path";

describe("Popover", () => {
  it("exports the Ark-backed product component", () => {
    expect(Popover).toBeDefined();
  });

  it("applies placement direction selectors for origin and transform", () => {
    const css = readFileSync(repoPath("packages/frontend/design-system/src/popover.css"), "utf8");
    expect(css).toContain('[data-placement^="bottom"] .pw-popover-surface { transform: translateY(-4px)');
    expect(css).toContain('[data-placement^="top"] .pw-popover-surface { transform: translateY(4px)');
    expect(css).toContain('[data-placement^="right"] .pw-popover-surface { transform: translateX(-4px)');
    expect(css).toContain('[data-placement^="left"] .pw-popover-surface { transform: translateX(4px)');
  });

  it("splits no-paint presence clock from inner visual motion", () => {
    const css = readFileSync(repoPath("packages/frontend/design-system/src/popover.css"), "utf8");
    expect(css).toMatch(/@keyframes pw-popover-presence {[\s\S]*--pw-popover-presence-clock: 1;/);
    expect(css).toMatch(/\.pw-popover-presence\s*{[^}]*animation: pw-popover-presence/);
    expect(css).toMatch(/\.pw-popover-surface\s*{[^}]*transition: transform/);
  });

  it("uses exact enter/exit durations", () => {
    const css = readFileSync(repoPath("packages/frontend/design-system/src/popover.css"), "utf8");
    expect(css).toContain("var(--duration-enter, 200ms)");
    expect(css).toContain("var(--duration-control, 160ms)");
    expect(css).toContain("transition-duration: var(--duration-enter, 200ms)");
    expect(css).toMatch(/animation: pw-popover-presence var\(--duration-control, 160ms\) forwards;/);
  });

  it("provides reduced-motion opacity-only behavior", () => {
    const css = readFileSync(repoPath("packages/frontend/design-system/src/popover.css"), "utf8");
    expect(css).toContain("@media (prefers-reduced-motion: reduce)");
    expect(css).toMatch(/transform: none !important;/);
    expect(css).toMatch(/transition: opacity var\(--duration-fast, 120ms\)/);
  });
});
