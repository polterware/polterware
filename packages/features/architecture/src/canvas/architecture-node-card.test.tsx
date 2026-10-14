import { describe, expect, it } from "vitest";
import type { JSX } from "solid-js";
import { render } from "solid-js/web";
import { readFileSync } from "node:fs";
import { ARCH_CANVAS_FIXTURE } from "../fixtures";
import { ArchitectureNodeCard } from "./architecture-node-card";
import { nodeStatusLabel } from "../domain/node-lifecycle";

/** Mounts the view in jsdom and returns its markup (renderToString is server-only). */
function renderToString(view: () => JSX.Element): string {
  const host = document.createElement("div");
  const dispose = render(view, host);
  const html = host.innerHTML;
  dispose();
  return html;
}

const here = (name: string) => new URL(name, import.meta.url);

describe("ArchitectureNodeCard", () => {
  it("exports the card component", () => {
    expect(ArchitectureNodeCard).toBeDefined();
  });

  it("maps node states to operational labels", () => {
    expect(nodeStatusLabel("observed")).toBe("Observed");
    expect(nodeStatusLabel("linked")).toBe("Linked");
    expect(nodeStatusLabel("drifted")).toBe("Drifted");
    expect(nodeStatusLabel("unreachable")).toBe("Unreachable");
    expect(nodeStatusLabel("planned")).toBe("Planned");
    expect(nodeStatusLabel("unknown")).toBe("Unknown");
  });

  it("renders accessible card chrome", () => {
    const html = renderToString(() => (
      <ArchitectureNodeCard
        name="API"
        type="api"
        state="observed"
        x={0}
        y={0}
        width={188}
        height={56}
        selected={false}
        dimmed={false}
        lodFar={false}
        editing={false}
        onRenameEnd={() => undefined}
        onRenameCancel={() => undefined}
        onActionOpenDetails={() => undefined}
        onActionRename={() => undefined}
        onActionConnect={() => undefined}
        onActionHighlight={() => undefined}
        onActionDuplicate={() => undefined}
        onActionCopy={() => undefined}
        onActionDelete={() => undefined}
      />
    ));

    expect(html).toContain('aria-label="Actions for API"');
    expect(html).toContain("data-icon-tile");
    expect(html).toContain("Observed");
    expect(html).not.toContain("All environments");
    expect(html).not.toContain("preview");
  });

  it("keeps preview payloads out of architecture fixtures", () => {
    expect("nodePreviews" in ARCH_CANVAS_FIXTURE).toBe(false);
    expect(ARCH_CANVAS_FIXTURE.nodes.every((node) => !("preview" in node))).toBe(true);
  });

  it("verifies ArchitectureNodeCard external positioner drag transform", () => {
    const html = renderToString(() => (
      <ArchitectureNodeCard
        name="API"
        type="api"
        state="observed"
        x={0}
        y={0}
        width={188}
        height={56}
        selected={false}
        dimmed={false}
        lodFar={false}
        editing={false}
        onRenameEnd={() => undefined}
        onRenameCancel={() => undefined}
        onActionOpenDetails={() => undefined}
        onActionRename={() => undefined}
        onActionConnect={() => undefined}
        onActionHighlight={() => undefined}
        onActionDuplicate={() => undefined}
        onActionCopy={() => undefined}
        onActionDelete={() => undefined}
        dragOffset={{ dx: 10, dy: 20 }}
      />
    ));
    expect(html).toContain('class="architecture-node-positioner"');
    expect(html).toMatch(/transform:\s*translate3d\(10px, 20px, 0\)/);
  });

  it("checks single entry animation completion, 0.97 scale CSS, no opacity entry, and reduced-motion removal", () => {
    const css = readFileSync(here("./architecture-canvas-host.css"), "utf8");
    const source = readFileSync(here("./architecture-node-card.tsx"), "utf8");
    const html = renderToString(() => (
      <ArchitectureNodeCard
        name="API"
        type="api"
        state="observed"
        x={0}
        y={0}
        width={188}
        height={56}
        selected={false}
        dimmed={false}
        lodFar={false}
        editing={false}
        onRenameEnd={() => undefined}
        onRenameCancel={() => undefined}
        onActionOpenDetails={() => undefined}
        onActionRename={() => undefined}
        onActionConnect={() => undefined}
        onActionHighlight={() => undefined}
        onActionDuplicate={() => undefined}
        onActionCopy={() => undefined}
        onActionDelete={() => undefined}
        entering={true}
      />
    ));
    expect(html).toContain('data-entering=""');
    const entryKeyframes = css.match(
      /@keyframes architecture-node-enter\s*{[\s\S]*?\n}/,
    )?.[0] ?? "";
    expect(entryKeyframes).toContain("scale: 0.97");
    expect(entryKeyframes).not.toContain("opacity");
    expect(css).toContain("animation: architecture-node-enter 200ms var(--ease-enter) forwards");
    expect(css).toMatch(
      /@media \(prefers-reduced-motion: reduce\)[\s\S]*?animation: none;/,
    );
    expect(source).toContain('event.animationName === "architecture-node-enter"');
    expect(source).toContain("props.onEnterEnd?.()");
  });

  it("keeps selection instant while dimming surface, text, and icon over 120ms", () => {
    const css = readFileSync(here("./architecture-canvas-host.css"), "utf8");
    const selectedRule = css.match(
      /\.architecture-node-card\[data-selected\]\s*{[^}]*}/,
    )?.[0] ?? "";

    expect(css).toMatch(/background-color 120ms var\(--ease-standard\)/);
    expect(css).toMatch(/color 120ms var\(--ease-standard\)/);
    expect(css).toContain(".architecture-node-card .pw-icon-tile");
    expect(selectedRule).toContain("outline: 2px solid var(--color-primary)");
    expect(selectedRule).not.toContain("transition");
  });
});
