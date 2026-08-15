// @vitest-environment jsdom

import { describe, expect, it } from "vitest";
import { render } from "solid-js/web";
import { SERVICE_MARK_TINT } from "../domain/service-marks";
import { ArchitectureNodeIcon, nodeIconAccent } from "./architecture-node-icon";

function renderHtml(ui: () => unknown) {
  const host = document.createElement("div");
  document.body.appendChild(host);
  const dispose = render(ui as Parameters<typeof render>[0], host);
  const html = host.innerHTML;
  dispose();
  host.remove();
  return html;
}

describe("ArchitectureNodeIcon", () => {
  it("uses the mark first, then provider, then the light neutral dither tone", () => {
    expect(nodeIconAccent("vercel", "AWS")).toBe(SERVICE_MARK_TINT.vercel);
    expect(nodeIconAccent("service", "AWS")).toBe(SERVICE_MARK_TINT.aws);
    expect(nodeIconAccent("service")).toBe("var(--color-dither-accent-neutral)");
  });

  it("renders an opaque dither tile with a dark mark over light brand dither", () => {
    const html = renderHtml(() => (
      <ArchitectureNodeIcon
        name="vercel"
        provider="Vercel"
        compact={false}
        dimmed={false}
      />
    ));

    expect(html).toContain("data-dither-icon");
    expect(html).toContain('data-size="28"');
    expect(html).toContain("pw-dither-icon");
    // jsdom serializes inline hex as rgb()
    expect(html).toMatch(/color:\s*rgb\(237,\s*237,\s*237\)|#EDEDED/i);
    expect(html).toContain('fill="currentColor"');
    expect(html).not.toContain("--color-text-primary");
  });
});
