import { afterEach, describe, expect, it, vi } from "vitest";
import { createSignal, type JSX } from "solid-js";
import { render } from "solid-js/web";
import { readFileSync } from "node:fs";
import { TreeView } from "./tree-view";
import { ProgressRing } from "./progress-ring";
import { Heatmap, heatmapLevel } from "./heatmap";
import { CalendarGrid } from "./calendar-grid";
import { PageEmpty, EmptyIllustration } from "./feedback";
const disposers: (() => void)[] = [];
function mount(view: () => JSX.Element) { const host = document.createElement("div"); document.body.append(host); disposers.push(render(view, host)); return host; }
function key(element: HTMLElement, value: string) { element.dispatchEvent(new KeyboardEvent("keydown", { key: value, bubbles: true, cancelable: true })); }
afterEach(() => { disposers.splice(0).forEach(dispose => dispose()); document.body.replaceChildren(); vi.unstubAllGlobals(); });
describe("TreeView", () => {
  const nodes = [{ id: "work", label: "Work", children: [{ id: "ideas", label: "Ideas" }] }, { id: "personal", label: "Personal" }];
  it("expands, roves focus and selects by keyboard in uncontrolled mode", async () => {
    const host = mount(() => <TreeView label="Minds" nodes={nodes} />);
    const work = host.querySelector<HTMLElement>('[data-part="branch-control"]')!; work.focus(); key(work, "ArrowRight");
    await vi.waitFor(() => expect(work.getAttribute("aria-expanded")).toBe("true"));
    key(work, "ArrowDown"); await vi.waitFor(() => expect(document.activeElement?.textContent).toBe("Ideas"));
    key(document.activeElement as HTMLElement, "Enter");
    await vi.waitFor(() => expect(document.activeElement?.getAttribute("aria-selected")).toBe("true"));
  });
  it("reports controlled selection and retains hierarchy with reduced motion", async () => {
    vi.stubGlobal("matchMedia", () => ({ matches: true })); const change = vi.fn();
    const host = mount(() => <TreeView label="Minds" nodes={nodes} selectedValue={["personal"]} onSelectionChange={change} defaultExpandedValue={["work"]} />);
    const ideas = Array.from(host.querySelectorAll<HTMLElement>('[data-part="item"]')).find(item => item.textContent === "Ideas")!;
    ideas.click(); await vi.waitFor(() => expect(change).toHaveBeenCalledWith(expect.objectContaining({ selectedValue: ["ideas"] })));
    expect(ideas.getAttribute("aria-selected")).toBe("false");
  });
});
describe("ProgressRing", () => {
  it("exposes reactive bounded progress and has no keyboard/focus target", () => {
    const [value, setValue] = createSignal(25);
    const host = mount(() => <ProgressRing label="Focus" value={value()} size={80} />);
    const progress = host.querySelector('[role="progressbar"]')!;
    expect(progress.getAttribute("aria-valuenow")).toBe("25"); setValue(75);
    expect(progress.getAttribute("aria-valuenow")).toBe("75"); expect(host.querySelector("button, input, [tabindex]")).toBeNull();
    expect(host.querySelector("svg")!.style.getPropertyValue("--size")).toBe("80px");
  });
  it("supports a default value and declares a reduced-motion snap without opacity", () => {
    vi.stubGlobal("matchMedia", () => ({ matches: true }));
    const host = mount(() => <ProgressRing label="Focus" defaultValue={50} />);
    expect(host.querySelector('[role="progressbar"]')!.getAttribute("aria-valuenow")).toBe("50");
    const css = readFileSync(new URL("./progress-ring.css", import.meta.url), "utf8");
    expect(css).toMatch(/prefers-reduced-motion: reduce[\s\S]*transition: none/); expect(css).not.toMatch(/opacity/);
  });
});
describe("Heatmap", () => {
  it.each([[0, 0], [1, 1], [3, 2], [7, 3], [14, 4], [500, 4], [-1, 0], [NaN, 0]])("maps count %s to opaque level %s", (count, level) => expect(heatmapLevel(count)).toBe(level));
  it("updates supplied days and labels without interactive or animated cells", () => {
    vi.stubGlobal("matchMedia", () => ({ matches: true }));
    const [value, setValue] = createSignal(1);
    const host = mount(() => <Heatmap label="Contributions" days={[{ date: "2026-10-04", value: value() }]} />);
    expect(host.querySelector('[role="img"]')!.getAttribute("data-level")).toBe("1"); setValue(14);
    expect(host.querySelector('[role="img"]')!.getAttribute("aria-label")).toContain("14");
    expect(host.querySelector("button, input, [tabindex]")).toBeNull();
  });
});
describe("CalendarGrid", () => {
  it.each(["day", "week", "month"] as const)("renders %s slots with semantic headers and opaque occurrences", view => {
    const action = vi.fn(); const host = mount(() => <CalendarGrid.Root view={view} label="Calendar"><CalendarGrid.Days><CalendarGrid.Day label="Oct 4" today><CalendarGrid.Header>Oct 4</CalendarGrid.Header><CalendarGrid.AllDay><CalendarGrid.Occurrence onClick={action}>Read</CalendarGrid.Occurrence></CalendarGrid.AllDay><CalendarGrid.HourRow label="09:00" /><CalendarGrid.Overflow count={3} /></CalendarGrid.Day></CalendarGrid.Days></CalendarGrid.Root>);
    expect(host.querySelector("section")!.getAttribute("data-view")).toBe(view);
    const button = host.querySelector<HTMLButtonElement>("button")!; button.focus(); expect(document.activeElement).toBe(button); key(button, "Enter"); button.click(); expect(action).toHaveBeenCalledOnce();
    expect(host.querySelector('[aria-label="3 more occurrences"]')).not.toBeNull();
  });
  it("updates controlled presentation and respects disabled occurrences in reduced motion", () => {
    vi.stubGlobal("matchMedia", () => ({ matches: true })); const [completed, setCompleted] = createSignal(false);
    const host = mount(() => <CalendarGrid.Root view="day" label="Calendar"><CalendarGrid.Occurrence completed={completed()} disabled>Read</CalendarGrid.Occurrence></CalendarGrid.Root>);
    const button = host.querySelector<HTMLButtonElement>("button")!; expect(button.disabled).toBe(true); setCompleted(true); expect(button.hasAttribute("data-completed")).toBe(true);
  });
});
describe("EmptyIllustration", () => {
  it("keeps decorative artwork out of keyboard/focus navigation while preserving the empty action", () => {
    vi.stubGlobal("matchMedia", () => ({ matches: true })); const action = vi.fn();
    const host = mount(() => <PageEmpty title="Empty" illustration={<EmptyIllustration>Artwork</EmptyIllustration>} action={<button onClick={action}>Create</button>} />);
    expect(host.querySelector('[aria-hidden="true"]')!.textContent).toBe("Artwork"); const button = host.querySelector<HTMLButtonElement>("button")!; button.focus(); key(button, "Enter"); button.click(); expect(document.activeElement).toBe(button); expect(action).toHaveBeenCalledOnce();
  });
});
