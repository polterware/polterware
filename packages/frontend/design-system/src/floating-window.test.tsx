// @vitest-environment jsdom

import { createSignal, type JSX } from "solid-js";
import { render } from "solid-js/web";
import { readFileSync } from "node:fs";
import { afterEach, describe, expect, it, vi } from "vitest";

import {
  FLOATING_WINDOW_DEFAULT_WIDTH,
  FLOATING_WINDOW_MAX_WIDTH,
  FLOATING_WINDOW_MIN_WIDTH,
  FloatingWindow,
  clampFloatingWindowWidth,
} from "./floating-window";
import { readFloatingWindowGeometry, resolveFloatingWindowRect, defaultFloatingWindowGeometry, moveFloatingWindowRect, resizeFloatingWindowRect, FLOATING_WINDOW_DOCKS, nearestFloatingWindowDock, windowGeometryAt } from "./floating-window-geometry";
import { A11yDialog } from "./dialog";
import { DropdownMenu, DropdownMenuTrigger, DropdownMenuContent, DropdownMenuItem } from "./dropdown-menu";
import { repoPath } from "../test/repo-path";

const css = () =>
  readFileSync(repoPath("packages/frontend/design-system/src/floating-window.css"), "utf8");
const source = () =>
  readFileSync(repoPath("packages/frontend/design-system/src/floating-window.tsx"), "utf8");

type Harness = {
  host: HTMLDivElement;
  root: () => HTMLElement;
  sidebar: () => HTMLElement;
  open: () => boolean;
  setOpen: (open: boolean) => void;
  expanded: () => boolean;
  setExpanded: (expanded: boolean) => void;
  positioner: () => HTMLDivElement;
  resize: (width: number, height: number) => void;
  dispose: () => void;
};

function mount(options: {
  open?: boolean;
  expanded?: boolean;
  minimized?: boolean;
  available?: boolean;
  storageKey?: string;
  content?: () => JSX.Element;
} = {}): Harness {
  let dimensions = { width: 1200, height: 800 };
  vi.spyOn(HTMLElement.prototype, "getBoundingClientRect").mockImplementation(function (this: HTMLElement) {
    return this.classList.contains("pw-floating-window-root") ? new DOMRect(0, 0, dimensions.width, dimensions.height) : new DOMRect();
  });
  const host = document.createElement("div");
  document.body.append(host);
  const [open, setOpen] = createSignal(options.open ?? true);
  const [minimized, setMinimized] = createSignal(options.minimized ?? false);
  const [expanded, setExpanded] = createSignal(options.expanded ?? false);
  const dispose = render(
    () => (
      <FloatingWindow.Root
        open={open()}
        expanded={expanded()}
        minimized={minimized()}
        onMinimizedChange={setMinimized}
        available={options.available ?? true}
        onOpenChange={(next) => { setMinimized(false); setOpen(next); if (!next) setExpanded(false); }}
        onExpandedChange={setExpanded}
        storageKey={options.storageKey}
      >
        <FloatingWindow.Workspace>
          <button data-floating-window-opener onClick={() => { setMinimized(false); setOpen(true); }}>Alpha row</button>
        </FloatingWindow.Workspace>
        <FloatingWindow.Content>
          {options.content?.() ?? (
            <>
              <FloatingWindow.Header />
              <div>
                <button type="button">Inside window</button>
              </div>
            </>
          )}
        </FloatingWindow.Content>
      </FloatingWindow.Root>
    ),
    host,
  );
  return {
    host,
    root: () => host.querySelector<HTMLElement>(".pw-floating-window-root")!,
    sidebar: () => host.querySelector<HTMLElement>(".pw-floating-window__frame")!,
    open,
    setOpen,
    expanded,
    setExpanded,
    positioner: () => host.querySelector<HTMLDivElement>(".pw-floating-window__positioner")!,
    resize: (width, height) => { dimensions = { width, height }; window.dispatchEvent(new Event("resize")); },
    dispose: () => {
      dispose();
      host.remove();
    },
  };
}

function press(init: KeyboardEventInit, target: EventTarget = window) {
  target.dispatchEvent(new KeyboardEvent("keydown", { bubbles: true, ...init }));
  target.dispatchEvent(new KeyboardEvent("keyup", { bubbles: true, ...init }));
}

