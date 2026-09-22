import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = dirname(fileURLToPath(import.meta.url));

describe("Resources process start/stop contract", () => {
  it("exposes a start-process dialog for the local host", () => {
    const dialog = readFileSync(resolve(root, "create-process-dialog.tsx"), "utf8");
    expect(dialog).toContain('title="Start process"');
    expect(dialog).toContain("Runs on the local machine through polterware-core");
    expect(dialog).toContain("Remote nodes are not supported");
  });

  it("wires Start process, Stop, and empty state on the Processes tab", () => {
    const index = readFileSync(resolve(root, "index.tsx"), "utf8");
    const empty = readFileSync(resolve(root, "resources-empty.tsx"), "utf8");
    expect(index).toContain(">Start process<");
    expect(index).toContain("onStopProcess");
    expect(index).toContain('label="Stop"');
    expect(index).toContain("ResourcesProcessesEmpty");
    expect(empty).toContain("No local processes");
    expect(index).toContain("Local core");
    expect(index).not.toContain('label="Preview command"');
    expect(index).toContain("CreateProcessDialog");
  });
});
