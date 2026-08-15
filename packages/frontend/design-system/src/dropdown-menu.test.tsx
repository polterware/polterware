import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "./dropdown-menu";

describe("DropdownMenu", () => {
  it("exports compound components", () => {
    expect(typeof DropdownMenu).toBe("function");
    expect(typeof DropdownMenuTrigger).toBe("function");
    expect(typeof DropdownMenuContent).toBe("function");
    expect(typeof DropdownMenuItem).toBe("function");
    expect(typeof DropdownMenuSeparator).toBe("function");
  });

  it("forwards custom positioning to Ark Menu.Root", () => {
    const source = readFileSync(
      "packages/frontend/design-system/src/dropdown-menu.tsx",
      "utf8",
    );
    expect(source).toContain('positioning?: MenuRootProps["positioning"]');
    expect(source).toContain("positioning={props.positioning}");
  });

  it("applies placement direction selectors for origin and transform", () => {
    const css = readFileSync("packages/frontend/design-system/src/dropdown-menu.css", "utf8");
    expect(css).toContain('[data-placement^="bottom"] .pw-dropdown-surface { transform: translateY(-4px)');
    expect(css).toContain('[data-placement^="top"] .pw-dropdown-surface { transform: translateY(4px)');
    expect(css).toContain('[data-placement^="right"] .pw-dropdown-surface { transform: translateX(-4px)');
    expect(css).toContain('[data-placement^="left"] .pw-dropdown-surface { transform: translateX(4px)');
  });

  it("splits no-paint presence clock from inner visual motion", () => {
    const css = readFileSync("packages/frontend/design-system/src/dropdown-menu.css", "utf8");
    expect(css).toMatch(/@keyframes pw-dropdown-presence {[\s\S]*--pw-dropdown-presence-clock: 1;/);
    expect(css).toMatch(/\.pw-dropdown-presence\s*{[^}]*animation: pw-dropdown-presence/);
    expect(css).toMatch(/\.pw-dropdown-surface\s*{[^}]*transition: transform/);
  });

  it("uses exact enter/exit durations", () => {
    const css = readFileSync("packages/frontend/design-system/src/dropdown-menu.css", "utf8");
    expect(css).toContain("var(--duration-control, 160ms)");
    expect(css).toContain("var(--duration-fast, 120ms)");
    expect(css).toContain("transition-duration: var(--duration-control, 160ms)");
    expect(css).toMatch(/animation: pw-dropdown-presence var\(--duration-fast, 120ms\) forwards;/);
  });

  it("provides reduced-motion opacity-only behavior", () => {
    const css = readFileSync("packages/frontend/design-system/src/dropdown-menu.css", "utf8");
    expect(css).toContain("@media (prefers-reduced-motion: reduce)");
    expect(css).toMatch(/transform: none !important;/);
    expect(css).toMatch(/transition: opacity var\(--duration-fast, 120ms\)/);
  });
});
