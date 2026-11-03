import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import { Popover } from "./popover";
import { repoPath } from "../test/repo-path";

describe("Popover", () => {
  it("exports the Ark-backed product component", () => {
    expect(Popover).toBeDefined();
  });

  it("springs the surface in from its anchor side", () => {
    const source = readFileSync(repoPath("packages/frontend/design-system/src/popover.tsx"), "utf8");
    expect(source).toContain("createPopupMotion()");
    expect(source).toContain("<div ref={motion.ref}");
  });

  it("keeps a no-paint presence clock so Ark waits for the exit tween", () => {
    const css = readFileSync(repoPath("packages/frontend/design-system/src/popover.css"), "utf8");
    expect(css).toMatch(/@keyframes pw-popover-presence {[\s\S]*--pw-popover-presence-clock: 1;/);
    expect(css).toMatch(/animation: pw-popover-presence var\(--duration-fast, 120ms\) forwards;/);
  });

  it("keeps the clock running under reduced motion so the fade still plays", () => {
    const css = readFileSync(repoPath("packages/frontend/design-system/src/popover.css"), "utf8");
    expect(css).toContain("@media (prefers-reduced-motion: reduce)");
    expect(css).toContain("animation-duration: var(--duration-fast, 120ms) !important;");
  });
});
