import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = dirname(fileURLToPath(import.meta.url));

describe("Operations schedule honesty contract", () => {
  it("describes local cron execution instead of a visual-only schedule", () => {
    const dialog = readFileSync(resolve(root, "create-schedule-dialog.tsx"), "utf8");
    const index = readFileSync(resolve(root, "index.tsx"), "utf8");

    expect(dialog).not.toContain("It does not start a runtime.");
    expect(dialog).toContain("runs its workflow when due while the Desktop core is open");
    expect(dialog).toContain("Missed times while offline are skipped");

    expect(index).not.toContain("Schedule is visual and does not start a runtime.");
    expect(index).toContain("Runs locally through polterware-core when due");
    expect(index).toContain("Missed times while the core was offline are skipped");
  });
});
