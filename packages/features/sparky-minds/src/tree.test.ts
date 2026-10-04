import { describe, expect, it } from "vitest";
import {
  buildMindTree,
  canMoveMind,
  cascadeDeleteIds,
  deletePreviewCopy,
  mindAncestors,
  moveTargetOptions,
  readExpandedMindIds,
  siblingOrder,
  type MindNode,
} from "./tree";

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

  it("rejects self and descendant moves and allows a root parent", () => {
    expect(canMoveMind(minds, "a", "a")).toBe(false);
    expect(canMoveMind(minds, "a", "c")).toBe(false);
    expect(canMoveMind(minds, "c", "a")).toBe(true);
    expect(canMoveMind(minds, "b", "")).toBe(true);
  });

  it("reads persisted expansion state safely", () => {
    expect(readExpandedMindIds({ getItem: () => '["a","b"]' })).toEqual(["a", "b"]);
    expect(readExpandedMindIds({ getItem: () => "invalid" })).toEqual([]);
  });

  it("matches core cascade ids and delete copy from {deletedMindIds, movedMemoryCount}", () => {
    expect(cascadeDeleteIds(minds, "a")).toEqual(["a", "b", "c"]);
    expect(deletePreviewCopy({ deletedMindIds: ["a", "b", "c"], movedMemoryCount: 4 })).toBe(
      "Deletes 2 child Minds; 4 memories move to Inbox.",
    );
  });

  it("walks ancestors and reorders siblings without wrapping", () => {
    expect(mindAncestors(minds, "c").map((mind) => mind.id)).toEqual(["a", "b"]);
    expect(siblingOrder(minds, "a", "b", 1)).toBeUndefined();
    expect(siblingOrder(
      [
        { id: "x", parentId: "p", name: "X", color: "", icon: "brain", sortOrder: 0 },
        { id: "y", parentId: "p", name: "Y", color: "", icon: "brain", sortOrder: 1 },
      ],
      "p",
      "x",
      1,
    )).toEqual(["y", "x"]);
  });

  it("disables cyclic Move to targets in the option list", () => {
    const options = moveTargetOptions(minds, "a");
    expect(options.find((option) => option.value === "")?.disabled).toBe(false);
    expect(options.find((option) => option.value === "a")?.disabled).toBe(true);
    expect(options.find((option) => option.value === "c")?.disabled).toBe(true);
    expect(options.find((option) => option.value === "b")?.disabled).toBe(true);
  });
});
