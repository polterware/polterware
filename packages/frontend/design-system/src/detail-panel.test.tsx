// @vitest-environment jsdom

import { createSignal } from "solid-js";
import { render } from "solid-js/web";
import { readFileSync } from "node:fs";
import { afterEach, describe, expect, it, vi } from "vitest";

import {
  DetailPanel,
  parseTranslate,
  type DetailPanelMode,
  type DetailPanelPresence,
} from "./detail-panel";
import { Pencil } from "./hugeicons";
import { repoPath } from "../test/repo-path";

/**
 * jsdom does not resolve the CSS poses (cqh geometry), so the panel's computed transform
 * is stubbed per mode and presence; the JS spring then travels between these poses.
 */
function stubPanelPoses(root: HTMLElement, panel: HTMLElement) {
  const original = window.getComputedStyle.bind(window);
  const offsets: Record<string, number> = { maximized: 0, open: 200, collapsed: 340 };
  vi.spyOn(window, "getComputedStyle").mockImplementation((element, pseudo) => {
    if (element !== panel || panel.style.transform !== "") return original(element, pseudo);
    const offscreen = root.dataset.mode === "collapsed" && root.dataset.presence !== "visible";
    const y = offscreen ? 400 : (offsets[root.dataset.mode ?? "open"] ?? 0);
    return { transform: `matrix(1, 0, 0, 1, 0, ${y})` } as CSSStyleDeclaration;
  });
}

/** Springs start after the panel's first painted frame. */
function nextFrame() {
  return new Promise<void>((resolve) => requestAnimationFrame(() => resolve()));
}

afterEach(() => {
  vi.restoreAllMocks();
  vi.unstubAllGlobals();
  document.body.replaceChildren();
});

describe("parseTranslate", () => {
  it("reads the translation of 2D and 3D matrices, and the origin otherwise", () => {
    expect(parseTranslate("matrix(1, 0, 0, 1, 12, 340)")).toEqual({ x: 12, y: 340 });
    expect(
      parseTranslate("matrix3d(1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, -8, 64, 0, 1)"),
    ).toEqual({ x: -8, y: 64 });
    expect(parseTranslate("none")).toEqual({ x: 0, y: 0 });
    expect(parseTranslate("")).toEqual({ x: 0, y: 0 });
  });
});