function pointer(target: HTMLElement, type: string, x: number, y: number) {
  target.setPointerCapture ??= vi.fn();
  target.hasPointerCapture ??= () => true;
  target.releasePointerCapture ??= vi.fn();
  const event = new MouseEvent(type, { bubbles: true, cancelable: true, clientX: x, clientY: y, button: 0 });
  Object.defineProperties(event, { pointerId: { value: 1 }, pointerType: { value: "mouse" } });
  target.dispatchEvent(event);
}

afterEach(() => {
  vi.restoreAllMocks();
  vi.unstubAllGlobals();
  window.localStorage.clear();
  document.body.replaceChildren();
});

describe("floating detail geometry", () => {
  it("clamps width against the surface without reserving collection space", () => {
    expect(clampFloatingWindowWidth(100, 0)).toBe(FLOATING_WINDOW_MIN_WIDTH);
    expect(clampFloatingWindowWidth(5000, 0)).toBe(FLOATING_WINDOW_MAX_WIDTH);
    expect(clampFloatingWindowWidth(700, 900)).toBe(700);
    expect(clampFloatingWindowWidth(500, 500)).toBe(468);
    expect(clampFloatingWindowWidth(500, 200)).toBe(168);
    expect(clampFloatingWindowWidth(Number.NaN, 0)).toBe(FLOATING_WINDOW_DEFAULT_WIDTH);
  });
  it("validates stored geometry and migrates a numeric width", () => {
    expect(readFloatingWindowGeometry("480")).toEqual({ ...defaultFloatingWindowGeometry, width: 480 });
    for (const raw of [null, "bad", "{}", '{"version":2}', '{"version":1,"width":-4}']) {
      expect(readFloatingWindowGeometry(raw)).toEqual(defaultFloatingWindowGeometry);
    }
  });
  it("resolves all eight docks for card and circle and preserves ties", () => {
    const bounds = { width: 1200, height: 800 };
    const expected = [[16,16],[340,16],[664,16],[16,190],[664,190],[16,364],[340,364],[664,364]];
    for (const [index, dock] of FLOATING_WINDOW_DOCKS.entries()) {
      const card = resolveFloatingWindowRect({ ...defaultFloatingWindowGeometry, dock }, bounds);
      expect([card.x, card.y]).toEqual(expected[index]);
      expect(nearestFloatingWindowDock(card, bounds, "bottom-right")).toBe(dock);
      const circle = resolveFloatingWindowRect({ ...defaultFloatingWindowGeometry, dock }, bounds, true);
      expect(nearestFloatingWindowDock(circle, bounds, "bottom-right")).toBe(dock);
      const resized = windowGeometryAt({ ...card, width: 600, height: 500 }, bounds, { ...defaultFloatingWindowGeometry, dock }, true);
      expect(resized).toMatchObject({ dock, width: 600, height: 500 });
    }
    const center = { x: 340, y: 190, width: 520, height: 420 };
    expect(nearestFloatingWindowDock(center, bounds, "bottom")).toBe("bottom");
    expect(nearestFloatingWindowDock(center, bounds, "top")).toBe("top");
    expect(readFloatingWindowGeometry(JSON.stringify({ version: 1, anchorX: 0, anchorY: 0, width: 480, height: 390 }), bounds))
      .toEqual({ version: 2, dock: "top-left", width: 480, height: 390 });
    expect(readFloatingWindowGeometry(JSON.stringify({ version: 1, anchorX: 1, anchorY: 1, width: 480, height: 390 }), bounds))
      .toEqual({ version: 2, dock: "bottom-right", width: 480, height: 390 });
  });
  it("fits small surfaces and constrains moves and all eight resize directions", () => {
    const bounds = { width: 1200, height: 800 };
    const rect = resolveFloatingWindowRect(defaultFloatingWindowGeometry, bounds);
    expect(rect).toEqual({ x: 664, y: 364, width: 520, height: 420 });
    expect(resolveFloatingWindowRect(defaultFloatingWindowGeometry, bounds, true)).toEqual({ x: 1136, y: 736, width: 48, height: 48 });
    expect(moveFloatingWindowRect(rect, -5000, -5000, bounds)).toEqual({ ...rect, x: 16, y: 16 });
    for (const edge of ["n", "s", "e", "w", "ne", "nw", "se", "sw"] as const) {
      for (const delta of [-5000, 5000]) {
        const box = resizeFloatingWindowRect(rect, edge, delta, delta, bounds);
        expect(box.x).toBeGreaterThanOrEqual(16);
        expect(box.y).toBeGreaterThanOrEqual(16);
        expect(box.x + box.width).toBeLessThanOrEqual(1184);
        expect(box.y + box.height).toBeLessThanOrEqual(784);
        expect(box.width).toBeGreaterThanOrEqual(320);
        expect(box.height).toBeGreaterThanOrEqual(280);
      }
    }
    expect(resolveFloatingWindowRect(defaultFloatingWindowGeometry, { width: 250, height: 200 })).toEqual({ x: 16, y: 16, width: 218, height: 168 });
    expect(resolveFloatingWindowRect(defaultFloatingWindowGeometry, { width: 30, height: 25 }, true)).toEqual({ x: 5, y: 0, width: 25, height: 25 });
  });
});

