import { describe, expect, it } from "vitest";
import { DEFAULT_TABS, SECTION_TABS, createDefaultRoute, enterProject, isNavSectionId, resolveRoute } from "./route";

describe("first version routes", () => {
  it("opens Atlas Canvas", () => {
    expect(createDefaultRoute()).toMatchObject({ section: "atlas", tab: "canvas" });
  });

  it("exposes only the Atlas project surfaces", () => {
    expect(
      ["inbox", "atlas", "work", "decisions", "reality", "agents", "integrations", "settings"].every(
        isNavSectionId,
      ),
    ).toBe(true);
    expect(isNavSectionId("architecture")).toBe(false);
    expect(isNavSectionId("knowledge")).toBe(false);
    expect(isNavSectionId("operations")).toBe(false);
    expect(isNavSectionId("terminal")).toBe(false);
    expect(SECTION_TABS.atlas?.map((tab) => tab.id)).toEqual(["canvas", "views"]);
    expect(SECTION_TABS.work?.map((tab) => tab.id)).toEqual(["board", "specs"]);
    expect(SECTION_TABS.decisions?.map((tab) => tab.id)).toEqual(["adrs"]);
    expect(SECTION_TABS.reality?.map((tab) => tab.id)).toEqual([
      "inventory",
      "sources",
      "facts",
      "docs",
    ]);
    expect(SECTION_TABS.agents?.map((tab) => tab.id)).toEqual(["access", "history", "context"]);
    expect(SECTION_TABS.settings?.map((tab) => tab.id)).toEqual([
      "general",
      "environments",
      "shortcuts",
      "advanced",
    ]);
    expect(DEFAULT_TABS.agents).toBe("access");
    expect(DEFAULT_TABS.work).toBe("board");
  });

  it("switches local projects back to Atlas", () => {
    expect(
      enterProject(createDefaultRoute("atlas"), "project-two", {
        environmentId: "local",
      }),
    ).toMatchObject({
      projectId: "project-two",
      environmentId: "local",
      section: "atlas",
      tab: "canvas",
    });
  });

  it("falls back unknown and condemned routes to Atlas", () => {
    expect(resolveRoute("unknown")).toEqual({ section: "atlas", tab: "canvas" });
    expect(resolveRoute("operations")).toEqual({ section: "atlas", tab: "canvas" });
    expect(resolveRoute("terminal")).toEqual({ section: "atlas", tab: "canvas" });
    expect(DEFAULT_TABS.integrations).toBe("connected");
  });

  it("maps pre-Atlas sections to their new homes", () => {
    expect(resolveRoute("architecture", "views")).toEqual({ section: "atlas", tab: "views" });
    expect(resolveRoute("architecture", "components")).toEqual({ section: "atlas", tab: "canvas" });
    expect(resolveRoute("resources", "nodes")).toEqual({ section: "reality", tab: "inventory" });
    expect(resolveRoute("knowledge")).toEqual({ section: "work", tab: "specs" });
    expect(resolveRoute("knowledge", "specs")).toEqual({ section: "work", tab: "specs" });
    expect(resolveRoute("knowledge", "library")).toEqual({ section: "decisions", tab: "adrs" });
    expect(resolveRoute("knowledge", "adrs")).toEqual({ section: "decisions", tab: "adrs" });
    expect(resolveRoute("knowledge", "docs")).toEqual({ section: "reality", tab: "docs" });
    expect(resolveRoute("knowledge", "runbooks")).toEqual({ section: "reality", tab: "docs" });
    expect(resolveRoute("knowledge", "sources")).toEqual({ section: "reality", tab: "sources" });
    expect(resolveRoute("agents", "facts")).toEqual({ section: "reality", tab: "facts" });
  });

  it("maps legacy Agents Compile and Drift tabs to Context", () => {
    expect(resolveRoute("agents", "compile")).toEqual({ section: "agents", tab: "context" });
    expect(resolveRoute("agents", "drift")).toEqual({ section: "agents", tab: "context" });
  });
});
