// @vitest-environment jsdom

import { createSignal } from "solid-js";
import { render } from "solid-js/web";
import { readFileSync } from "node:fs";
import { afterEach, describe, expect, it, vi } from "vitest";
import { Tooltip } from "./tooltip";
import { repoPath } from "../test/repo-path";

afterEach(() => {
  vi.useRealTimers();
  document.body.replaceChildren();
});

async function advanceTooltipTime(milliseconds: number) {
  await vi.advanceTimersByTimeAsync(milliseconds);
  await Promise.resolve();
}

function dispatchTooltipPointer(
  element: Element,
  type: "pointermove" | "pointerleave",
) {
  const event = new MouseEvent(type, { bubbles: true });
  Object.defineProperty(event, "pointerType", { value: "mouse" });
  element.dispatchEvent(event);
}

describe("Tooltip", () => {
  it("renders grouped children without a DOM wrapper", () => {
    const host = document.createElement("div");
    document.body.append(host);
    const dispose = render(
      () => (
        <Tooltip.Group>
          <button>First</button>
          <button>Second</button>
        </Tooltip.Group>
      ),
      host,
    );

    expect(Object.keys(Tooltip)).toEqual(["Group", "Root", "Trigger", "Content"]);
    expect(Array.from(host.children, (element) => element.textContent)).toEqual([
      "First",
      "Second",
    ]);

    dispose();
  });

  it("keeps a group warm across adjacent tooltips and resets after cooldown", async () => {
    vi.useFakeTimers();
    const host = document.createElement("div");
    document.body.append(host);
    const firstChanges: boolean[] = [];
    const secondChanges: boolean[] = [];
    const dispose = render(
      () => (
        <Tooltip.Group>
          <Tooltip.Root onOpenChange={(open) => firstChanges.push(open)}>
            <Tooltip.Trigger asChild={(triggerProps) => (
              <button {...triggerProps()}>First action</button>
            )} />
            <Tooltip.Content>First label</Tooltip.Content>
          </Tooltip.Root>
          <Tooltip.Root onOpenChange={(open) => secondChanges.push(open)}>
            <Tooltip.Trigger asChild={(triggerProps) => (
              <button {...triggerProps()}>Second action</button>
            )} />
            <Tooltip.Content>Second label</Tooltip.Content>
          </Tooltip.Root>
        </Tooltip.Group>
      ),
      host,
    );
    const [first, second] = Array.from(host.querySelectorAll("button"));

    dispatchTooltipPointer(first!, "pointermove");
    await advanceTooltipTime(399);
    expect(firstChanges).toEqual([]);
    await advanceTooltipTime(1);
    expect(firstChanges).toEqual([true]);

    dispatchTooltipPointer(first!, "pointerleave");
    dispatchTooltipPointer(second!, "pointermove");
    await advanceTooltipTime(0);
    expect(secondChanges).toEqual([true]);
    await advanceTooltipTime(100);
    expect(firstChanges).toEqual([true, false]);

    dispatchTooltipPointer(second!, "pointerleave");
    await advanceTooltipTime(100);
    expect(secondChanges).toEqual([true, false]);
    await advanceTooltipTime(50);
    dispatchTooltipPointer(first!, "pointermove");
    await advanceTooltipTime(0);
    expect(firstChanges).toEqual([true, false, true]);

    dispatchTooltipPointer(first!, "pointerleave");
    await advanceTooltipTime(200);
    expect(firstChanges).toEqual([true, false, true, false]);
    dispatchTooltipPointer(second!, "pointermove");
    await advanceTooltipTime(399);
    expect(secondChanges).toEqual([true, false]);
    await advanceTooltipTime(1);
    expect(secondChanges).toEqual([true, false, true]);

    dispatchTooltipPointer(second!, "pointerleave");
    await advanceTooltipTime(100);
    const firstBefore = [...firstChanges];
    const secondBefore = [...secondChanges];
    dispose();
    // A disposed group must not keep emitting: flushing whatever timers remain changes nothing.
    vi.runOnlyPendingTimers();
    expect(firstChanges).toEqual(firstBefore);
    expect(secondChanges).toEqual(secondBefore);
  });

  it("preserves explicit delays and ungrouped defaults", async () => {
    vi.useFakeTimers();
    const host = document.createElement("div");
    document.body.append(host);
    const explicitChanges: boolean[] = [];
    const ungroupedChanges: boolean[] = [];
    const dispose = render(
      () => (
        <>
          <Tooltip.Group>
            <Tooltip.Root
              openDelay={25}
              closeDelay={30}
              onOpenChange={(open) => explicitChanges.push(open)}
            >
              <Tooltip.Trigger asChild={(triggerProps) => (
                <button {...triggerProps()}>Explicit action</button>
              )} />
              <Tooltip.Content>Explicit label</Tooltip.Content>
            </Tooltip.Root>
          </Tooltip.Group>
          <Tooltip.Root onOpenChange={(open) => ungroupedChanges.push(open)}>
            <Tooltip.Trigger asChild={(triggerProps) => (
              <button {...triggerProps()}>Ungrouped action</button>
            )} />
            <Tooltip.Content>Ungrouped label</Tooltip.Content>
          </Tooltip.Root>
        </>
      ),
      host,
    );
    const [explicit, ungrouped] = Array.from(host.querySelectorAll("button"));

    dispatchTooltipPointer(explicit!, "pointermove");
    await advanceTooltipTime(24);
    expect(explicitChanges).toEqual([]);
    await advanceTooltipTime(1);
    expect(explicitChanges).toEqual([true]);
    dispatchTooltipPointer(explicit!, "pointerleave");
    await advanceTooltipTime(29);
    expect(explicitChanges).toEqual([true]);
    await advanceTooltipTime(1);
    expect(explicitChanges).toEqual([true, false]);

    dispatchTooltipPointer(ungrouped!, "pointermove");
    await advanceTooltipTime(399);
    expect(ungroupedChanges).toEqual([]);
    await advanceTooltipTime(1);
    expect(ungroupedChanges).toEqual([true]);
    dispatchTooltipPointer(ungrouped!, "pointerleave");
    await advanceTooltipTime(99);
    expect(ungroupedChanges).toEqual([true]);
    await advanceTooltipTime(1);
    expect(ungroupedChanges).toEqual([true, false]);

    dispose();
  });

  it("portals accessible content and closes with Escape", async () => {
    const host = document.createElement("div");
    document.body.append(host);
    const [open, setOpen] = createSignal(true);
    const dispose = render(
      () => (
        <Tooltip.Root open={open()} onOpenChange={setOpen}>
          <Tooltip.Trigger
            asChild={(triggerProps) => (
              <button {...triggerProps()} aria-label="Edit">Edit icon</button>
            )}
          />
          <Tooltip.Content>Edit</Tooltip.Content>
        </Tooltip.Root>
      ),
      host,
    );

    await vi.waitFor(() => {
      expect(document.body.querySelector("[role='tooltip']")?.textContent).toBe("Edit");
    });
    const trigger = host.querySelector("button")!;
    expect(trigger.getAttribute("aria-describedby")).toBe(
      document.body.querySelector("[role='tooltip']")?.id,
    );

    document.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape", bubbles: true }));
    await vi.waitFor(() => expect(open()).toBe(false));

    dispose();
  });

  it("uses Ark hover and focus behavior with the shared positioning defaults", async () => {
    const host = document.createElement("div");
    document.body.append(host);
    const dispose = render(
      () => (
        <Tooltip.Root openDelay={0} closeDelay={0}>
          <Tooltip.Trigger
            asChild={(triggerProps) => (
              <button {...triggerProps()}>Action</button>
            )}
          />
          <Tooltip.Content>Action label</Tooltip.Content>
        </Tooltip.Root>
      ),
      host,
    );

    const trigger = host.querySelector("button")!;
    trigger.focus();
    await vi.waitFor(() => {
      expect(document.body.querySelector("[role='tooltip']")?.textContent).toBe("Action label");
    });

    trigger.blur();
    trigger.dispatchEvent(new MouseEvent("pointerover", { bubbles: true }));
    await vi.waitFor(() => {
      expect(document.body.querySelector("[role='tooltip']")?.textContent).toBe("Action label");
    });

    dispose();
  });

  it("keeps presence styling centralized and reduced-motion safe", () => {
    const css = readFileSync(
      repoPath("packages/frontend/design-system/src/tooltip.css"),
      "utf8",
    );
    const source = readFileSync(
      repoPath("packages/frontend/design-system/src/tooltip.tsx"),
      "utf8",
    );
    expect(css).toContain(".pw-tooltip-content");
    expect(source).toContain("pw-tooltip-content menu-panel");
    expect(css).toContain("@media (prefers-reduced-motion: reduce)");
    expect(css).toContain("animation: none !important");
    expect(source).toContain("const DEFAULT_TOOLTIP_OPEN_DELAY = 400;");
    expect(source).toContain("const DEFAULT_TOOLTIP_CLOSE_DELAY = 100;");
    expect(source).toContain("if (group?.isWarm()) return 0;");
    expect(source).toContain("return props.openDelay ?? DEFAULT_TOOLTIP_OPEN_DELAY;");
    expect(source).toContain("props.closeDelay ?? DEFAULT_TOOLTIP_CLOSE_DELAY");
    expect(source).toContain("onCleanup(clearCooldown)");
  });
});
