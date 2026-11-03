// @vitest-environment jsdom

import { render } from "solid-js/web";
import { afterEach, describe, expect, it, vi } from "vitest";

import { ToolbarIconButton } from "./toolbar-icon-button";

afterEach(() => {
  vi.useRealTimers();
  document.body.replaceChildren();
});

async function advanceTooltipTime(milliseconds: number) {
  await vi.advanceTimersByTimeAsync(milliseconds);
  await Promise.resolve();
}

function dispatchTooltipPointer(element: Element, type: "pointermove" | "pointerleave") {
  const event = new MouseEvent(type, { bubbles: true });
  Object.defineProperty(event, "pointerType", { value: "mouse" });
  element.dispatchEvent(event);
}

describe("ToolbarIconButton", () => {
  it("renders a round primary icon button with aria-label", () => {
    const host = document.createElement("div");
    document.body.append(host);
    const dispose = render(
      () => (
        <ToolbarIconButton label="Add resource">
          <span data-testid="icon">+</span>
        </ToolbarIconButton>
      ),
      host,
    );

    const button = host.querySelector("button");
    expect(button).not.toBeNull();
    expect(button?.getAttribute("aria-label")).toBe("Add resource");
    expect(button?.className).toContain("!rounded-full");
    expect(button?.textContent).toContain("+");
    expect(button?.hasAttribute("title")).toBe(false);

    dispose();
    host.remove();
  });

  it("shows the label in a tooltip after the open delay", async () => {
    vi.useFakeTimers();
    const host = document.createElement("div");
    document.body.append(host);
    const dispose = render(
      () => (
        <ToolbarIconButton label="Add workflow">
          <span>+</span>
        </ToolbarIconButton>
      ),
      host,
    );
    const button = host.querySelector("button");
    expect(button).not.toBeNull();

    dispatchTooltipPointer(button!, "pointermove");
    await advanceTooltipTime(399);
    expect(document.body.textContent).not.toContain("Add workflow");
    await advanceTooltipTime(1);
    expect(document.body.textContent).toContain("Add workflow");

    dispose();
    host.remove();
  });
});
