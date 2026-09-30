import { describe, expect, it } from "vitest";
import { DEFAULT_TABS, SECTION_TABS, createDefaultRoute, enterProject, isNavSectionId, resolveRoute } from "./route";

describe("first version routes", () => {
  it("opens Inbox", () => {
    expect(createDefaultRoute()).toMatchObject({ section: "inbox", tab: "all" });
  });

  it("exposes only the Atlas project surfaces", () => {
    expect(
      ["inbox", "atlas", "work", "decisions", "reality", "agents", "integrations"].every(
        isNavSectionId,
      ),
    ).toBe(true);
    expect(isNavSectionId("settings")).toBe(false);
    expect(isNavSectionId("architecture")).toBe(false);
    expect(isNavSectionId("knowledge")).toBe(false);
    expect(isNavSectionId("operations")).toBe(false);
    expect(isNavSectionId("terminal")).toBe(false);
    expect(SECTION_TABS.atlas?.map((tab) => tab.id)).toEqual(["map", "components"]);
    expect(SECTION_TABS.work?.map((tab) => tab.id)).toEqual(["board", "specs"]);
    expect(SECTION_TABS.decisions?.map((tab) => tab.id)).toEqual(["active", "challenged", "proposed", "superseded"]);
    expect(SECTION_TABS.reality?.map((tab) => tab.id)).toEqual([
      "inventory",
      "sources",
      "facts",
      "docs",
    ]);
    expect(SECTION_TABS.agents?.map((tab) => tab.id)).toEqual(["access", "sessions", "history", "context"]);
    expect(DEFAULT_TABS.agents).toBe("access");
    expect(DEFAULT_TABS.work).toBe("board");
  });

  it("switches local projects back to Inbox", () => {
    expect(
      enterProject(createDefaultRoute("inbox"), "project-two", {
        environmentId: "local",
      }),
    ).toMatchObject({
      projectId: "project-two",
      environmentId: "local",
      section: "inbox",
      tab: "all",
    });
  });

  it("falls back unknown routes to Inbox and condemned routes to Atlas", () => {
    expect(resolveRoute("unknown")).toEqual({ section: "inbox", tab: "all" });
    expect(resolveRoute("operations")).toEqual({ section: "atlas", tab: "map" });
    expect(resolveRoute("terminal")).toEqual({ section: "atlas", tab: "map" });
    expect(DEFAULT_TABS.integrations).toBe("connected");
  });

  it("sends the retired Canvas tab id to the Map", () => {
    expect(resolveRoute("atlas", "canvas")).toEqual({ section: "atlas", tab: "map" });
    expect(resolveRoute("atlas", "hierarchy")).toEqual({ section: "atlas", tab: "map" });
    expect(resolveRoute("atlas", "views")).toEqual({ section: "atlas", tab: "map" });
  });

  it("maps pre-Atlas sections to their new homes", () => {
    expect(resolveRoute("architecture", "views")).toEqual({ section: "atlas", tab: "map" });
    expect(resolveRoute("architecture", "components")).toEqual({ section: "atlas", tab: "components" });
    expect(resolveRoute("resources", "nodes")).toEqual({ section: "reality", tab: "inventory" });
    expect(resolveRoute("knowledge")).toEqual({ section: "work", tab: "specs" });
    expect(resolveRoute("knowledge", "specs")).toEqual({ section: "work", tab: "specs" });
    expect(resolveRoute("knowledge", "library")).toEqual({ section: "decisions", tab: "active" });
    expect(resolveRoute("knowledge", "adrs")).toEqual({ section: "decisions", tab: "active" });
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
