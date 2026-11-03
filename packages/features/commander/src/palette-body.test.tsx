import { render } from "solid-js/web";
import { afterEach, describe, expect, it, vi } from "vitest";
import { PaletteBody, RUN_AGENT_PREFIX } from "./palette-body";
import type { PaletteItem } from "./palette-items";
import type { ComposerProperty } from "@polterware/frontend-design-system";

const ITEMS: PaletteItem[] = [
  { id: "nav.inbox", label: "Inbox", group: "Navigation" },
  { id: "nav.activity", label: "Activity", group: "Navigation" },
];

let dispose: (() => void) | undefined;
afterEach(() => {
  dispose?.();
  dispose = undefined;
  document.body.innerHTML = "";
});

function mount(runProperties?: ComposerProperty[]) {
  const onCommand = vi.fn();
  const host = document.createElement("div");
  document.body.append(host);
  dispose = render(() => <PaletteBody items={ITEMS} onCommand={onCommand} runAgent runProperties={runProperties} />, host);
  const input = host.querySelector<HTMLInputElement>('input[role="searchbox"]')!;
  const type = (value: string) => {
    input.value = value;
    input.dispatchEvent(new InputEvent("input", { bubbles: true }));
  };
  const press = (init: KeyboardEventInit) => input.dispatchEvent(new KeyboardEvent("keydown", { key: "Enter", bubbles: true, cancelable: true, ...init }));
  return { host, input, type, press, onCommand };
}

describe("PaletteBody Run an agent", () => {
  it("drops the list for a run and offers Start run instead", () => {
    const { host, type, onCommand } = mount();
    expect(host.querySelector('[role="option"]')?.textContent).toContain("Inbox");
    expect(host.querySelector(".commander-run-footer")).toBeNull();
    type("fix the toast");
    expect(host.querySelector('[role="listbox"]')).toBeNull();
    const start = host.querySelector<HTMLButtonElement>(".commander-run-footer button")!;
    expect(start.textContent).toBe("Start run");
    start.click();
    expect(onCommand).toHaveBeenCalledWith(`${RUN_AGENT_PREFIX}fix the toast`);
  });

  it("runs a matching command on Enter", () => {
    const { type, press, onCommand } = mount();
    type("inbox");
    press({});
    expect(onCommand).toHaveBeenCalledWith("nav.inbox");
  });

  it("moves to the details on Enter in a run, like the Composer, and Cmd+Enter starts it with them", () => {
    const { host, type, press, onCommand } = mount();
    type("fix the toast spacing");
    press({});
    expect(onCommand).not.toHaveBeenCalled();
    const details = host.querySelector<HTMLTextAreaElement>('textarea[aria-label="Description"]')!;
    expect(document.activeElement).toBe(details);
    details.value = "Check the inbox toast.";
    details.dispatchEvent(new InputEvent("input", { bubbles: true }));
    details.dispatchEvent(new KeyboardEvent("keydown", { key: "Enter", metaKey: true, bubbles: true, cancelable: true }));
    expect(onCommand).toHaveBeenCalledWith(`${RUN_AGENT_PREFIX}fix the toast spacing\n\nCheck the inbox toast.`);
  });

  it("always runs the agent on Cmd+Enter, and never without a prompt", () => {
    const { type, press, onCommand } = mount();
    press({ metaKey: true });
    expect(onCommand).not.toHaveBeenCalled();
    type("inbox");
    press({ metaKey: true });
    expect(onCommand).toHaveBeenCalledWith(`${RUN_AGENT_PREFIX}inbox`);
  });

  it("opens the run pills only for a run", () => {
    const { host, type } = mount([
      { id: "route", label: "Route", options: [{ value: "tier:normal", label: "Normal" }], value: "tier:normal", onChange: () => {} },
      { id: "card", label: "Card", options: [{ value: "", label: "No card" }], value: "", onChange: () => {}, hidden: true },
    ]);
    const region = () => host.querySelector<HTMLElement>(".commander-run-pills-region")!;
    expect(region().hidden).toBe(true);
    type("inbox");
    expect(region().dataset.state ?? "closed").toBe("closed");
    type("fix the toast spacing");
    expect(region().hidden).toBe(false);
    expect(region().dataset.state).toBe("open");
    const pills = [...region().querySelectorAll('[aria-label="Properties"] button')].map((button) => button.getAttribute("aria-label"));
    expect(pills).toEqual(["Route: Normal"]);
  });
});