describe("DetailPanel", () => {
  it("exports the complete compound contract", () => {
    expect(Object.keys(DetailPanel)).toEqual([
      "Root",
      "Collection",
      "Header",
      "Actions",
      "Action",
      "PrimaryAction",
      "Overflow",
      "OverflowItem",
      "Body",
      "Main",
      "Rail",
      "Section",
      "Properties",
      "Property",
      "LinkedEntities",
      "Content",
    ]);
    for (const component of Object.values(DetailPanel)) {
      expect(component).toBeDefined();
    }
  });

  it("composes record main, properties rail, and hides empty properties", async () => {
    const host = document.createElement("div");
    document.body.append(host);
    const onAdd = vi.fn();
    const dispose = render(
      () => (
        <DetailPanel.Root
          mode="open"
          activeEntityId="record"
          onCollapse={() => undefined}
          onMaximize={() => undefined}
          onRestore={() => undefined}
        >
          <DetailPanel.Collection>Collection</DetailPanel.Collection>
          <DetailPanel.Content>
            <DetailPanel.Header title="Record" />
            <DetailPanel.Body>
              <DetailPanel.Main>
                <DetailPanel.Section title="Overview">Description</DetailPanel.Section>
              </DetailPanel.Main>
              <DetailPanel.Rail>
                <DetailPanel.Properties>
                  <DetailPanel.Property label="Provider" value="AWS" />
                  <DetailPanel.Property label="Region" value="us-east-1" mono />
                  <DetailPanel.Property label="Owner" value={null} />
                  <DetailPanel.Property label="Tags" value="" onAdd={onAdd} />
                </DetailPanel.Properties>
              </DetailPanel.Rail>
            </DetailPanel.Body>
          </DetailPanel.Content>
        </DetailPanel.Root>
      ),
      host,
    );

    await vi.waitFor(() =>
      expect(host.querySelector(".pw-detail-panel__record")).not.toBeNull(),
    );
    const record = host.querySelector(".pw-detail-panel__record")!;
    expect(record.querySelector(":scope > .pw-detail-panel__main h3")?.textContent).toBe("Overview");
    expect(record.querySelector(":scope > aside.pw-detail-panel__rail")).not.toBeNull();
    expect(host.querySelector(".pw-detail-panel__subtitle")).toBeNull();
    expect(
      Array.from(host.querySelectorAll(".pw-detail-panel__property dt"), (dt) => dt.textContent),
    ).toEqual(["Provider", "Region", "Tags"]);
    expect(host.querySelector("dd.font-mono")?.textContent).toBe("us-east-1");
    host.querySelector<HTMLButtonElement>(".pw-detail-panel__property-add")!.click();
    expect(onAdd).toHaveBeenCalledOnce();

    const css = readFileSync(
      repoPath("packages/frontend/design-system/src/detail-panel.css"),
      "utf8",
    );
    expect(css).toMatch(
      /\.pw-detail-panel__record:has\(> \.pw-detail-panel__rail\)\s*{[^}]*grid-template-columns: minmax\(0, 1fr\) 280px;/,
    );
    expect(css).toMatch(/@container \(max-width: 720px\)/);
    expect(css).not.toContain(".pw-detail-panel__grid");
    expect(css).not.toContain(".pw-detail-panel__metric");

    dispose();
    host.remove();
  });

  it("mirrors bottom geometry on the X axis for side placement", () => {
    const css = readFileSync(
      repoPath("packages/frontend/design-system/src/detail-panel.css"),
      "utf8",
    );
    const source = readFileSync(
      repoPath("packages/frontend/design-system/src/detail-panel.tsx"),
      "utf8",
    );
    expect(source).toContain('data-panel-placement={props.placement ?? "bottom"}');
    expect(css).toMatch(
      /data-panel-placement="side"\] \.pw-detail-panel__panel\s*{[^}]*transform: translateX\(calc\(100cqw - var\(--detail-panel-side-width\)\)\);/,
    );
    expect(css).toMatch(
      /data-panel-placement="side"\]\[data-mode="maximized"\]\s*\.pw-detail-panel__panel\s*{[^}]*transform: translateX\(0\);/,
    );
    expect(css).not.toContain("[data-placement=");
  });

  it("disables layout transitions for reduced motion", () => {
    const css = readFileSync(
      repoPath("packages/frontend/design-system/src/detail-panel.css"),
      "utf8",
    );
    expect(css).toContain("@media (prefers-reduced-motion: reduce)");
    expect(css).toMatch(/prefers-reduced-motion:[\s\S]*transition: none/);
  });

  it("nests DataTable rows without dark surface fills inside sections", () => {
    const css = readFileSync(
      repoPath("packages/frontend/design-system/src/detail-panel.css"),
      "utf8",
    );
    expect(css).toMatch(
      /\.pw-detail-panel__section \.pw-data-table-scroll\s*{[^}]*padding-inline: 0;/,
    );
    expect(css).toMatch(
      /\.pw-detail-panel__section \.pw-data-table-cell::before[\s\S]*?background: transparent;/,
    );
    // Hover is the table's gliding highlight; rows carry no :hover fill.
    expect(css).not.toMatch(/\.pw-detail-panel__section \.pw-data-table-row:hover/);
    expect(css).toMatch(
      /\.pw-detail-panel__section \.pw-data-table-active,[\s\S]*?background: var\(--color-surface-selected\);/,
    );
    // Nested override must not reintroduce page-level surface chips.
    expect(css).not.toMatch(
      /\.pw-detail-panel__section \.pw-data-table-cell[\s\S]*?background: var\(--color-surface\);/,
    );
  });

  it("keeps discrete button-driven geometry with progressive chrome", () => {
    const css = readFileSync(
      repoPath("packages/frontend/design-system/src/detail-panel.css"),
      "utf8",
    );
    const source = readFileSync(
      repoPath("packages/frontend/design-system/src/detail-panel.tsx"),
      "utf8",
    );
    const materials = readFileSync(
      repoPath("packages/frontend/design-system/src/materials.css"),
      "utf8",
    );
    expect(css).toMatch(
      /\.pw-detail-panel__viewport\s*{[^}]*padding: var\(--detail-panel-gutter\);/,
    );
    expect(css).toMatch(
      /\.pw-detail-panel-root\[data-mode="collapsed"\]\[data-presence="hidden"\][\s\S]*?\.pw-detail-panel__viewport\s*{[^}]*padding: 0;/,
    );
    expect(css).toMatch(
      /\.pw-detail-panel__surface\.surface-panel-float\s*{[^}]*border-radius: var\(--detail-panel-radius\);/,
    );
    expect(css).toMatch(
      /\.pw-detail-panel-root\s*{[^}]*container-type: size;/,
    );
    expect(css).toMatch(
      /\.pw-detail-panel__panel\s*{[^}]*inset: 0;[^}]*height: 100%;[^}]*transform: translateY\(50cqh\);/,
    );
    // The travel is a JS spring; the poses stay in CSS without a transform transition.
    expect(css).not.toMatch(/\.pw-detail-panel__panel\s*{[^}]*transition: transform/);
    expect(css).toMatch(
      /\.pw-detail-panel-root\[data-settling="true"\] \.pw-detail-panel__panel\s*{[^}]*will-change: transform;/,
    );
    expect(css).toMatch(
      /data-mode="collapsed"\] \.pw-detail-panel__panel\s*{[^}]*transform: translateY\(calc\(100cqh - var\(--detail-panel-collapsed-height\)\)\);/,
    );
    expect(css).toMatch(
      /data-mode="maximized"\] \.pw-detail-panel__panel\s*{[^}]*transform: translateY\(0\);/,
    );
    expect(css).toContain("--detail-panel-gutter: 8px;");
    expect(css).toContain("--detail-panel-radius: 14px;");
    expect(css).toContain("--detail-panel-viewport-height: 50cqh;");
    expect(css).toContain("--detail-panel-collection-clearance: 0px;");
    expect(css).toMatch(
      /data-presence="visible"\]:not\(\s*\[data-viewport-mode="maximized"\]\s*\)\s*{[^}]*--detail-panel-collection-clearance: var\(--detail-panel-viewport-height\);/,
    );
    expect(css).toMatch(
      /\.pw-detail-panel__collection \.pw-data-table-root:last-child \.pw-data-table-scroll\s*{[^}]*padding-bottom: var\(--detail-panel-collection-clearance\);/,
    );
    expect(css).not.toMatch(
      /\.pw-detail-panel__collection\s*{[^}]*padding-bottom: var\(--detail-panel-collection-clearance\)/,
    );
    expect(css).not.toMatch(
      /\.pw-detail-panel__collection \.pw-data-table-scroll\s*{[^}]*padding-bottom: var\(--detail-panel-collection-clearance\)/,
    );
    expect(css).toMatch(
      /data-viewport-mode="collapsed"\]\s*{[^}]*--detail-panel-viewport-height: var\(--detail-panel-collapsed-height\);/,
    );
    expect(css).toMatch(
      /data-viewport-mode="maximized"\]\s*{[^}]*--detail-panel-viewport-height: 100cqh;/,
    );
    expect(css).toMatch(
      /data-mode="maximized"\]\s*{[^}]*--detail-panel-gutter: 0px;[^}]*--detail-panel-radius: 0px;/,
    );
    expect(css).toMatch(
      /\.pw-detail-panel__viewport\s*{[^}]*height: var\(--detail-panel-viewport-height\);[^}]*padding: var\(--detail-panel-gutter\);[^}]*transition: padding var\(--detail-panel-snap-duration\)[^}]*var\(--detail-panel-snap-ease\);/,
    );
    expect(css).not.toMatch(/transition:\s*height/);
    expect(css).toMatch(
      /\.pw-detail-panel__panel::before\s*{[^}]*top: 0;[^}]*right: 0;[^}]*left: 0;[^}]*height: var\(--detail-panel-collapsed-height\);[^}]*background: var\(--material-fade-edge-top\);/,
    );
    expect(css).toContain("var(--detail-panel-fade-alpha)");
    expect(css).toMatch(
      /\.pw-detail-panel__viewport\s*{[^}]*position: relative;[^}]*z-index: 1;/,
    );
    expect(css).toMatch(
      /\.pw-detail-panel__portal\s*{[^}]*display: flex;[^}]*height: 100%;[^}]*min-height: 0;[^}]*overflow: hidden;/,
    );
    expect(css).toMatch(
      /\.pw-detail-panel__body\s*{[^}]*min-height: 0;[^}]*flex: 1 1 0;[^}]*overflow: auto;[^}]*overscroll-behavior: contain;[^}]*container-type: inline-size;/,
    );
    expect(css).toMatch(
      /\.pw-detail-panel__title\s*{[^}]*font-size: 14px;/,
    );
    expect(css).not.toMatch(
      /data-mode="collapsed"\] \.pw-detail-panel__title\s*{/,
    );
    expect(css).toMatch(
      /\.pw-detail-panel__identity-content\s*{[^}]*position: relative;[^}]*isolation: isolate;[^}]*height: var\(--control-height-sm\);[^}]*gap: 8px;/,
    );
    expect(css).toMatch(
      /\.pw-detail-panel__identity-content::before\s*{[^}]*inset-block: 0;[^}]*inset-inline: -1rem;[^}]*border-radius: inherit;[^}]*background-color: var\(--color-surface-subtle\);[^}]*var\(--edge-light-field-highlight\)[^}]*var\(--edge-light-field-rim\)[^}]*var\(--detail-panel-title-chrome-alpha\);/,
    );
    expect(css).toContain("--detail-panel-title-chrome-alpha: 0;");
    expect(css).toMatch(
      /data-mode="collapsed"\]\s*{[^}]*--detail-panel-title-chrome-alpha: 1;/,
    );
    expect(css).not.toContain("padding-inline: 1rem;");
    expect(css).toMatch(
      /\.pw-detail-panel__header-icon\s*{[^}]*width: 16px;[^}]*height: 16px;[^}]*flex: 0 0 16px;/,
    );
    expect(css).toMatch(
      /\.pw-detail-panel__header-icon svg\s*{[^}]*transition: transform var\(--detail-panel-snap-duration\)[^}]*var\(--detail-panel-snap-ease\);/,
    );
    expect(css).toMatch(
      /data-mode="collapsed"\] \.pw-detail-panel__header-icon svg\s*{[^}]*transform: scale\(0\.875\);/,
    );
    expect(css).not.toMatch(
      /data-mode="collapsed"\] \.pw-detail-panel__header-icon\s*{[^}]*(?:width|height|flex-basis):/,
    );
    expect(css).not.toContain(".pw-detail-panel__description");
    expect(source).not.toContain("description?: JSX.Element");
    expect(css).not.toContain(
      ".pw-detail-panel__surface.surface-panel-float::after",
    );
    expect(css).toContain(
      "--detail-panel-hover-duration: var(--duration-fast);",
    );
    expect(css).toContain(
      "--detail-panel-hover-ease: var(--ease-standard);",
    );
    expect(css).toContain(
      "opacity var(--detail-panel-snap-duration) var(--detail-panel-snap-ease)",
    );
    expect(css).toContain(
      "background-color var(--detail-panel-hover-duration)",
    );
    expect(css).not.toContain(
      '.pw-detail-panel-root[data-mode="collapsed"][data-settling="false"]',
    );
    expect(css).not.toContain("--detail-panel-chrome-alpha: 1;");
    expect(css).not.toContain(
      ".pw-detail-panel__surface.surface-panel-float:hover",
    );
    expect(css).not.toContain(
      ".pw-detail-panel__surface.surface-panel-float:focus-within",
    );
    expect(css).toContain(
      ".pw-detail-panel__identity:has(.pw-detail-panel__collapsed-trigger:hover)",
    );
    expect(css).toMatch(
      /@media \(hover: hover\) and \(pointer: fine\)\s*{\s*\.pw-detail-panel-root\[data-mode="collapsed"\][\s\S]*?\.pw-detail-panel__collapsed-trigger:hover\)[\s\S]*?background-color: var\(--color-surface-hover\);\s*}\s*}\s*\.pw-detail-panel-root\[data-mode="collapsed"\][\s\S]*?\.pw-detail-panel__collapsed-trigger:focus-visible/,
    );
    expect(css).toMatch(
      /\.pw-detail-panel__collapsed-trigger:focus-visible[\s\S]*?\.pw-detail-panel__identity-content::before\s*{[^}]*background-color: var\(--color-surface-hover\);/,
    );
    expect(css).toMatch(
      /\.pw-detail-panel__collapsed-trigger:active\)[\s\S]*?\.pw-detail-panel__identity-content::before\s*{[^}]*background-color: var\(--color-surface-active\);/,
    );
    expect(css).toMatch(
      /\.pw-detail-panel__identity:has\(\.pw-detail-panel__collapsed-trigger:focus-visible\)[\s\S]*?\.pw-detail-panel__identity-content::before\s*{[^}]*outline: 1px solid var\(--color-border-focus\);[^}]*outline-offset: 2px;/,
    );
    expect(css).not.toMatch(
      /\.pw-detail-panel__surface\.surface-panel-float\s*{[^}]*--detail-panel-fade-alpha: 0;/,
    );
    expect(css).toMatch(
      /\.pw-detail-panel__collapsed-trigger\s*{[^}]*cursor: pointer;[^}]*touch-action: manipulation;/,
    );
    expect(source).not.toContain("@corvu/resizable");
    expect(source).not.toContain("Resizable.Handle");
    expect(source).not.toContain("settleDetailPanelSize");
    expect(source).not.toContain("DETAIL_PANEL_SETTLEMENT_DURATION");
    expect(source).toContain("readDetailPanelPose(panelElement)");
    expect(source).toContain("animate(panelY, target.y, spring.slow)");
    expect(source).not.toContain("onTransitionEnd");
    expect(css).not.toContain("row-resize");
    expect(css).not.toContain("pw-detail-panel__resize-handle");
    expect(css).toMatch(
      /\.pw-detail-panel__header\s*{[^}]*display: grid;[^}]*grid-template-columns: minmax\(0, 1fr\) auto minmax\(0, 1fr\);[^}]*border-bottom: 1px solid var\(--color-border-subtle\);[^}]*gap: 4px;/,
    );
    expect(css).toMatch(
      /data-mode="collapsed"\] \.pw-detail-panel__header\s*{[^}]*border-bottom: 0;/,
    );
    expect(css).toMatch(
      /\.pw-detail-panel__identity\s*{[^}]*grid-column: 2;[^}]*grid-row: 1;[^}]*justify-self: center;/,
    );
    expect(css).toMatch(
      /\.pw-detail-panel__actions\s*{[^}]*grid-column: 1;[^}]*grid-row: 1;[^}]*justify-content: flex-start;[^}]*justify-self: start;/,
    );
    expect(css).toMatch(
      /\.pw-detail-panel__global-actions\s*{[^}]*grid-column: 3;[^}]*grid-row: 1;[^}]*justify-content: flex-end;[^}]*justify-self: end;[^}]*margin-inline-start: 0;/,
    );
    expect(css).toMatch(
      /\.pw-detail-panel__actions\s*{[^}]*margin-inline-start: 0;/,
    );
    expect(source).toMatch(
      /<Tooltip\.Group>\s*{props\.children}\s*<div class="pw-detail-panel__global-actions">/,
    );
    expect(source).toContain('openDelay={DETAIL_PANEL_ACTION_TOOLTIP_DELAY}');
    // Labels come from the configurable shortcut store, not hard-coded constants.
    expect(source).toContain('createShortcutLabel("shell.toggle_detail")');
    expect(source).toContain('createShortcutLabel("shell.maximize_detail")');
    expect(source).toContain("shortcut={toggleShortcut()}");
    expect(source).toContain("shortcut={maximizeShortcut()}");
    expect(source).toContain("<Kbd>{local.shortcut}</Kbd>");
    expect(css).toContain(".pw-detail-panel__action-tooltip");
    expect(materials).toContain("--material-fade-edge-bottom: linear-gradient(");
    expect(materials).toContain("--material-fade-edge-top: linear-gradient(to top");
  });

  it("sequences viewport geometry around the panel spring and retargeting", async () => {
    const host = document.createElement("div");
    document.body.append(host);
    const [mode, setMode] = createSignal<DetailPanelMode>("maximized");
    const dispose = render(
      () => (
        <DetailPanel.Root
          mode={mode()}
          activeEntityId="motion-detail"
          onCollapse={() => setMode("collapsed")}
          onMaximize={() => setMode("maximized")}
          onRestore={() => setMode("open")}
        >
          <DetailPanel.Collection>Collection</DetailPanel.Collection>
          <DetailPanel.Content>
            <DetailPanel.Header title="Motion detail" />
            <DetailPanel.Body>Motion body</DetailPanel.Body>
          </DetailPanel.Content>
        </DetailPanel.Root>
      ),
      host,
    );

    const root = host.querySelector<HTMLElement>(".pw-detail-panel-root")!;
    const panel = host.querySelector<HTMLElement>(".pw-detail-panel__panel")!;
    const body = host.querySelector<HTMLElement>(".pw-detail-panel__body")!;
    stubPanelPoses(root, panel);
    await nextFrame();
    expect(root.dataset.viewportMode).toBe("maximized");

    setMode("open");
    await vi.waitFor(() => expect(root.dataset.mode).toBe("open"));
    expect(root.dataset.viewportMode).toBe("maximized");
    expect(root.dataset.settling).toBe("true");
    expect(body.hidden).toBe(false);
    await vi.waitFor(() => expect(root.dataset.settling).toBe("false"));
    expect(root.dataset.viewportMode).toBe("open");

    setMode("collapsed");
    await vi.waitFor(() => expect(root.dataset.mode).toBe("collapsed"));
    expect(root.dataset.viewportMode).toBe("open");
    expect(body.hidden).toBe(false);
    await vi.waitFor(() => expect(root.dataset.viewportMode).toBe("collapsed"));
    expect(body.hidden).toBe(true);

    setMode("open");
    await vi.waitFor(() => expect(root.dataset.viewportMode).toBe("open"));
    expect(body.hidden).toBe(false);
    await vi.waitFor(() => expect(root.dataset.settling).toBe("false"));

    setMode("maximized");
    await vi.waitFor(() => expect(root.dataset.viewportMode).toBe("maximized"));
    await vi.waitFor(() => expect(root.dataset.settling).toBe("false"));

    // Interrupted: a collapse retargeted to open before it lands never commits.
    setMode("collapsed");
    expect(root.dataset.settling).toBe("true");
    setMode("open");
    await vi.waitFor(() => expect(root.dataset.mode).toBe("open"));
    expect(root.dataset.viewportMode).toBe("maximized");
    await vi.waitFor(() => expect(root.dataset.viewportMode).toBe("open"));
    expect(root.dataset.settling).toBe("false");

    dispose();
    host.remove();
  });

  it("commits viewport geometry immediately for reduced motion", async () => {
    vi.stubGlobal("matchMedia", vi.fn(() => ({
      matches: true,
      media: "(prefers-reduced-motion: reduce)",
      onchange: null,
      addListener: vi.fn(),
      removeListener: vi.fn(),
      addEventListener: vi.fn(),
      removeEventListener: vi.fn(),
      dispatchEvent: vi.fn(),
    })));
    const host = document.createElement("div");
    document.body.append(host);
    const [mode, setMode] = createSignal<DetailPanelMode>("open");
    const dispose = render(
      () => (
        <DetailPanel.Root
          mode={mode()}
          activeEntityId="reduced-motion-detail"
          onCollapse={() => setMode("collapsed")}
          onMaximize={() => setMode("maximized")}
          onRestore={() => setMode("open")}
        >
          <DetailPanel.Collection>Collection</DetailPanel.Collection>
          <DetailPanel.Content>
            <DetailPanel.Header title="Reduced motion detail" />
            <DetailPanel.Body>Reduced motion body</DetailPanel.Body>
          </DetailPanel.Content>
        </DetailPanel.Root>
      ),
      host,
    );

    const root = host.querySelector<HTMLElement>(".pw-detail-panel-root")!;
    const body = host.querySelector<HTMLElement>(".pw-detail-panel__body")!;
    setMode("collapsed");
    await vi.waitFor(() => expect(root.dataset.mode).toBe("collapsed"));
    expect(root.dataset.viewportMode).toBe("collapsed");
    expect(root.dataset.settling).toBe("false");
    expect(body.hidden).toBe(true);

    dispose();
    host.remove();
  });

  it("restores fallback focus before disabling a dismissing detail and completes when the spring settles", async () => {
    const host = document.createElement("div");
    document.body.append(host);
    const [mode, setMode] = createSignal<DetailPanelMode>("open");
    const [presence, setPresence] = createSignal<DetailPanelPresence>("visible");
    const onDismissComplete = vi.fn();
    const dispose = render(
      () => (
        <DetailPanel.Root
          mode={mode()}
          presence={presence()}
          activeEntityId="alpha"
          onCollapse={() => setMode("collapsed")}
          onMaximize={() => setMode("maximized")}
          onRestore={() => setMode("open")}
          onDismissComplete={onDismissComplete}
        >
          <DetailPanel.Collection>
            <button data-detail-focus-fallback>Canvas</button>
          </DetailPanel.Collection>
          <DetailPanel.Content>
            <DetailPanel.Header title="Alpha detail" />
            <DetailPanel.Body>
              <button data-detail-action>Detail action</button>
            </DetailPanel.Body>
          </DetailPanel.Content>
        </DetailPanel.Root>
      ),
      host,
    );

    await vi.waitFor(() => expect(host.querySelector("[data-detail-action]")).not.toBeNull());
    host.querySelector<HTMLButtonElement>("[data-detail-action]")!.focus();
    const surface = host.querySelector<HTMLElement>(".pw-detail-panel__surface")!;
    const fallback = host.querySelector<HTMLButtonElement>("[data-detail-focus-fallback]")!;
    const panel = host.querySelector<HTMLElement>(".pw-detail-panel__panel")!;
    const root = host.querySelector<HTMLElement>(".pw-detail-panel-root")!;
    stubPanelPoses(root, panel);
    await nextFrame();

    setMode("collapsed");
    setPresence("dismissing");
    expect(surface.inert).not.toBe(true);
    expect(onDismissComplete).not.toHaveBeenCalled();

    await vi.waitFor(() => expect(document.activeElement).toBe(fallback));
    expect(surface.inert).toBe(true);
    expect(surface.getAttribute("aria-hidden")).toBe("true");
    expect(onDismissComplete).not.toHaveBeenCalled();

    await vi.waitFor(() => expect(onDismissComplete).toHaveBeenCalledOnce());

    setPresence("visible");
    setMode("open");
    await vi.waitFor(() => expect(root.dataset.settling).toBe("false"));
    expect(onDismissComplete).toHaveBeenCalledOnce();

    dispose();
    host.remove();
  });

  it("completes reduced-motion dismissal immediately after focus handoff", async () => {
    vi.stubGlobal("matchMedia", vi.fn(() => ({
      matches: true,
      media: "(prefers-reduced-motion: reduce)",
      onchange: null,
      addListener: vi.fn(),
      removeListener: vi.fn(),
      addEventListener: vi.fn(),
      removeEventListener: vi.fn(),
      dispatchEvent: vi.fn(),
    })));
    const host = document.createElement("div");
    document.body.append(host);
    const [presence, setPresence] = createSignal<DetailPanelPresence>("visible");
    const onDismissComplete = vi.fn();
    const dispose = render(
      () => (
        <DetailPanel.Root
          mode="collapsed"
          presence={presence()}
          activeEntityId="alpha"
          onCollapse={() => undefined}
          onMaximize={() => undefined}
          onRestore={() => undefined}
          onDismissComplete={onDismissComplete}
        >
          <DetailPanel.Collection>
            <button data-detail-focus-fallback>Canvas</button>
          </DetailPanel.Collection>
          <DetailPanel.Content>
            <DetailPanel.Header title="Alpha detail" />
            <DetailPanel.Body>Body</DetailPanel.Body>
          </DetailPanel.Content>
        </DetailPanel.Root>
      ),
      host,
    );

    setPresence("dismissing");
    await vi.waitFor(() => expect(onDismissComplete).toHaveBeenCalledOnce());
    expect(document.activeElement).toBe(
      host.querySelector("[data-detail-focus-fallback]"),
    );

    dispose();
    host.remove();
  });

  it("portals content, exposes controls, applies inert, restores with Escape, and restores focus", async () => {
    const host = document.createElement("div");
    document.body.append(host);
    const [mode, setMode] = createSignal<DetailPanelMode>("open");
    const dispose = render(
      () => (
        <DetailPanel.Root
          mode={mode()}
          activeEntityId="alpha"
          onCollapse={() => setMode("collapsed")}
          onMaximize={() => setMode("maximized")}
          onRestore={() => setMode("open")}
        >
          <DetailPanel.Collection>
            <button data-detail-target-id="alpha">Alpha row</button>
            <DetailPanel.Content>
              <DetailPanel.Header icon={<Pencil />} title="Alpha detail">
                <DetailPanel.Actions>
                  <DetailPanel.Action label="Edit" tone="primary">
                    <Pencil />
                  </DetailPanel.Action>
                </DetailPanel.Actions>
              </DetailPanel.Header>
              <DetailPanel.Body>Complete detail</DetailPanel.Body>
            </DetailPanel.Content>
          </DetailPanel.Collection>
        </DetailPanel.Root>
      ),
      host,
    );

    await vi.waitFor(() => expect(host.textContent).toContain("Complete detail"));
    const panel = host.querySelector<HTMLElement>(".pw-detail-panel__panel")!;
    stubPanelPoses(host.querySelector<HTMLElement>(".pw-detail-panel-root")!, panel);
    await nextFrame();
    expect(host.querySelector(".pw-detail-panel__surface")?.classList.contains("surface-panel-float"))
      .toBe(true);
    expect(host.querySelector("[aria-label^='Dock details']")).toBeNull();
    expect(host.querySelector("[aria-label='Maximize details']")).not.toBeNull();
    const header = host.querySelector(".pw-detail-panel__header")!;
    const headerIcon = host.querySelector(".pw-detail-panel__header-icon")!;
    const portal = host.querySelector<HTMLElement>(".pw-detail-panel__portal")!;
    expect(header).not.toBeNull();
    expect(headerIcon.getAttribute("aria-hidden")).not.toBeNull();
    expect(headerIcon.querySelector("svg")).not.toBeNull();
    expect(
      Array.from(header.children, (element) => element.classList.item(0)),
    ).toEqual([
      "pw-detail-panel__identity",
      "pw-detail-panel__actions",
      "pw-detail-panel__global-actions",
    ]);
    expect(
      Array.from(header.querySelectorAll("button"), (button) =>
        button.getAttribute("aria-label")),
    ).toEqual(["Edit", "Maximize details", "Collapse details"]);
    expect(host.querySelector("[aria-label='Resize detail panel']")).toBeNull();
    expect(host.querySelector("[data-corvu-resizable-handle]")).toBeNull();
    expect(portal.parentElement?.classList.contains("pw-detail-panel__surface")).toBe(true);

    const collapseButton = host.querySelector<HTMLButtonElement>("[aria-label='Collapse details']")!;
    const domainAction = host.querySelector<HTMLButtonElement>("[aria-label='Edit']")!;
    expect(domainAction.textContent).toBe("");
    expect(domainAction.classList.contains("!rounded-full")).toBe(true);
    expect(domainAction.dataset.tone).toBe("primary");
    expect(domainAction.hasAttribute("title")).toBe(false);
    expect(collapseButton.classList.contains("control-field")).toBe(true);
    expect(collapseButton.classList.contains("control-field-sm")).toBe(true);
    expect(collapseButton.classList.contains("!rounded-full")).toBe(true);
    host.querySelector<HTMLButtonElement>("[aria-label='Maximize details']")!.click();
    await vi.waitFor(() => expect(mode()).toBe("maximized"));
    const collection = host.querySelector<HTMLElement & { inert: boolean }>(
      ".pw-detail-panel__collection",
    );
    await vi.waitFor(() => expect(collection?.inert).toBe(true));
    expect(host.querySelector("[aria-label='Restore details']")).not.toBeNull();
    expect(host.querySelector("[aria-label='Maximize details']")).toBeNull();

    window.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape", bubbles: true }));
    await vi.waitFor(() => expect(mode()).toBe("open"));
    expect(collection?.inert).toBe(true);
    await vi.waitFor(() => expect(collection?.inert).toBe(false));

    const row = host.querySelector<HTMLButtonElement>("[data-detail-target-id='alpha']")!;
    host.querySelector<HTMLButtonElement>("[aria-label='Collapse details']")!.click();
    await vi.waitFor(() => expect(mode()).toBe("collapsed"));
    expect(document.activeElement).toBe(row);
    expect(host.querySelector("[data-detail-panel-collapsed-trigger]")).not.toBeNull();
    expect(host.querySelector("[aria-label='Open details']")).not.toBeNull();
    expect(host.querySelector("[aria-label='Maximize details']")).not.toBeNull();
    expect(host.querySelector("[aria-label='Collapse details']")).toBeNull();
    const collapsedIdentity = host.querySelector(".pw-detail-panel__identity-content")!;
    expect(collapsedIdentity.classList.contains("control-field")).toBe(false);
    expect(collapsedIdentity.classList.contains("!rounded-full")).toBe(true);
    expect(collapsedIdentity.querySelector(".pw-detail-panel__title")?.textContent).toBe(
      "Alpha detail",
    );

    dispose();
    host.remove();
  });

  it("keeps retained entity surfaces on the shared DetailPanel", () => {
    const featureFiles = [
      repoPath("packages/features/architecture/src/node-detail.tsx"),
      repoPath("packages/features/architecture/src/view-detail.tsx"),
      repoPath("packages/features/integrations/src/integration-detail.tsx"),
      repoPath("packages/features/knowledge/src/knowledge-document-shell.tsx"),
      repoPath("packages/features/resources/src/resource-detail.tsx"),
    ];

    for (const file of featureFiles) {
      const source = readFileSync(file, "utf8");
      expect(source, file).toContain("<DetailPanel.Content>");
      expect(source, file).toContain("<DetailPanel.Header");
    }
  });

  it("keeps the header available and expands when the collapsed strip is clicked", async () => {
    const host = document.createElement("div");
    document.body.append(host);
    const [mode, setMode] = createSignal<DetailPanelMode>("open");
    const dispose = render(
      () => (
        <DetailPanel.Root
          mode={mode()}
          activeEntityId="persistent-detail"
          onCollapse={() => setMode("collapsed")}
          onMaximize={() => setMode("maximized")}
          onRestore={() => setMode("open")}
        >
          <DetailPanel.Collection>Collection</DetailPanel.Collection>
          <DetailPanel.Content>
            <DetailPanel.Header title="Persistent detail header" />
            <DetailPanel.Body>Hidden detail body</DetailPanel.Body>
          </DetailPanel.Content>
        </DetailPanel.Root>
      ),
      host,
    );

    stubPanelPoses(
      host.querySelector<HTMLElement>(".pw-detail-panel-root")!,
      host.querySelector<HTMLElement>(".pw-detail-panel__panel")!,
    );
    await nextFrame();
    const collapseButton = host.querySelector<HTMLButtonElement>(
      "[aria-label='Collapse details']",
    );
    expect(collapseButton).not.toBeNull();
    collapseButton?.click();

    await vi.waitFor(() => expect(mode()).toBe("collapsed"));
    expect(host.textContent).toContain("Persistent detail header");
    const body = host.querySelector<HTMLElement>(".pw-detail-panel__body")!;
    expect(body.hidden).toBe(false);
    await vi.waitFor(() => expect(body.hidden).toBe(true));
    expect(host.querySelector<HTMLElement>(".pw-detail-panel__surface")?.hidden).toBe(false);

    const collapsedTrigger = host.querySelector<HTMLButtonElement>(
      "[data-detail-panel-collapsed-trigger]",
    );
    expect(collapsedTrigger?.getAttribute("aria-label")).toBe("Open details");
    expect(host.querySelector("[data-corvu-resizable-handle]")).toBeNull();
    collapsedTrigger?.click();
    await vi.waitFor(() => expect(mode()).toBe("open"));
    expect(body.hidden).toBe(false);

    dispose();
    host.remove();
  });

  it("keeps empty collapsed chrome hidden when no detail target exists", async () => {
    const host = document.createElement("div");
    document.body.append(host);
    const dispose = render(
      () => (
        <DetailPanel.Root
          mode="collapsed"
          activeEntityId={null}
          onCollapse={() => undefined}
          onMaximize={() => undefined}
          onRestore={() => undefined}
        >
          <DetailPanel.Collection>Collection without details</DetailPanel.Collection>
        </DetailPanel.Root>
      ),
      host,
    );

    await vi.waitFor(() =>
      expect(host.querySelector(".pw-detail-panel__surface")).not.toBeNull(),
    );
    const surface = host.querySelector<HTMLElement>(".pw-detail-panel__surface")!;
    expect(surface.getAttribute("aria-hidden")).toBe("true");
    expect(surface.inert).toBe(true);

    dispose();
    host.remove();
  });

  it("moves directly between all modes through independent buttons", async () => {
    const host = document.createElement("div");
    document.body.append(host);
    const [mode, setMode] = createSignal<DetailPanelMode>("collapsed");
    const onEdit = vi.fn();
    const dispose = render(
      () => (
        <DetailPanel.Root
          mode={mode()}
          activeEntityId="detail"
          onCollapse={() => setMode("collapsed")}
          onMaximize={() => setMode("maximized")}
          onRestore={() => setMode("open")}
        >
          <DetailPanel.Collection>Collection</DetailPanel.Collection>
          <DetailPanel.Content>
            <DetailPanel.Header title="Detail">
              <DetailPanel.Actions>
                <DetailPanel.Action label="Edit" onClick={onEdit}>
                  <Pencil />
                </DetailPanel.Action>
              </DetailPanel.Actions>
            </DetailPanel.Header>
            <DetailPanel.Body>Body</DetailPanel.Body>
          </DetailPanel.Content>
        </DetailPanel.Root>
      ),
      host,
    );

    await vi.waitFor(() => expect(host.querySelector("[aria-label='Edit']")).not.toBeNull());
    expect(host.querySelectorAll("[aria-label='Open details']")).toHaveLength(2);
    expect(host.querySelector("[aria-label='Maximize details']")).not.toBeNull();
    expect(host.querySelector("[aria-label='Collapse details']")).toBeNull();
    expect(
      Array.from(
        host.querySelectorAll(".pw-detail-panel__global-actions button"),
        (button) => button.getAttribute("aria-label"),
      ),
    ).toEqual(["Maximize details", "Open details"]);

    host.querySelector<HTMLButtonElement>("[aria-label='Edit']")!.click();
    expect(onEdit).toHaveBeenCalledOnce();
    expect(mode()).toBe("collapsed");

    host.querySelector<HTMLButtonElement>("[aria-label='Maximize details']")!.click();
    await vi.waitFor(() => expect(mode()).toBe("maximized"));
    expect(host.querySelector("[aria-label='Collapse details']")).not.toBeNull();
    expect(host.querySelector("[aria-label='Restore details']")).not.toBeNull();
    expect(host.querySelector("[aria-label='Open details']")).toBeNull();

    host.querySelector<HTMLButtonElement>("[aria-label='Collapse details']")!.click();
    await vi.waitFor(() => expect(mode()).toBe("collapsed"));

    host.querySelector<HTMLButtonElement>("[aria-label='Maximize details']")!.click();
    await vi.waitFor(() => expect(mode()).toBe("maximized"));
    host.querySelector<HTMLButtonElement>("[aria-label='Restore details']")!.click();
    await vi.waitFor(() => expect(mode()).toBe("open"));
    expect(host.querySelector("[aria-label='Collapse details']")).not.toBeNull();
    expect(host.querySelector("[aria-label='Maximize details']")).not.toBeNull();
    expect(host.querySelector("[aria-label='Restore details']")).toBeNull();

    host.querySelector<HTMLButtonElement>("[aria-label='Collapse details']")!.click();
    await vi.waitFor(() => expect(mode()).toBe("collapsed"));
    host.querySelector<HTMLButtonElement>("[data-detail-panel-collapsed-trigger]")!.click();
    await vi.waitFor(() => expect(mode()).toBe("open"));

    dispose();
    host.remove();
  });

  it("toggles and maximizes with Command+J shortcuts", async () => {
    const host = document.createElement("div");
    document.body.append(host);
    const [mode, setMode] = createSignal<DetailPanelMode>("open");
    const dispose = render(
      () => (
        <DetailPanel.Root
          mode={mode()}
          activeEntityId="alpha"
          onCollapse={() => setMode("collapsed")}
          onMaximize={() => setMode("maximized")}
          onRestore={() => setMode("open")}
        >
          <DetailPanel.Collection>
            <button data-detail-target-id="alpha">Alpha row</button>
          </DetailPanel.Collection>
          <DetailPanel.Content>
            <DetailPanel.Header title="Alpha detail" />
            <DetailPanel.Body>Body</DetailPanel.Body>
          </DetailPanel.Content>
        </DetailPanel.Root>
      ),
      host,
    );

    await vi.waitFor(() => expect(host.querySelector("[aria-label='Collapse details']")).not.toBeNull());

    window.dispatchEvent(
      new KeyboardEvent("keydown", { key: "j", metaKey: true, bubbles: true }),
    );
    await vi.waitFor(() => expect(mode()).toBe("collapsed"));

    window.dispatchEvent(
      new KeyboardEvent("keydown", { key: "j", metaKey: true, bubbles: true }),
    );
    await vi.waitFor(() => expect(mode()).toBe("open"));

    window.dispatchEvent(
      new KeyboardEvent("keydown", {
        key: "j",
        metaKey: true,
        shiftKey: true,
        bubbles: true,
      }),
    );
    await vi.waitFor(() => expect(mode()).toBe("maximized"));

    window.dispatchEvent(
      new KeyboardEvent("keydown", {
        key: "j",
        metaKey: true,
        shiftKey: true,
        bubbles: true,
      }),
    );
    await vi.waitFor(() => expect(mode()).toBe("open"));

    dispose();
    host.remove();
  });
});
