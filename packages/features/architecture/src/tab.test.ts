import { describe, expect, it } from "vitest";
import { ARCHITECTURE_TABS, normalizeArchitectureTab } from "./tab";

describe("normalizeArchitectureTab", () => {
  it("exposes Canvas · Views only", () => {
    expect(ARCHITECTURE_TABS.map((tab) => tab.id)).toEqual(["canvas", "views"]);
  });

  it("maps retired Components tab to canvas", () => {
    expect(normalizeArchitectureTab("components")).toBe("canvas");
    expect(normalizeArchitectureTab("Components")).toBe("canvas");
  });

  it("keeps canvas and views; maps other retired tabs to canvas", () => {
    expect(normalizeArchitectureTab("canvas")).toBe("canvas");
    expect(normalizeArchitectureTab("views")).toBe("views");
    expect(normalizeArchitectureTab("dependencies")).toBe("canvas");
    expect(normalizeArchitectureTab("changes")).toBe("canvas");
  });
});