describe("FloatingWindow", () => {

  it("starts at the bottom right and leaves the collection at full width", async () => {
    const panel = mount();
    await vi.waitFor(() => expect(panel.positioner().style.transform).toBe("translate3d(664px, 364px, 0)"));
    expect(panel.root().style.getPropertyValue("--floating-window-width")).toBe("520px");
    expect(panel.root().style.getPropertyValue("--floating-window-height")).toBe("420px");
    expect(css()).not.toContain("grid-template-columns:\n    minmax(0, 1fr)");
    expect(css()).not.toContain("pw-floating-window__spacer");
    expect(panel.host.querySelectorAll(".pw-floating-window__resize-handle")).toHaveLength(8);
    panel.dispose();
  });

  it("starts completely closed and exposes no circle until explicit activation", () => {
    const panel = mount({ open: false });
    expect(panel.root().dataset.state).toBe("closed");
    expect(panel.positioner().hidden).toBe(true);
    panel.host.querySelector<HTMLButtonElement>("[data-floating-window-opener]")!.click();
    expect(panel.root().dataset.state).toBe("open");
    expect(panel.positioner().hidden).toBe(false);
    panel.dispose();
  });

  it("shows Apple-style close, minimize and expand controls in both open modes", () => {
    const panel = mount();
    expect(panel.host.querySelector('[aria-label="Close window"]')).not.toBeNull();
    const yellow = panel.host.querySelector('[aria-label="Minimize window"]');
    expect(yellow).not.toBeNull();
    panel.setExpanded(true);
    expect(panel.host.querySelector('[aria-label="Close window"]')).not.toBeNull();
    expect(panel.host.querySelector('[aria-label="Minimize window"]')).toBe(yellow);
    expect(panel.host.querySelector('[aria-label="Restore window"]')).not.toBeNull();
    expect(panel.host.querySelector('[aria-label="Move window"]')?.getAttribute("aria-disabled")).toBe("true");
    panel.dispose();
  });

  it("red closes completely and immediately disables input before the exit finishes", async () => {
    const panel = mount();
    await new Promise<void>((resolve) => requestAnimationFrame(() => resolve()));
    panel.host.querySelector<HTMLButtonElement>('[aria-label="Close window"]')!.click();
    expect(panel.root().dataset.state).toBe("closed");
    expect(panel.sidebar().hasAttribute("inert")).toBe(true);
    expect(panel.positioner().hasAttribute("inert")).toBe(true);
    expect(panel.host.querySelector<HTMLButtonElement>("[data-floating-window-toggle]")!.disabled).toBe(true);
    await vi.waitFor(() => expect(panel.positioner().hidden).toBe(true));
    expect(source()).toContain("spring.fast.exit.duration * 1000");
    panel.dispose();
  });

  it("outside activation preserves its action and focus; opener and card clicks stay open", async () => {
    const panel = mount();
    panel.host.querySelector<HTMLButtonElement>("[data-floating-window-opener]")!.click();
    expect(panel.open()).toBe(true);
    const inside = panel.host.querySelector<HTMLButtonElement>(".pw-floating-window__content button")!;
    inside.focus(); inside.click();
    expect(panel.open()).toBe(true);
    const outside = document.createElement("button");
    const action = vi.fn(); outside.onclick = action; document.body.append(outside);
    pointer(outside, "pointerdown", 0, 0); outside.focus(); outside.click();
    expect(panel.open()).toBe(false);
    expect(action).toHaveBeenCalledOnce();
    await Promise.resolve();
    expect(document.activeElement).toBe(outside);
    panel.dispose();
  });

  it("registers callback-owned sibling confirmations", async () => {
    const [confirm, setConfirm] = createSignal(false);
    const panel = mount({ content: () => <button onClick={() => setConfirm(true)}>Confirm action</button> });
    const dialogHost = document.createElement("div"); document.body.append(dialogHost);
    const disposeDialog = render(() => <A11yDialog open={confirm()} title="Confirm action" onClose={() => setConfirm(false)}><button>Keep editing</button></A11yDialog>, dialogHost);
    panel.host.querySelector<HTMLButtonElement>(".pw-floating-window__content button")!.click();
    await vi.waitFor(() => expect(document.querySelector(".pw-dialog-content")).not.toBeNull());
    const dialogButton = document.querySelector<HTMLButtonElement>(".pw-dialog-content button:last-child")!;
    pointer(dialogButton, "pointerdown", 0, 0); dialogButton.click();
    expect(panel.open()).toBe(true);
    setConfirm(false);
    disposeDialog(); dialogHost.remove(); panel.dispose();
  });

  it("keeps an owned portalled menu interactive while unrelated dialogs dismiss window", async () => {
    const [menuOpen, setMenuOpen] = createSignal(false);
    const action = vi.fn();
    const panel = mount({ content: () => (
      <DropdownMenu open={menuOpen()} onOpenChange={setMenuOpen}>
        <DropdownMenuTrigger aria-label="Window options">Options</DropdownMenuTrigger>
        <DropdownMenuContent><DropdownMenuItem value="edit" onSelect={action}>Edit window</DropdownMenuItem></DropdownMenuContent>
      </DropdownMenu>
    ) });
    const trigger = panel.host.querySelector<HTMLButtonElement>('[aria-label="Window options"]')!;
    trigger.focus(); trigger.click();
    await vi.waitFor(() => expect(document.querySelector('[role="menuitem"]')).not.toBeNull());
    const item = document.querySelector<HTMLElement>('[role="menuitem"]')!;
    pointer(item, "pointerdown", 0, 0); item.click();
    expect(panel.open()).toBe(true);
    expect(action).toHaveBeenCalledOnce();
    const outside = document.createElement("button"); document.body.append(outside);
    outside.focus();
    const [unrelated, setUnrelated] = createSignal(false);
    const dialogHost = document.createElement("div"); document.body.append(dialogHost);
    const disposeDialog = render(() => <A11yDialog open={unrelated()} title="Unrelated" onClose={() => setUnrelated(false)}><button>External action</button></A11yDialog>, dialogHost);
    setUnrelated(true);
    await vi.waitFor(() => expect(document.querySelector(".pw-dialog-content")).not.toBeNull());
    pointer(document.querySelector<HTMLElement>(".pw-dialog-content")!, "pointerdown", 0, 0);
    expect(panel.open()).toBe(false);
    disposeDialog(); dialogHost.remove(); panel.dispose();
  });

  it("can interrupt a close by explicitly opening another item", async () => {
    const panel = mount();
    await new Promise<void>((resolve) => requestAnimationFrame(() => resolve()));
    panel.host.querySelector<HTMLButtonElement>('[aria-label="Close window"]')!.click();
    panel.host.querySelector<HTMLButtonElement>("[data-floating-window-opener]")!.click();
    await vi.waitFor(() => expect(panel.positioner().style.opacity).toBe("1"));
    expect(panel.root().dataset.state).toBe("open");
    expect(panel.positioner().hidden).toBe(false);
    expect(panel.sidebar().hasAttribute("inert")).toBe(false);
    panel.dispose();
  });

  it("minimizes to a draggable circle and keeps the same focused toggle", async () => {
    const panel = mount();
    const toggle = panel.host.querySelector<HTMLButtonElement>("[data-floating-window-toggle]")!;
    toggle.focus();
    toggle.click();
    await vi.waitFor(() => expect(panel.positioner().style.transform).toBe("translate3d(1136px, 736px, 0)"));
    expect(panel.open()).toBe(false);
    expect(panel.host.querySelector("[data-floating-window-toggle]")).toBe(toggle);
    expect(document.activeElement).toBe(toggle);
    expect(toggle.dataset.bubble).toBe("true");
    expect(panel.sidebar().hasAttribute("inert")).toBe(true);
    expect(panel.host.querySelector(".pw-floating-window__resize-handle")).toBeNull();
    panel.dispose();
  });

  it("hides card and circle on unavailable surfaces without losing the preference", async () => {
    const panel = mount({ available: false });
    expect(panel.positioner().hidden).toBe(true);
    expect(panel.host.querySelector<HTMLButtonElement>("[data-floating-window-toggle]")?.disabled).toBe(true);
    press({ key: "j", metaKey: true });
    expect(panel.open()).toBe(true);
    panel.dispose();
  });

  it("resizes both dimensions with keyboard edge/corner controls and persists JSON", async () => {
    const storageKey = "test.detail.width";
    window.localStorage.setItem(storageKey, "480");
    const panel = mount({ storageKey });
    const west = () => panel.host.querySelector<HTMLElement>('[data-edge="w"]')!;
    const north = () => panel.host.querySelector<HTMLElement>('[data-edge="n"]')!;
    await vi.waitFor(() => expect(west().getAttribute("aria-valuenow")).toBe("480"));
    press({ key: "ArrowLeft" }, west());
    press({ key: "ArrowUp", shiftKey: true }, north());
    const stored = readFloatingWindowGeometry(window.localStorage.getItem(storageKey));
    expect(stored.width).toBe(496);
    expect(stored.height).toBe(484);
    expect(stored.version).toBe(2);
    panel.dispose();
    const reopened = mount({ storageKey });
    expect(reopened.root().style.getPropertyValue("--floating-window-width")).toBe("496px");
    expect(reopened.root().style.getPropertyValue("--floating-window-height")).toBe("484px");
    reopened.dispose();
  });

  it("moves by keyboard and fits temporary viewport changes without rewriting storage", () => {
    const storageKey = "test.detail.geometry";
    const panel = mount({ storageKey });
    const move = panel.host.querySelector<HTMLElement>('[aria-label="Move window"]')!;
    for (let i = 0; i < 10; i++) move.dispatchEvent(new KeyboardEvent("keydown", { bubbles: true, key: "ArrowLeft", shiftKey: true }));
    window.dispatchEvent(new KeyboardEvent("keyup", { key: "ArrowLeft" }));
    const saved = window.localStorage.getItem(storageKey);
    expect(readFloatingWindowGeometry(saved).dock).toBe("bottom-left");
    panel.resize(250, 200);
    expect(panel.root().style.getPropertyValue("--floating-window-width")).toBe("218px");
    expect(panel.root().style.getPropertyValue("--floating-window-height")).toBe("168px");
    expect(window.localStorage.getItem(storageKey)).toBe(saved);
    panel.resize(1200, 800);
    expect(panel.root().style.getPropertyValue("--floating-window-width")).toBe("520px");
    expect(window.localStorage.getItem(storageKey)).toBe(saved);
    panel.dispose();
  });

  it("docks the dragged circle on reopening and swallows the release click", async () => {
    const storageKey = "test.detail.geometry";
    const panel = mount({ open: false, minimized: true, storageKey });
    const toggle = panel.host.querySelector<HTMLButtonElement>("[data-floating-window-toggle]")!;
    pointer(toggle, "pointerdown", 1140, 740);
    pointer(toggle, "pointermove", 940, 540);
    pointer(toggle, "pointerup", 940, 540);
    toggle.click();
    expect(panel.open()).toBe(false);
    expect(readFloatingWindowGeometry(window.localStorage.getItem(storageKey)).dock).toBe("right");
    // An explicit activation reopens beside the circle's dock after a real drag.
    panel.host.querySelector<HTMLButtonElement>("[data-floating-window-opener]")!.click();
    await vi.waitFor(() => expect(panel.positioner().style.transform).toBe("translate3d(664px, 190px, 0)"));
    expect(readFloatingWindowGeometry(window.localStorage.getItem(storageKey)).width).toBe(520);
    panel.dispose();
  });

  it("rolls back cancelled move and resize gestures and does not persist previews", () => {
    const storageKey = "test.detail.geometry";
    const panel = mount({ storageKey });
    const original = panel.positioner().style.transform;
    const move = panel.host.querySelector<HTMLElement>('[aria-label="Move window"]')!;
    pointer(move, "pointerdown", 800, 380);
    pointer(move, "pointermove", 500, 300);
    pointer(move, "pointercancel", 500, 300);
    expect(panel.positioner().style.transform).toBe(original);
    expect(window.localStorage.getItem(storageKey)).toBeNull();
    const corner = panel.host.querySelector<HTMLElement>('[data-edge="nw"]')!;
    pointer(corner, "pointerdown", 670, 370);
    pointer(corner, "pointermove", 570, 270);
    pointer(corner, "lostpointercapture", 570, 270);
    expect(panel.root().style.getPropertyValue("--floating-window-width")).toBe("520px");
    expect(panel.root().style.getPropertyValue("--floating-window-height")).toBe("420px");
    expect(window.localStorage.getItem(storageKey)).toBeNull();
    panel.dispose();
  });

  it("cancels keyboard movement on Escape or window blur without saving a preview", () => {
    const storageKey = "test.keyboard.cancel";
    const panel = mount({ storageKey });
    const handle = panel.host.querySelector<HTMLElement>('[aria-label="Move window"]')!;
    const original = panel.positioner().style.transform;
    handle.dispatchEvent(new KeyboardEvent("keydown", { bubbles: true, key: "ArrowLeft", shiftKey: true }));
    press({ key: "Escape" });
    expect(panel.positioner().style.transform).toBe(original);
    expect(window.localStorage.getItem(storageKey)).toBeNull();
    handle.dispatchEvent(new KeyboardEvent("keydown", { bubbles: true, key: "ArrowUp", shiftKey: true }));
    window.dispatchEvent(new Event("blur"));
    expect(panel.root().dataset.gesturing).toBeUndefined();
    expect(window.localStorage.getItem(storageKey)).toBeNull();
    panel.dispose();
  });

  it("distinguishes a click from a drag with the dead zone", () => {
    const panel = mount({ open: false, minimized: true });
    const toggle = panel.host.querySelector<HTMLButtonElement>("[data-floating-window-toggle]")!;
    pointer(toggle, "pointerdown", 1140, 740);
    pointer(toggle, "pointermove", 1141, 741);
    pointer(toggle, "pointerup", 1141, 741);
    toggle.click();
    expect(panel.open()).toBe(true);
    panel.dispose();
  });

  it("retains controls without a feature header and restores the floating box after expansion", () => {
    const panel = mount({ content: () => <div>No selection</div> });
    expect(panel.host.querySelector('[aria-label="Move window"]')).not.toBeNull();
    const original = panel.positioner().style.transform;
    panel.setExpanded(true);
    expect(panel.host.querySelector(".pw-floating-window__resize-handle")).toBeNull();
    expect(panel.host.querySelector(".pw-floating-window__collection")?.hasAttribute("inert")).toBe(true);
    panel.setExpanded(false);
    expect(panel.positioner().style.transform).toBe(original);
    expect(panel.root().style.getPropertyValue("--floating-window-width")).toBe("520px");
    panel.dispose();
  });

  it("minimizes an expanded card at its saved dock", () => {
    const panel = mount({ expanded: true });
    const toggle = panel.host.querySelector<HTMLButtonElement>("[data-floating-window-toggle]")!;
    toggle.click();
    expect(panel.expanded()).toBe(false);
    expect(panel.positioner().style.transform).toBe("translate3d(1136px, 736px, 0)");
    toggle.click();
    expect(panel.positioner().style.transform).toBe("translate3d(664px, 364px, 0)");
    panel.dispose();
  });

  it("keeps limit keys and double-click size reset available", () => {
    const panel = mount();
    const west = panel.host.querySelector<HTMLElement>('[data-edge="w"]')!;
    press({ key: "Home" }, west);
    expect(west.getAttribute("aria-valuenow")).toBe("320");
    press({ key: "End" }, west);
    expect(west.getAttribute("aria-valuenow")).toBe("1168");
    west.dispatchEvent(new MouseEvent("dblclick", { bubbles: true }));
    expect(west.getAttribute("aria-valuenow")).toBe("520");
    panel.dispose();
  });

  it("keeps input isolated from the underlying collection and uses projected motion", () => {
    const code = source();
    expect(code).toContain("createDragMotion(dragLayer)");
    expect(code).toContain("setPointerCapture(event.pointerId)");
    expect(code).toContain("const DEAD_ZONE = 3");
    expect(code).toContain('handle.removeEventListener("lostpointercapture", onCancel)');
    expect(code).toContain("event.stopPropagation()");
    expect(code).toContain("shell.style.transform = `scale(");
    expect(code).toContain("frame.style.clipPath");
    expect(code).toContain("spring.slow.exit");
    expect(code).toContain("prefersReducedMotion()");
    expect(code).not.toContain("collectionRight");
    expect(code).not.toContain("toggleProgress");
    expect(readFileSync(repoPath("packages/frontend/design-system/src/page.tsx"), "utf8")).not.toContain("FloatingWindowToggleSlot");
  });

  it("hides the handle and makes the collection inert while expanded", async () => {
    const panel = mount({ expanded: true });
    await vi.waitFor(() => expect(panel.root().dataset.expanded).toBe("true"));
    expect(panel.host.querySelector("[role='separator']")).toBeNull();
    expect(panel.host.querySelector(".pw-floating-window__collection")?.hasAttribute("inert")).toBe(true);
    panel.dispose();
  });

  it("does not intercept entity inspector shortcuts", () => {
    const panel = mount({ open: false });
    press({ key: "j", metaKey: true });
    press({ key: "j", metaKey: true, shiftKey: true });
    expect(panel.open()).toBe(false);
    expect(panel.expanded()).toBe(false);
    expect(source()).not.toContain("@polterware/frontend-state");
    panel.dispose();
  });
  it("exports generic slots and assigns independent control IDs", () => {
    expect(Object.keys(FloatingWindow)).toEqual(["Root", "Workspace", "Header", "Content"]);
    const first = mount(), second = mount();
    expect(first.sidebar().id).not.toBe(second.sidebar().id);
    for (const panel of [first, second]) {
      expect(panel.host.querySelector("[data-floating-window-toggle]")?.getAttribute("aria-controls")).toBe(panel.sidebar().id);
    }
    second.dispose(); first.dispose();
  });
  it("closes a minimized window completely and keeps its saved geometry", () => {
    const key = "test.window";
    const saved = JSON.stringify({ ...defaultFloatingWindowGeometry, dock: "top", width: 480 });
    localStorage.setItem(key, saved);
    const panel = mount({ open: false, minimized: true, storageKey: key });
    const outside = document.createElement("button"); document.body.append(outside);
    pointer(outside, "pointerdown", 0, 0);
    expect(panel.root().dataset.state).toBe("closed");
    expect(panel.positioner().hasAttribute("inert")).toBe(true);
    expect(localStorage.getItem(key)).toBe(saved);
    panel.dispose();
  });
  it("restores expansion and returns focus to its opener without entity selection", async () => {
    const panel = mount({ open: false });
    const opener = panel.host.querySelector<HTMLButtonElement>("[data-floating-window-opener]")!;
    opener.focus(); opener.click();
    const inside = panel.host.querySelector<HTMLButtonElement>(".pw-floating-window__content button")!;
    inside.focus(); panel.setExpanded(true);
    press({ key: "Escape" }); expect(panel.expanded()).toBe(false);
    inside.focus(); press({ key: "Escape" });
    await Promise.resolve();
    expect(document.activeElement).toBe(opener);
    panel.dispose();
  });

});

describe("FloatingWindow toggle", () => {

  it("falls back when storage is inaccessible", () => {
    vi.spyOn(Storage.prototype, "getItem").mockImplementation(() => { throw new Error("blocked"); });
    const panel = mount({ storageKey: "test.detail" });
    expect(panel.root().style.getPropertyValue("--floating-window-width")).toBe("520px");
    panel.dispose();
  });
});
