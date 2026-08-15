import { describe, expect, it } from "vitest";
import { createEmptyDocument } from "../document/document";
import { addNode, updateNode } from "../document/mutations";
import { commit, createHistory, undo, redo } from "./history";

describe("history", () => {
  it("coalesces eventually into one undo", () => {
    let doc = createEmptyDocument({ systemId: "s" });
    doc = addNode(doc, { name: "A", type: "api", x: 0, y: 0, id: "n1" });
    let h = createHistory();

    const start = doc;
    // drag frames
    for (let i = 1; i <= 5; i++) {
      const before = doc;
      doc = updateNode(doc, "n1", { x: i * 10 });
      const r = commit(h, before, doc, "eventually");
      h = r.history;
      doc = r.doc;
    }
    const fin = commit(h, start, doc, "immediately", { label: "move" });
    h = fin.history;
    doc = fin.doc;

    expect(h.undoStack).toHaveLength(1);
    const u = undo(h, doc);
    expect(u.doc.nodes[0]!.x).toBe(0);
    const r = redo(u.history, u.doc);
    expect(r.doc.nodes[0]!.x).toBe(50);
  });

  it("invalidates redo on new edit", () => {
    let doc = createEmptyDocument({ systemId: "s" });
    doc = addNode(doc, { name: "A", type: "api", x: 0, y: 0, id: "n1" });
    let h = createHistory();
    let before = doc;
    doc = updateNode(doc, "n1", { name: "B" });
    ({ history: h, doc } = commit(h, before, doc, "immediately"));
    ({ history: h, doc } = undo(h, doc));
    expect(h.redoStack.length).toBe(1);
    before = doc;
    doc = updateNode(doc, "n1", { name: "C" });
    ({ history: h, doc } = commit(h, before, doc, "immediately"));
    expect(h.redoStack.length).toBe(0);
  });
});
