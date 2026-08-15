import { describe, expect, it } from "vitest";
import { loadDocumentFromLayoutBody } from "./load";

describe("load document", () => {
  it("loads empty body as empty doc", () => {
    const { doc } = loadDocumentFromLayoutBody("", "sys-local");
    expect(doc.nodes).toHaveLength(0);
    expect(doc.formatVersion).toBe(1);
  });

  it("does not throw on corrupt json", () => {
    const { warnings, doc } = loadDocumentFromLayoutBody("{bad", "sys-local");
    expect(doc.nodes).toHaveLength(0);
    expect(warnings.some((w) => w.level === "error")).toBe(true);
  });
});
