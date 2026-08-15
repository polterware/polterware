import { describe, expect, it } from "vitest";
import { DEFAULT_TABS, SECTION_TABS, createDefaultRoute, enterProject, isNavSectionId, resolveRoute } from "./route";

describe("first version routes", () => {
  it("opens Architecture Canvas", () => {
    expect(createDefaultRoute()).toMatchObject({ section: "architecture", tab: "canvas" });
  });

  it("exposes only retained project surfaces", () => {
    expect(["architecture", "resources", "knowledge", "operations", "terminal", "integrations", "settings"].every(isNavSectionId)).toBe(true);
    expect(SECTION_TABS.knowledge?.map((tab) => tab.id)).toEqual(["docs", "runbooks"]);
    expect(SECTION_TABS.operations?.map((tab) => tab.id)).toEqual(["runs", "workflows", "schedules"]);
  });

  it("switches local projects back to Architecture", () => {
    expect(enterProject(createDefaultRoute("terminal"), "project-two", { environmentId: "local" })).toMatchObject({ projectId: "project-two", environmentId: "local", section: "architecture", tab: "canvas" });
  });

  it("falls back unknown routes to Architecture", () => {
    expect(resolveRoute("unknown")).toEqual({ section: "architecture", tab: "canvas" });
    expect(DEFAULT_TABS.integrations).toBe("connected");
  });
});
