// @vitest-environment jsdom

import { readFileSync } from "node:fs";
import { afterEach, describe, expect, it, vi } from "vitest";
import { render } from "solid-js/web";
import { createSignal } from "solid-js";
import { ContextMenu } from "./context-menu";
import { repoPath } from "../test/repo-path";

afterEach(() => {
  document.body.replaceChildren();
});

describe("ContextMenu", () => {
  it("exposes the compound API and fixed virtual-anchor positioning", () => {
    const source = readFileSync(
      repoPath("packages/frontend/design-system/src/context-menu.tsx"),
      "utf8",
    );

    expect(Object.keys(ContextMenu)).toEqual(["Root", "Content", "Item", "Separator"]);
    expect(source).toContain("getAnchorRect");
    expect(source).toContain('strategy: "fixed"');
    expect(source).toContain('placement: "bottom-start"');
    expect(source).toContain("...props.anchorPoint, width: 0, height: 0");
  });

  it("controls open state and renders Ark content", async () => {
    const host = document.createElement("div");
    document.body.append(host);
    const [open, setOpen] = createSignal(true);
    const dispose = render(() => (
      <ContextMenu.Root open={open()} onOpenChange={setOpen} anchorPoint={{ x: 10, y: 10 }}>
        <ContextMenu.Content>
          <ContextMenu.Item value="test">Item</ContextMenu.Item>
        </ContextMenu.Content>
      </ContextMenu.Root>
    ), host);

    await vi.waitFor(() => {
      expect(document.querySelector(".pw-dropdown-presence")).not.toBeNull();
    });

    setOpen(false);
    await vi.waitFor(() => {
      const content = document.querySelector(".pw-dropdown-presence");
      expect(content === null || content.getAttribute("data-state") === "closed").toBe(true);
    });

    dispose();
    host.remove();
  });

  it("selects an item through Ark and reports the close", async () => {
    const host = document.createElement("div");
    document.body.append(host);
    const [open, setOpen] = createSignal(true);
    const onSelect = vi.fn();
    const dispose = render(() => (
      <ContextMenu.Root open={open()} onOpenChange={setOpen} anchorPoint={{ x: 10, y: 10 }}>
        <ContextMenu.Content>
          <ContextMenu.Item value="test" onSelect={onSelect}>Item</ContextMenu.Item>
        </ContextMenu.Content>
      </ContextMenu.Root>
    ), host);

    await vi.waitFor(() => {
      expect(document.querySelector("[role='menuitem']")).not.toBeNull();
    });

    const item = document.querySelector<HTMLElement>("[role='menuitem']");
    expect(item).not.toBeNull();
    if (!item) return;
    // Ark highlights an item on pointer move before a click selects it.
    item.dispatchEvent(new PointerEvent("pointermove", { bubbles: true, pointerType: "mouse" }));
    item.dispatchEvent(new PointerEvent("pointerdown", { bubbles: true, pointerType: "mouse", button: 0 }));
    item.dispatchEvent(new PointerEvent("pointerup", { bubbles: true, pointerType: "mouse", button: 0 }));
    item.click();
    await vi.waitFor(() => expect(onSelect).toHaveBeenCalled());
    await vi.waitFor(() => expect(open()).toBe(false));

    dispose();
    host.remove();
  });

  it("dismisses on Escape", async () => {
    const host = document.createElement("div");
    document.body.append(host);
    const [open, setOpen] = createSignal(true);
    const dispose = render(() => (
      <ContextMenu.Root open={open()} onOpenChange={setOpen} anchorPoint={{ x: 10, y: 10 }}>
        <ContextMenu.Content>
          <ContextMenu.Item value="test">Item</ContextMenu.Item>
        </ContextMenu.Content>
      </ContextMenu.Root>
    ), host);

    await vi.waitFor(() => {
      expect(document.querySelector("[role='menuitem']")).not.toBeNull();
    });

    document.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape", bubbles: true }));
    await vi.waitFor(() => {
      expect(open()).toBe(false);
    });

    dispose();
    host.remove();
  });
});
