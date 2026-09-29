import { describe, expect, it } from "vitest";
import { ARCHITECTURE_TABS, normalizeArchitectureTab } from "./tab";

describe("normalizeArchitectureTab", () => {
  it("exposes Map · Hierarchy · Views", () => {
    expect(ARCHITECTURE_TABS.map((tab) => tab.id)).toEqual(["map", "hierarchy", "views"]);
  });

  it("maps the retired Canvas and Components tabs to the map", () => {
    expect(normalizeArchitectureTab("canvas")).toBe("map");
    expect(normalizeArchitectureTab("components")).toBe("map");
    expect(normalizeArchitectureTab("Components")).toBe("map");
  });

  it("keeps the current tabs and maps other retired tabs to the map", () => {
    expect(normalizeArchitectureTab("map")).toBe("map");
    expect(normalizeArchitectureTab("hierarchy")).toBe("hierarchy");
    expect(normalizeArchitectureTab("views")).toBe("views");
    expect(normalizeArchitectureTab("dependencies")).toBe("map");
    expect(normalizeArchitectureTab("changes")).toBe("map");
    expect(normalizeArchitectureTab(undefined)).toBe("map");
  });
});
