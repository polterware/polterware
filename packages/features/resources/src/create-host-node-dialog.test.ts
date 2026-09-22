import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = dirname(fileURLToPath(import.meta.url));

describe("Resources host-node create contract", () => {
  it("exposes a dedicated SSH host dialog with alias, user, host, and port", () => {
    const dialogSource = readFileSync(resolve(root, "create-host-node-dialog.tsx"), "utf8");

    expect(dialogSource).toContain("export function CreateHostNodeDialog");
    expect(dialogSource).toContain('title="Add SSH host"');
    expect(dialogSource).toContain('Field label="Alias"');
    expect(dialogSource).toContain('Field label="User"');
    expect(dialogSource).toContain('Field label="Host"');
    expect(dialogSource).toContain('Field label="Port"');
    expect(dialogSource).toContain("currently selected environment from the header switcher");
    expect(dialogSource).toContain("Becomes a Terminal target");
    expect(dialogSource).toContain("ssh-agent");
  });

  it("uses a tab-aware create action and wires the host-node dialog", () => {
    const indexSource = readFileSync(resolve(root, "index.tsx"), "utf8");

    expect(indexSource).toContain('when={tab() === "inventory"}');
    expect(indexSource).toContain('label="Add resource"');
    expect(indexSource).toContain('when={tab() === "nodes"}');
    expect(indexSource).toContain('label="Add SSH host"');
    expect(indexSource).toContain("ToolbarIconButton");
    expect(indexSource).toContain("nodeDialogOpen");
    expect(indexSource).toContain("onCreateNode");
    expect(indexSource).toContain("onCloseNodeDialog");
    expect(indexSource).toContain("onSubmitNodeDialog");
    expect(indexSource).toContain("CreateHostNodeDialog");
    expect(indexSource).toContain("tab() === \"processes\"");
    expect(indexSource).toContain('label="Start process"');
  });
});
