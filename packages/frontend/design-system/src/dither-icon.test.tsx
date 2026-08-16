// @vitest-environment jsdom

import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";
import { render } from "solid-js/web";
import {
  DITHER_BRAND_ACCENTS,
  DitherIconTile,
  resolveDitherIconAccent,
} from "./dither-icon";

const styles = readFileSync(
  resolve(dirname(fileURLToPath(import.meta.url)), "dither-icon.css"),
  "utf8",
);

function renderHtml(ui: Parameters<typeof render>[0]) {
  const host = document.createElement("div");
  document.body.appendChild(host);
  const dispose = render(ui, host);
  const html = host.innerHTML;
  dispose();
  host.remove();
  return html;
}

describe("DitherIconTile", () => {
  it("prefers the first recognized brand before the domain tone", () => {
    expect(resolveDitherIconAccent(["prod-postgres", "AWS"], "infrastructure"))
      .toBe(DITHER_BRAND_ACCENTS.postgres);
    expect(resolveDitherIconAccent(["github-actions", "GitHub"], "integration"))
      .toBe(DITHER_BRAND_ACCENTS.github_actions);
    expect(resolveDitherIconAccent("unknown-service", "knowledge"))
      .toBe("var(--color-chart-amber)");
  });

  it("renders the supported sizes with a neutral centered mark", () => {
    for (const size of [20, 24, 28] as const) {
      const html = renderHtml(() => (
        <DitherIconTile size={size} tone="architecture">
          <svg data-mark viewBox="0 0 16 16" />
        </DitherIconTile>
      ));

      expect(html).toContain("data-dither-icon");
      expect(html).toContain(`data-size="${size}"`);
      expect(html).toContain(`width: ${size}px`);
      expect(html).toContain("pw-dither-icon-mark");
      expect(html).toContain("--color-chart-blue");
    }
  });

  it("uses light neutral dither + dark dimmed mark tokens when dimmed", () => {
    const html = renderHtml(() => (
      <DitherIconTile tone="infrastructure" brand="AWS" dimmed>
        <svg data-mark viewBox="0 0 16 16" />
      </DitherIconTile>
    ));

    expect(html).toContain("pw-dither-icon--dimmed");
    expect(html).toContain("--color-dither-accent-neutral");
    expect(html).not.toContain("--color-text-disabled");
    expect(html).not.toContain("opacity");
    expect(html).not.toContain("gradient");
  });

  it("keeps a two-tone light tile (no black surface gaps)", () => {
    expect(styles).toContain("var(--color-dither-icon-base)");
    expect(styles).toContain("var(--color-dither-icon-edge)");
    expect(styles).toMatch(/color-mix\(\s*in srgb/);
    expect(styles).toContain("currentColor");
    expect(styles).not.toContain("--color-surface-subtle");
    expect(styles).toContain("border-radius: var(--radius-item)");
    expect(styles).not.toMatch(/\bopacity\b/);
    expect(styles).not.toMatch(/\bgradient\b/);
  });

  it("paints dark marks on light dither accents, never text-primary", () => {
    expect(styles).toContain("color: var(--color-dither-icon-mark)");
    expect(styles).toContain("color: var(--color-dither-icon-mark-dimmed)");
    expect(styles).not.toContain("--color-text-primary");
    expect(styles).not.toContain("--color-text-disabled");
  });

  it("uses a bolder stroke on identity-tile marks than the global 1.5 default", () => {
    expect(styles).toContain("stroke-width: var(--pw-dither-icon-stroke-width, 2)");
  });

  it("keeps brand dither accents light pastels for dark-mark contrast", () => {
    expect(DITHER_BRAND_ACCENTS.postgres).toBe("#7EA8C9");
    expect(DITHER_BRAND_ACCENTS.elasticsearch).toBe("#6EB8C9");
    expect(DITHER_BRAND_ACCENTS.nginx).toBe("#6EC99A");
    expect(DITHER_BRAND_ACCENTS.datadog).toBe("#B08FD9");
    expect(DITHER_BRAND_ACCENTS.github).toBe("#EDEDED");
    expect(resolveDitherIconAccent(undefined, "neutral")).toBe(
      "var(--color-dither-accent-neutral)",
    );
  });
});
