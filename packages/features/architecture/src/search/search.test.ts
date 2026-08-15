import { describe, expect, it } from "vitest";
import { createEmptyDocument, docAddNode } from "@polterware/frontend-canvas";
import { searchDocument } from "./document-search";

describe("document search", () => {
  it("ranks exact name higher", () => {
    let doc = createEmptyDocument({ systemId: "s" });
    doc = docAddNode(doc, { name: "Payment Service", type: "service", x: 0, y: 0, id: "a" });
    doc = docAddNode(doc, { name: "Pay API", type: "api", x: 0, y: 0, id: "b" });
    const hits = searchDocument(doc, "Payment Service");
    expect(hits[0]!.id).toBe("a");
  });
});
