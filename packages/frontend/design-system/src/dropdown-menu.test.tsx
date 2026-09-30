import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "./dropdown-menu";
import { repoPath } from "../test/repo-path";

describe("DropdownMenu", () => {
  it("exports compound components", () => {
    expect(DropdownMenu).toBeDefined();
    expect(DropdownMenuTrigger).toBeDefined();
    expect(DropdownMenuContent).toBeDefined();
    expect(DropdownMenuItem).toBeDefined();
    expect(DropdownMenuSeparator).toBeDefined();
  });

  it("forwards custom positioning to Ark Menu.Root", () => {
    const source = readFileSync(
      repoPath("packages/frontend/design-system/src/dropdown-menu.tsx"),
      "utf8",
    );
    expect(source).toContain('positioning?: MenuRootProps["positioning"]');
    expect(source).toContain("positioning={props.positioning}");
  });

  it("springs the surface from its resolved anchor side", () => {
    const motion = readFileSync(repoPath("packages/frontend/design-system/src/popup-motion.ts"), "utf8");
    const source = readFileSync(repoPath("packages/frontend/design-system/src/dropdown-menu.tsx"), "utf8");
    expect(source).toContain("createPopupMotion({ scale: true })");
    expect(motion).toContain('side === "top" ? slide : side === "bottom" ? -slide : 0');
    expect(motion).toContain('side === "left" ? slide : side === "right" ? -slide : 0');
    expect(motion).toContain("dataset.placement");
    expect(motion).toContain("spring.fast.exit");
  });

  it("keeps a no-paint presence clock so Ark waits for the exit tween", () => {
    const css = readFileSync(repoPath("packages/frontend/design-system/src/dropdown-menu.css"), "utf8");
    expect(css).toMatch(/@keyframes pw-dropdown-presence {[\s\S]*--pw-dropdown-presence-clock: 1;/);
    expect(css).toMatch(/animation: pw-dropdown-presence var\(--duration-fast, 120ms\) forwards;/);
    expect(css).toContain("transform-origin: var(--transform-origin, top left)");
  });

  it("drops travel but keeps the fade under reduced motion", () => {
    const motion = readFileSync(repoPath("packages/frontend/design-system/src/popup-motion.ts"), "utf8");
    expect(motion).toContain("prefersReducedMotion()");
    expect(motion).toContain("const slide = reduced ? 0 : POPUP_SLIDE;");
  });

  it("moves one shared row highlight from the pointer and Ark's keyboard highlight", () => {
    const source = readFileSync(repoPath("packages/frontend/design-system/src/dropdown-menu.tsx"), "utf8");
    expect(source).toContain("menu().highlightedValue");
    expect(source).toContain("api.setHighlightedValue(value)");
    expect(source).toContain("<MenuRowsLayer rows={rows} />");
  });
});
