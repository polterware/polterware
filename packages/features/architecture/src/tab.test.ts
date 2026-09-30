import { describe, expect, it } from "vitest";
import { ARCHITECTURE_TABS, normalizeArchitectureTab } from "./tab";

describe("normalizeArchitectureTab", () => {
  it("exposes Map and Components", () => {
    expect(ARCHITECTURE_TABS.map((tab) => tab.id)).toEqual(["map", "components"]);
  });

  it("keeps Components and maps retired tabs to the map", () => {
    expect(normalizeArchitectureTab("map")).toBe("map");
    expect(normalizeArchitectureTab("components")).toBe("components");
    expect(normalizeArchitectureTab("Components")).toBe("components");
    expect(normalizeArchitectureTab("canvas")).toBe("map");
    expect(normalizeArchitectureTab("hierarchy")).toBe("map");
    expect(normalizeArchitectureTab("views")).toBe("map");
    expect(normalizeArchitectureTab("dependencies")).toBe("map");
    expect(normalizeArchitectureTab("changes")).toBe("map");
    expect(normalizeArchitectureTab(undefined)).toBe("map");
  });
});
