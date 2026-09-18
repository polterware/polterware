import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = dirname(fileURLToPath(import.meta.url));

describe("Architecture create-node persistence contract", () => {
  it("flushes layout immediately and reports a typed create result", () => {
    const source = readFileSync(resolve(root, "index.tsx"), "utf8");

    expect(source).toContain("async function submitCreate");
    expect(source).toContain("await ctrl.flush");
    expect(source).toContain('id: nodeId');
    expect(source).toContain("select(nodeId)");
    expect(source).toContain("onSubmitCreateNode?: (");
    expect(source).toContain("{ ok: true; nodeId: string }");
    expect(source).toContain("{ ok: false; error: string }");
    expect(source).not.toContain("onSubmitCreateNode?: (draft: ArchNodeDraft) => void");
  });
});
