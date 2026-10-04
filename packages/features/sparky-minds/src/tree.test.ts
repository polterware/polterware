import { describe, expect, it } from "vitest";
import { buildMindTree, canMoveMind, readCollapsedMindIds, type MindNode } from "./tree";

const minds: MindNode[] = [
  { id: "a", parentId: "", name: "Alpha", color: "", icon: "brain", sortOrder: 1 },
  { id: "b", parentId: "a", name: "Beta", color: "", icon: "folder", sortOrder: 0 },
  { id: "c", parentId: "b", name: "Gamma", color: "", icon: "book", sortOrder: 0 },
];

describe("mind tree helpers", () => {
  it("builds a sorted hierarchy and promotes orphaned nodes to roots", () => {
    const tree = buildMindTree([...minds, { ...minds[0]!, id: "orphan", parentId: "missing" }]);
    expect(tree.map((node) => node.id)).toEqual(["a", "orphan"]);
    expect(tree[0]?.children[0]?.children[0]?.id).toBe("c");
  });

  it("rejects self and descendant moves", () => {
    expect(canMoveMind(minds, "a", "a")).toBe(false);
    expect(canMoveMind(minds, "a", "c")).toBe(false);
    expect(canMoveMind(minds, "c", "a")).toBe(true);
  });

  it("reads persisted collapse state safely", () => {
    expect(readCollapsedMindIds({ getItem: () => '["a","b"]' })).toEqual(["a", "b"]);
    expect(readCollapsedMindIds({ getItem: () => "invalid" })).toEqual([]);
  });
});
