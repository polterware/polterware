import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import {
  resolveHandleState,
  shouldShowConnectionHandles,
} from "./connection-handles";

const here = dirname(fileURLToPath(import.meta.url));

describe("connection handles", () => {
  it("shows handles only during connection and never for notes or empty ports", () => {
    expect(shouldShowConnectionHandles({ type: "service" }, false)).toBe(false);
    expect(shouldShowConnectionHandles({ type: "service" }, true)).toBe(true);
    expect(shouldShowConnectionHandles({ type: "note" }, true)).toBe(false);
    expect(
      shouldShowConnectionHandles({ type: "service", ports: [] }, true),
    ).toBe(false);
  });

  it("resolves per-port handle focus independently of whole-card target highlight", () => {
    expect(
      resolveHandleState({
        nodeId: "a",
        side: "left",
        hoveredPort: { nodeId: "a", portId: "left" },
        isConnectTarget: false,
      }),
    ).toBe("focus");
    expect(
      resolveHandleState({
        nodeId: "a",
        side: "right",
        hoveredPort: { nodeId: "a", portId: "left" },
        isConnectTarget: true,
      }),
    ).toBe("target");
    expect(
      resolveHandleState({
        nodeId: "b",
        side: "top",
        hoveredPort: { nodeId: "a", portId: "left" },
        isConnectTarget: false,
      }),
    ).toBe("hot");
    expect(
      resolveHandleState({
        nodeId: "b",
        side: "bottom",
        hoveredPort: null,
        isConnectTarget: true,
      }),
    ).toBe("target");
  });

  it("animates focused connection handles with scale and reduced-motion safety", () => {
    const css = readFileSync(join(here, "architecture-canvas-host.css"), "utf8");
    expect(css).toContain(".architecture-canvas-handle-dot");
    expect(css).toMatch(
      /\[data-handle-state="focus"\][\s\S]*?transform:\s*scale\(1\.5\)/,
    );
    expect(css).toMatch(
      /\[data-handle-state="target"\][\s\S]*?transform:\s*scale\(1\.15\)/,
    );
    expect(css).toMatch(
      /@media \(prefers-reduced-motion: reduce\)[\s\S]*?\.architecture-canvas-handle-dot[\s\S]*?transition:\s*none/,
    );
    expect(css).toMatch(
      /prefers-reduced-motion: reduce[\s\S]*?data-handle-state="focus"[\s\S]*?transform:\s*none/,
    );
  });
});
