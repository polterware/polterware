import { createSignal } from "solid-js";
import { render } from "solid-js/web";
import { Button } from "./button";
import { afterEach, describe, expect, it, vi } from "vitest";
import { readFileSync } from "node:fs";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuSub,
  DropdownMenuRadioGroup,
  DropdownMenuRadioItem,
  DropdownMenuTrigger,
} from "./dropdown-menu";
import { repoPath } from "../test/repo-path";

describe("DropdownMenu", () => {
  it("exports compound components", () => {
    expect(DropdownMenu).toBeDefined();
    expect(DropdownMenuTrigger).toBeDefined();
    expect(DropdownMenuContent).toBeDefined();
    expect(DropdownMenuItem).toBeDefined();
    expect(DropdownMenuSeparator).toBeDefined();
    expect(DropdownMenuSub).toBeDefined();
    expect(DropdownMenuRadioGroup).toBeDefined();
    expect(DropdownMenuRadioItem).toBeDefined();
  });

  it("forwards custom positioning to Ark Menu.Root", () => {
    const source = readFileSync(
      repoPath("packages/frontend/design-system/src/dropdown-menu.tsx"),
      "utf8",
    );
    expect(source).toContain('positioning?: MenuRootProps["positioning"]');
    expect(source).toContain("positioning={props.positioning}");
  });

  it("routes a custom trigger id through Root ids so Zag keeps its anchor", () => {
    const source = readFileSync(repoPath("packages/frontend/design-system/src/dropdown-menu.tsx"), "utf8");
    const select = readFileSync(repoPath("packages/frontend/design-system/src/select.tsx"), "utf8");
    expect(source).toContain("ids={props.triggerId ? { trigger: props.triggerId } : undefined}");
    // An `id` on the trigger overrides Zag's id: no anchor, menu at top-left behind dialogs.
    expect(source).not.toMatch(/\sid=\{props\.id\}/);
    expect(select).toContain("triggerId={props.id}");
    expect(select).not.toMatch(/\sid=\{props\.id\}/);
  });

  it("restores aria-controls so a dialog's focus trap accepts the portalled menu", () => {
    const source = readFileSync(repoPath("packages/frontend/design-system/src/dropdown-menu.tsx"), "utf8");
    expect(source).toContain("menu().open ? menu().getContentProps().id : undefined");
    expect(source).toContain("aria-controls={controls()}");
  });

  it("springs the surface from its resolved anchor side", () => {
    const motion = readFileSync(repoPath("packages/frontend/design-system/src/popup-motion.ts"), "utf8");
    const source = readFileSync(repoPath("packages/frontend/design-system/src/dropdown-menu.tsx"), "utf8");
    expect(source).toContain("createPopupMotion({ scale: true })");
    expect(motion).toContain('side === "top" ? slide : side === "bottom" ? -slide : 0');
    expect(motion).toContain('side === "left" ? slide : side === "right" ? -slide : 0');
    expect(motion).toContain("dataset.placement");
    expect(motion).toContain("spring.fast.exit");
  });

  it("keeps a no-paint presence clock so Ark waits for the exit tween", () => {
    const css = readFileSync(repoPath("packages/frontend/design-system/src/dropdown-menu.css"), "utf8");
    expect(css).toMatch(/@keyframes pw-dropdown-presence {[\s\S]*--pw-dropdown-presence-clock: 1;/);
    expect(css).toMatch(/animation: pw-dropdown-presence var\(--duration-fast, 120ms\) forwards;/);
    expect(css).toContain("transform-origin: var(--transform-origin, top left)");
  });

  it("drops travel but keeps the fade under reduced motion", () => {
    const motion = readFileSync(repoPath("packages/frontend/design-system/src/popup-motion.ts"), "utf8");
    expect(motion).toContain("prefersReducedMotion()");
    expect(motion).toContain("const slide = reduced ? 0 : POPUP_SLIDE;");
  });

  it("moves one shared row highlight from the pointer and Ark's keyboard highlight", () => {
    const source = readFileSync(repoPath("packages/frontend/design-system/src/dropdown-menu.tsx"), "utf8");
    expect(source).toContain("menu().highlightedValue");
    expect(source).toContain("api.setHighlightedValue(value)");
    expect(source).toContain("<MenuRowsLayer rows={rows} />");
  });
});

const disposers: Array<() => void> = [];
afterEach(() => {
  for (const dispose of disposers.splice(0)) dispose();
  document.body.replaceChildren();
});

function mountMenu(view: () => any) {
  const host = document.createElement("div");
  document.body.append(host);
  disposers.push(render(view, host));
}

describe("DropdownMenu choices", () => {
  it("uses a shared icon button as its trigger without nesting buttons", async () => {
    mountMenu(() => (
      <DropdownMenu>
        <DropdownMenuTrigger aria-label="Display" asChild={(trigger) => (
          <Button {...trigger()} size="icon" variant="ghost" aria-label="Display">D</Button>
        )}>{null}</DropdownMenuTrigger>
        <DropdownMenuContent><DropdownMenuItem value="new">New section</DropdownMenuItem></DropdownMenuContent>
      </DropdownMenu>
    ));
    const button = document.querySelector<HTMLButtonElement>('button[aria-label="Display"]')!;
    expect(button.querySelector("button")).toBeNull();
    button.click();
    await vi.waitFor(() => expect(document.querySelector('[role="menuitem"]')?.textContent).toBe("New section"));
  });

  it("keeps radio checks exclusive and reports a picked value", async () => {
    const picked = vi.fn();
    mountMenu(() => {
      const [value, setValue] = createSignal("conversation");
      return (
        <DropdownMenu open>
          <DropdownMenuTrigger>Display</DropdownMenuTrigger>
          <DropdownMenuContent>
            <DropdownMenuRadioGroup value={value()} onValueChange={(next) => { setValue(next); picked(next); }} aria-label="Group by">
              <DropdownMenuRadioItem value="conversation">Conversation</DropdownMenuRadioItem>
              <DropdownMenuRadioItem value="workspace">Workspace</DropdownMenuRadioItem>
            </DropdownMenuRadioGroup>
          </DropdownMenuContent>
        </DropdownMenu>
      );
    });
    await vi.waitFor(() => expect(document.querySelectorAll('[role="menuitemradio"]')).toHaveLength(2));
    const rows = document.querySelectorAll<HTMLElement>('[role="menuitemradio"]');
    expect(rows[0]!.getAttribute("aria-checked")).toBe("true");
    expect(rows[1]!.getAttribute("aria-checked")).toBe("false");
    rows[1]!.click();
    await vi.waitFor(() => expect(picked).toHaveBeenCalledWith("workspace"));
    expect(rows[0]!.getAttribute("aria-checked")).toBe("false");
    expect(rows[1]!.getAttribute("aria-checked")).toBe("true");
  });

  it("opens a nested menu with ArrowRight and selects its radio option", async () => {
    const picked = vi.fn();
    mountMenu(() => (
      <DropdownMenu defaultOpen>
        <DropdownMenuTrigger>Display</DropdownMenuTrigger>
        <DropdownMenuContent>
          <DropdownMenuSub label="Group by">
            <DropdownMenuRadioGroup value="conversation" onValueChange={picked}>
              <DropdownMenuRadioItem value="conversation">Conversation</DropdownMenuRadioItem>
              <DropdownMenuRadioItem value="workspace">Workspace</DropdownMenuRadioItem>
            </DropdownMenuRadioGroup>
          </DropdownMenuSub>
        </DropdownMenuContent>
      </DropdownMenu>
    ));
    await vi.waitFor(() => expect(document.querySelector('[role="menuitem"][aria-haspopup="menu"]')).not.toBeNull());
    const parent = document.querySelector<HTMLElement>('[role="menu"]')!;
    parent.dispatchEvent(new KeyboardEvent("keydown", { key: "ArrowDown", bubbles: true }));
    parent.dispatchEvent(new KeyboardEvent("keydown", { key: "ArrowRight", bubbles: true }));
    await vi.waitFor(() => expect(document.querySelectorAll('[role="menuitemradio"]')).toHaveLength(2));
    document.querySelectorAll<HTMLElement>('[role="menuitemradio"]')[1]!.click();
    await vi.waitFor(() => expect(picked).toHaveBeenCalledWith("workspace"));
  });
});
