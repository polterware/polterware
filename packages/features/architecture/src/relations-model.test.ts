import { describe, expect, it } from "vitest";
import type { AtlasCanvasNode } from "./atlas-canvas";
import { buildRelations, filterRelations } from "./relations-model";

function node(id: string, level: AtlasCanvasNode["level"], uses: string[] = []): AtlasCanvasNode {
  return {
    id,
    parentId: "",
    level,
    slug: id,
    path: id,
    title: id,
    intent: "",
    decisions: [],
    invariants: [],
    status: "delivered",
    childCount: 0,
    featureCount: 0,
    openQuestions: 0,
    repo: "",
    folder: "",
    migrationFrom: "",
    migrationTo: "",
    uses,
  };
}

describe("buildRelations", () => {
  it("collects uses and usedBy for both ends of a relation", () => {
    const result = buildRelations([node("a", "component", ["b"]), node("b", "component")]);
    expect(result.map((row) => row.node.id)).toEqual(["a", "b"]);
    expect(result[0]?.uses.map((n) => n.id)).toEqual(["b"]);
    expect(result[1]?.usedBy.map((n) => n.id)).toEqual(["a"]);
  });

  it("skips systems, unknown paths, self references and unrelated nodes", () => {
    const result = buildRelations([
      node("sys", "system", ["a"]),
      node("a", "component", ["a", "missing", "sys"]),
      node("lonely", "domain"),
    ]);
    expect(result).toEqual([]);
  });
});

describe("filterRelations", () => {
  const rows = buildRelations([
    { ...node("a", "component", ["b"]), parentId: "d1", title: "Alpha" },
    { ...node("b", "component"), parentId: "d2", title: "Beta" },
  ]);
  const none = { query: "", domainIds: new Set<string>(), direction: null } as const;

  it("matches the query against the node and its neighbours", () => {
    expect(filterRelations(rows, { ...none, query: "alp" }).map((r) => r.node.id)).toEqual(["a", "b"]);
    expect(filterRelations(rows, { ...none, query: "zzz" })).toEqual([]);
  });

  it("filters by domain and direction, trimming the other side", () => {
    expect(filterRelations(rows, { ...none, domainIds: new Set(["d2"]) }).map((r) => r.node.id)).toEqual(["b"]);
    const usesOnly = filterRelations(rows, { ...none, direction: "uses" });
    expect(usesOnly.map((r) => r.node.id)).toEqual(["a"]);
    const usedByOnly = filterRelations(rows, { ...none, direction: "usedBy" });
    expect(usedByOnly.map((r) => r.node.id)).toEqual(["b"]);
    expect(usedByOnly[0]?.uses).toEqual([]);
  });
});
