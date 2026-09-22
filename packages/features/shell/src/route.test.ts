import { describe, expect, it } from "vitest";
import { DEFAULT_TABS, SECTION_TABS, createDefaultRoute, enterProject, isNavSectionId, resolveRoute } from "./route";

describe("first version routes", () => {
  it("opens Architecture Canvas", () => {
    expect(createDefaultRoute()).toMatchObject({ section: "architecture", tab: "canvas" });
  });

  it("exposes only retained project surfaces", () => {
    expect(
      ["architecture", "resources", "knowledge", "agents", "integrations", "settings"].every(
        isNavSectionId,
      ),
    ).toBe(true);
    expect(isNavSectionId("operations")).toBe(false);
    expect(isNavSectionId("terminal")).toBe(false);
    expect(SECTION_TABS.architecture?.map((tab) => tab.id)).toEqual(["canvas", "views"]);
    expect(SECTION_TABS.knowledge?.map((tab) => tab.id)).toEqual(["specs", "library", "sources"]);
    expect(SECTION_TABS.resources?.map((tab) => tab.id)).toEqual(["inventory"]);
    expect(SECTION_TABS.agents?.map((tab) => tab.id)).toEqual(["access", "compile", "history"]);
    expect(SECTION_TABS.settings?.map((tab) => tab.id)).toEqual([
      "general",
      "environments",
      "shortcuts",
      "advanced",
    ]);
    expect(DEFAULT_TABS.agents).toBe("access");
  });

  it("switches local projects back to Architecture", () => {
    expect(
      enterProject(createDefaultRoute("architecture"), "project-two", {
        environmentId: "local",
      }),
    ).toMatchObject({
      projectId: "project-two",
      environmentId: "local",
      section: "architecture",
      tab: "canvas",
    });
  });

  it("falls back unknown and condemned routes to Architecture", () => {
    expect(resolveRoute("unknown")).toEqual({ section: "architecture", tab: "canvas" });
    expect(resolveRoute("operations")).toEqual({ section: "architecture", tab: "canvas" });
    expect(resolveRoute("terminal")).toEqual({ section: "architecture", tab: "canvas" });
    expect(resolveRoute("resources", "nodes")).toEqual({
      section: "resources",
      tab: "inventory",
    });
    expect(resolveRoute("architecture", "components")).toEqual({
      section: "architecture",
      tab: "canvas",
    });
    expect(DEFAULT_TABS.integrations).toBe("connected");
  });

  it("maps legacy Knowledge tabs to Library", () => {
    expect(resolveRoute("knowledge", "docs")).toEqual({ section: "knowledge", tab: "library" });
    expect(resolveRoute("knowledge", "adrs")).toEqual({ section: "knowledge", tab: "library" });
    expect(resolveRoute("knowledge", "runbooks")).toEqual({ section: "knowledge", tab: "library" });
  });
});
