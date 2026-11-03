// @vitest-environment jsdom

import { describe, expect, it } from "vitest";
import { render } from "solid-js/web";
import {
  AddTeam,
  Copy,
  Delete,
  FloppyDisk,
  HugeiconsIcon,
  Pause,
  POLTER_ICON_STROKE_WIDTH,
  Plus,
  PlusIcon,
  Search,
  Stop,
  type HugeIconDefinition,
} from "./hugeicons";

const TEST_ICON = [
  [
    "path",
    {
      d: "M4 12H20",
      key: "test-path",
      stroke: "currentColor",
      strokeLinecap: "round",
      strokeLinejoin: "round",
      strokeWidth: "1.5",
    },
  ],
] as const satisfies HugeIconDefinition;

describe("HugeiconsIcon", () => {
  it("renders normalized decorative SVG attributes", () => {
    const host = document.createElement("div");
    const dispose = render(
      () => (
        <HugeiconsIcon
          icon={TEST_ICON}
          size={16}
          class="test-icon"
        />
      ),
      host,
    );

    const svg = host.querySelector("svg");
    const path = host.querySelector("path");

    expect(svg?.getAttribute("width")).toBe("16");
    expect(svg?.getAttribute("height")).toBe("16");
    expect(svg?.getAttribute("viewBox")).toBe("0 0 24 24");
    expect(svg?.getAttribute("color")).toBe("currentColor");
    expect(svg?.getAttribute("class")).toBe("test-icon");
    expect(svg?.getAttribute("aria-hidden")).toBe("true");
    expect(svg?.getAttribute("focusable")).toBe("false");
    expect(path?.getAttribute("stroke-linecap")).toBe("round");
    expect(path?.getAttribute("stroke-linejoin")).toBe("round");
    expect(path?.getAttribute("stroke-width")).toBe(
      String(POLTER_ICON_STROKE_WIDTH),
    );
    expect(path?.hasAttribute("key")).toBe(false);

    dispose();
  });

  it("supports an explicit stroke width", () => {
    const host = document.createElement("div");
    const dispose = render(
      () => (
        <HugeiconsIcon
          icon={TEST_ICON}
          strokeWidth={2}
        />
      ),
      host,
    );

    expect(host.querySelector("path")?.getAttribute("stroke-width")).toBe("2");

    dispose();
  });

  it("exposes curated Solid components and the PlusIcon alias", () => {
    const host = document.createElement("div");
    const dispose = render(() => <Search size={18} />, host);

    expect(host.querySelector("svg")?.getAttribute("width")).toBe("18");
    expect(host.querySelectorAll("path").length).toBeGreaterThan(0);
    expect(PlusIcon).toBe(Plus);
    for (const actionIcon of [
      AddTeam,
      Copy,
      Delete,
      FloppyDisk,
      Pause,
      Stop,
    ]) {
      expect(actionIcon).toBeDefined();
    }

    dispose();
  });
});
