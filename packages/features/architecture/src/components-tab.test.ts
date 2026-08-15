import { describe, expect, it } from "vitest";
import type { ArchNode } from "@polterware/frontend-canvas";
import {
  componentsDetailTarget,
  filterComponentRows,
  projectComponentRows,
} from "./components-list";
import { ARCHITECTURE_TABS, normalizeArchitectureTab } from "./tab";

function node(partial: Partial<ArchNode> & Pick<ArchNode, "id" | "name" | "type">): ArchNode {
  return {
    description: undefined,
    subtype: undefined,
    origin: "manual",
    provider: undefined,
    environmentIds: ["production"],
    state: "observed",
    x: 0,
    y: 0,
    width: 180,
    height: 72,
    zIndex: 0,
    linkStatus: "unlinked",
    resourceId: null,
    owner: undefined,
    tags: [],
    ...partial,
  };
}

describe("normalizeArchitectureTab", () => {
  it("keeps components as components (never aliases to canvas)", () => {
    expect(normalizeArchitectureTab("components")).toBe("components");
    expect(normalizeArchitectureTab("Components")).toBe("components");
  });

  it("exposes Canvas · Components · Views in order", () => {
    expect(ARCHITECTURE_TABS.map((tab) => tab.id)).toEqual([
      "canvas",
      "components",
      "views",
    ]);
  });

  it("maps only retired tabs away from components", () => {
    expect(normalizeArchitectureTab("dependencies")).toBe("canvas");
    expect(normalizeArchitectureTab("changes")).toBe("canvas");
    expect(normalizeArchitectureTab("canvas")).toBe("canvas");
    expect(normalizeArchitectureTab("views")).toBe("views");
  });
});

describe("projectComponentRows (SSOT)", () => {
  it("projects the same node ids as the architecture document", () => {
    const nodes = [
      node({ id: "n-pay", name: "Payment Service", type: "service", state: "observed" }),
      node({
        id: "n-pg",
        name: "PostgreSQL",
        type: "database",
        state: "linked",
        linkStatus: "linked",
        owner: "Data",
      }),
    ];
    const rows = projectComponentRows(nodes);
    expect(rows.map((row) => row.id)).toEqual(["n-pay", "n-pg"]);
    expect(rows[0]?.lifecycleLabel).toBe("Observed");
    expect(rows[1]?.linkLabel).toBe("Linked");
    expect(rows[1]?.owner).toBe("Data");
  });

  it("does not invent rows when the document is empty", () => {
    expect(projectComponentRows([])).toEqual([]);
  });
});

describe("filterComponentRows", () => {
  const rows = projectComponentRows([
    node({ id: "a", name: "Auth", type: "service", state: "observed" }),
    node({
      id: "b",
      name: "Cache",
      type: "cache",
      state: "planned",
      environmentIds: ["staging"],
    }),
  ]);

  it("filters by lifecycle and type", () => {
    expect(filterComponentRows(rows, { lifecycle: ["planned"] }).map((r) => r.id)).toEqual(["b"]);
    expect(filterComponentRows(rows, { type: ["service"] }).map((r) => r.id)).toEqual(["a"]);
  });

  it("filters by environment", () => {
    expect(filterComponentRows(rows, { environment: ["staging"] }).map((r) => r.id)).toEqual([
      "b",
    ]);
  });
});

describe("components list activation contract", () => {
  it("targets global arch-node detail under architecture:components", () => {
    expect(componentsDetailTarget("n-pay")).toEqual({
      scopeId: "architecture:components",
      entityId: "n-pay",
      entityKind: "arch-node",
    });
  });
});
