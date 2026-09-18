import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = dirname(fileURLToPath(import.meta.url));

describe("Resources host-node create contract", () => {
  it("exposes a dedicated host-node dialog with name, platform, and shell", () => {
    const dialogSource = readFileSync(resolve(root, "create-host-node-dialog.tsx"), "utf8");

    expect(dialogSource).toContain("export function CreateHostNodeDialog");
    expect(dialogSource).toContain('title="Add node"');
    expect(dialogSource).toContain('Field label="Name"');
    expect(dialogSource).toContain('Field label="Platform"');
    expect(dialogSource).toContain('Field label="Shell"');
    expect(dialogSource).toContain("currently selected environment from the header switcher");
    expect(dialogSource).toContain("becomes a Terminal target");
  });

  it("uses a tab-aware create action and wires the host-node dialog", () => {
    const indexSource = readFileSync(resolve(root, "index.tsx"), "utf8");

    expect(indexSource).toContain('when={tab() === "inventory"}');
    expect(indexSource).toContain(">Add resource<");
    expect(indexSource).toContain('when={tab() === "nodes"}');
    expect(indexSource).toContain(">Add node<");
    expect(indexSource).toContain("nodeDialogOpen");
    expect(indexSource).toContain("onCreateNode");
    expect(indexSource).toContain("onCloseNodeDialog");
    expect(indexSource).toContain("onSubmitNodeDialog");
    expect(indexSource).toContain("CreateHostNodeDialog");
    expect(indexSource).toContain("tab() === \"processes\"");
  });
});
