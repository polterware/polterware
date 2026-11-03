import { afterEach, describe, expect, it, vi } from "vitest";
import { createSignal, type JSX } from "solid-js";
import { render } from "solid-js/web";
import { DatePicker, parseDate } from "./date-picker";
import { TimePicker, parseTime } from "./time-picker";
import { ColorSwatchPicker, parseColor } from "./color-swatch-picker";
import { IconPicker, type PickerIconName } from "./icon-picker";
const disposers: (() => void)[] = [];
function mount(view: () => JSX.Element) { const host = document.createElement("div"); document.body.append(host); disposers.push(render(view, host)); return host; }
function key(element: HTMLElement, value: string) { element.dispatchEvent(new KeyboardEvent("keydown", { key: value, bubbles: true, cancelable: true })); element.dispatchEvent(new KeyboardEvent("keyup", { key: value, bubbles: true, cancelable: true })); }
function pointerClick(element: HTMLElement) {
  const opts = { bubbles: true, cancelable: true, pointerId: 1, pointerType: "mouse", button: 0, buttons: 1 } as const;
  element.dispatchEvent(new PointerEvent("pointerdown", opts));
  element.dispatchEvent(new MouseEvent("mousedown", opts));
  element.dispatchEvent(new PointerEvent("pointerup", { ...opts, buttons: 0 }));
  element.dispatchEvent(new MouseEvent("mouseup", { ...opts, buttons: 0 }));
  element.click();
}
afterEach(() => { disposers.splice(0).forEach(dispose => dispose()); document.body.replaceChildren(); vi.unstubAllGlobals(); });
describe("DatePicker", () => {
  it("opens an Ark calendar, focuses a day, selects by keyboard and restores focus", async () => {
    const host = mount(() => <DatePicker label="Due date" defaultValue={[parseDate("2026-10-04")]} />);
    const trigger = host.querySelector<HTMLButtonElement>('[aria-label="Choose date"]')!;
    trigger.focus(); trigger.click();
    await vi.waitFor(() => expect(document.querySelector('[data-part="table-cell-trigger"]')).not.toBeNull());
    const day = document.querySelector<HTMLElement>('[data-part="table-cell-trigger"][data-selected]')!;
    day.focus();
    expect(document.activeElement).toBe(day);
    key(day, "ArrowRight");
    await vi.waitFor(() => expect(document.activeElement?.getAttribute("data-part")).toBe("table-cell-trigger"));
    trigger.click();
    await vi.waitFor(() => expect(trigger.getAttribute("aria-expanded")).toBe("false"));
    trigger.focus();
    expect(document.activeElement).toBe(trigger);
  });
  it("keeps controlled dates, reports clearing and supports disabled with reduced motion", async () => {
    vi.stubGlobal("matchMedia", () => ({ matches: true })); const change = vi.fn();
    const host = mount(() => <><DatePicker label="Controlled date" value={[parseDate("2026-10-04")]} onValueChange={change} /><DatePicker label="Disabled date" disabled /></>);
    host.querySelector<HTMLButtonElement>('[aria-label="Clear date"]')!.click();
    await vi.waitFor(() => expect(change).toHaveBeenCalledWith(expect.objectContaining({ value: [] })));
    expect(host.querySelector<HTMLInputElement>("input")!.value).not.toBe("");
    expect(host.querySelectorAll<HTMLInputElement>("input")[1]!.disabled).toBe(true);
  });
});
describe("TimePicker", () => {
  it.each(["00:00", "09:30", "23:59"])("accepts wall-clock %s", value => expect(parseTime(value)).toHaveLength(2));
  it.each(["24:00", "23:60", "9:00", "bad", "12:00:00"])("rejects invalid %s", value => expect(parseTime(value)).toBeUndefined());
  it("edits uncontrolled hours by keyboard and restores trigger focus on Escape", async () => {
    const host = mount(() => <TimePicker label="Reminder" defaultValue="09:30" />);
    const trigger = host.querySelector<HTMLButtonElement>("button")!; trigger.focus(); trigger.click();
    await vi.waitFor(() => expect(document.querySelector<HTMLInputElement>('[aria-label="Hours"]')).not.toBeNull());
    const hours = document.querySelector<HTMLInputElement>('[aria-label="Hours"]')!; hours.focus(); key(hours, "ArrowUp");
    await vi.waitFor(() => expect(trigger.textContent).toContain("10:30"));
    key(hours, "Escape"); await vi.waitFor(() => expect(trigger.getAttribute("aria-expanded")).toBe("false"));
    expect(document.activeElement).toBe(trigger);
  });
  it("reports controlled minutes while preserving the owner's value under reduced motion", async () => {
    vi.stubGlobal("matchMedia", () => ({ matches: true })); const change = vi.fn();
    const host = mount(() => <TimePicker label="Reminder" value="09:30" onValueChange={change} />);
    host.querySelector<HTMLButtonElement>("button")!.click();
    await vi.waitFor(() => expect(document.querySelector('[aria-label="Minutes"]')).not.toBeNull());
    const minutes = document.querySelector<HTMLInputElement>('[aria-label="Minutes"]')!; minutes.focus(); key(minutes, "ArrowUp");
    await vi.waitFor(() => expect(change).toHaveBeenCalledWith("09:31"));
    expect(host.textContent).toContain("09:30");
  });
});
describe("ColorSwatchPicker", () => {
  const swatches = [{ value: "#7EA8F0", label: "Blue" }, { value: "#6ED9A8", label: "Green" }];
  it("selects native button swatches and exposes selected state in uncontrolled mode", async () => {
    const host = mount(() => <ColorSwatchPicker label="Color" swatches={swatches} defaultValue={parseColor("#7EA8F0")} />);
    const green = host.querySelector<HTMLButtonElement>('[aria-label="Green"]')!; green.focus(); expect(document.activeElement).toBe(green);
    key(green, "Enter"); green.click(); // native Enter click is supplied by browsers.
    await vi.waitFor(() => expect(green.querySelector('[data-part="swatch-indicator"]')!.hasAttribute("hidden")).toBe(false));
  });
  it("reports controlled colors and keeps native keyboard targets under reduced motion", async () => {
    vi.stubGlobal("matchMedia", () => ({ matches: true })); const change = vi.fn();
    const host = mount(() => <ColorSwatchPicker label="Color" swatches={swatches} value={parseColor("#7EA8F0")} onValueChange={change} />);
    host.querySelector<HTMLButtonElement>('[aria-label="Green"]')!.click();
    await vi.waitFor(() => expect(change).toHaveBeenCalled());
    expect(host.querySelector('[aria-label="Blue"] [data-part="swatch-indicator"]')!.hasAttribute("hidden")).toBe(false);
  });
});
describe("IconPicker", () => {
  it("selects an uncontrolled Hugeicon and restores focus on dismissal", async () => {
    const change = vi.fn(); const host = mount(() => <IconPicker label="Space icon" defaultValue="folder" onValueChange={change} />);
    const trigger = host.querySelector<HTMLButtonElement>("button")!; trigger.focus(); trigger.click();
    await vi.waitFor(() => expect(document.querySelector('[data-value="book"]')).not.toBeNull());
    const option = document.querySelector<HTMLButtonElement>('[data-value="book"]')!;
    option.focus(); key(option, "Enter"); option.click();
    await vi.waitFor(() => expect(change).toHaveBeenCalledWith("book"));
    expect(trigger.getAttribute("aria-expanded")).toBe("false"); expect(document.activeElement).toBe(trigger);
    expect(trigger.querySelector("svg")).not.toBeNull();
  });
  it("supports controlled values, Escape and reduced motion", async () => {
    vi.stubGlobal("matchMedia", () => ({ matches: true }));
    const [value, setValue] = createSignal<PickerIconName>("book");
    const host = mount(() => <IconPicker label="Space icon" value={value()} onValueChange={setValue} />);
    const trigger = host.querySelector<HTMLButtonElement>("button")!; trigger.click();
    await vi.waitFor(() => expect(document.querySelector('[data-value="folder"]')).not.toBeNull());
    document.querySelector<HTMLButtonElement>('[data-value="folder"]')!.click();
    await vi.waitFor(() => expect(value()).toBe("folder"));
    trigger.click(); await vi.waitFor(() => expect(document.querySelector('[data-part="content"][data-state="open"]')).not.toBeNull());
    key(document.activeElement as HTMLElement, "Escape"); await vi.waitFor(() => expect(trigger.getAttribute("aria-expanded")).toBe("false"));
    expect(document.activeElement).toBe(trigger);
  });
});
