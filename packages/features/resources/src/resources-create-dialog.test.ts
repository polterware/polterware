import { describe, expect, it } from "vitest";

describe("Resources create dialog contract", () => {
  it("submits name and type from the form draft", async () => {
    const { readFileSync } = await import("node:fs");
    const { dirname, resolve } = await import("node:path");
    const { fileURLToPath } = await import("node:url");
    const root = dirname(fileURLToPath(import.meta.url));
    const indexSource = readFileSync(resolve(root, "index.tsx"), "utf8");

    expect(indexSource).toContain("onSubmitDialog?: (draft: { name: string; type: string }) => void");
    expect(indexSource).toContain("onSubmitDialog?.({ name: name().trim(), type: type() })");
  });

  it("does not offer a fake Environment field in the dialog", async () => {
    const { readFileSync } = await import("node:fs");
    const { dirname, resolve } = await import("node:path");
    const { fileURLToPath } = await import("node:url");
    const root = dirname(fileURLToPath(import.meta.url));
    const indexSource = readFileSync(resolve(root, "index.tsx"), "utf8");

    expect(indexSource).not.toContain('Field label="Environment"');
    expect(indexSource).not.toContain('{ value: "Development", label: "Development" }');
    expect(indexSource).not.toContain('{ value: "Staging", label: "Staging" }');
    expect(indexSource).not.toContain('{ value: "Production", label: "Production" }');
  });
});
