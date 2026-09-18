import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";

const root = dirname(fileURLToPath(import.meta.url));
const dialogSource = readFileSync(resolve(root, "create-environment-dialog.tsx"), "utf8");

describe("CreateEnvironmentDialog contract", () => {
  it("collects name, kind, and protection for local persistence", () => {
    expect(dialogSource).toContain("CreateEnvironmentDraft");
    expect(dialogSource).toContain('title="Add environment"');
    expect(dialogSource).toContain('label="Name"');
    expect(dialogSource).toContain('label="Kind"');
    expect(dialogSource).toContain('label="Protection"');
    expect(dialogSource).toContain("defaultProtectionForKind");
    expect(dialogSource).toContain("local SQLite database");
  });
});
