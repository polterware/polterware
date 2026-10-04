import { Show, batch, createContext, createEffect, createSignal, createUniqueId, onCleanup, onMount, untrack, useContext, type Accessor, type JSX } from "solid-js";
import { Portal, isServer } from "solid-js/web";
import { animate, motionValue, type AnimationPlaybackControls } from "motion";
import { Button } from "./button";
import { prefersReducedMotion } from "./reduced-motion";
import { spring } from "./springs";
import { CollapseIcon, ExpandIcon, X } from "./hugeicons";
import { PanelInteractionContext, panelOwnerAt, rememberPanelInteraction, registerPanelInteraction, type PanelInteractionOwner } from "./panel-interaction";
import { DetailPanelLinkedEntities } from "./detail-panel-linked-entities";
import { DetailPanelMenuItem } from "./detail-panel-menu";
import { DetailPanelPrimaryAction } from "./detail-panel-primary-action";
import { DetailPanelProperties, DetailPanelProperty } from "./detail-panel-properties";
import { useDesignSystem } from "./provider";
import "./detail-panel.css";

export const DETAIL_PANEL_DEFAULT_WIDTH = 400;
export const DETAIL_PANEL_MIN_WIDTH = 320;
export const DETAIL_PANEL_MAX_WIDTH = 720;
export const DETAIL_PANEL_SIDEBAR_ID = "pw-detail-panel-sidebar";

export function clampDetailPanelWidth(width: number, containerWidth: number): number {
  const upper = containerWidth > 0 ? Math.min(DETAIL_PANEL_MAX_WIDTH, Math.max(1, containerWidth - 16)) : DETAIL_PANEL_MAX_WIDTH;
  return Math.min(upper, Math.max(Math.min(DETAIL_PANEL_MIN_WIDTH, upper), Number.isFinite(width) ? width : DETAIL_PANEL_DEFAULT_WIDTH));
}

/** Floating JSON is intentionally ignored; its preference remains available to FloatingWindow. */
export function readDetailPanelWidth(raw: string | null): number {
  try {
    const value: unknown = JSON.parse(raw ?? "null");
    return typeof value === "number" && Number.isFinite(value) && value > 0
      ? clampDetailPanelWidth(value, 0) : DETAIL_PANEL_DEFAULT_WIDTH;
  } catch { return DETAIL_PANEL_DEFAULT_WIDTH; }
}

export type DetailPanelRootProps = {
  open: boolean;
  expanded: boolean;
  available?: boolean;
  activeEntityId?: string | null;
  onOpenChange: (open: boolean) => void;
  onExpandedChange: (expanded: boolean) => void;
  storageKey?: string;
  /** Read only numeric preferences from the previous key; never overwrite floating geometry. */
  legacyStorageKey?: string;
  children: JSX.Element;
  class?: string;
  "aria-label"?: string;
};

export type DetailPanelBodyProps = {
  children: JSX.Element;
  class?: string;
  /** `record` = Main + optional Rail; `document` = knowledge writing canvas. */
  layout?: "record" | "document";
};

export type DetailPanelHeaderProps = {
  class?: string;
};

export type DetailPanelTitleProps = {
  title: JSX.Element;
  icon?: JSX.Element;
  meta?: JSX.Element;
};

export type DetailPanelSectionProps = {
  children: JSX.Element;
  class?: string;
  title?: JSX.Element;
  /** Right-aligned element on the section title row. */
  aside?: JSX.Element;
};

type DetailPanelContextValue = {
  expanded: Accessor<boolean>;
  open: Accessor<boolean>;
  available: Accessor<boolean>;
  activeEntityId: Accessor<string | null>;
  detailHost: Accessor<HTMLElement | undefined>;
  setHeaderClass: (value: string) => void;
  collectionHeaderHost: Accessor<HTMLElement | undefined>;
  interaction: PanelInteractionOwner;
};
const DetailPanelContext = createContext<DetailPanelContextValue>();
function useDetailPanel(part: string): DetailPanelContextValue {
  const context = useContext(DetailPanelContext);
  if (!context) throw new Error(`DetailPanel.${part} must be used inside DetailPanel.Root`);
  return context;
}

/** Collection controls remain above the divided workspace; standalone pages render inline. */
export function DetailPanelCollectionHeader(props: { children: JSX.Element; class?: string }) {
  const context = useContext(DetailPanelContext);
  if (!context) return props.children;
  return <Show when={context.collectionHeaderHost()}>{(host) =>
    <Portal mount={host()}>
      <div class={`pw-detail-panel__collection-header-part ${props.class ?? ""}`}>{props.children}</div>
    </Portal>
  }</Show>;
}

function DetailPanelRoot(props: DetailPanelRootProps) {
  const designSystem = useDesignSystem();
  const storageKey = () => props.storageKey ?? designSystem.storageKey("shell.detail.sidebar.width");
  const [detailHost, setDetailHost] = createSignal<HTMLElement>();
  const [headerClass, setHeaderClass] = createSignal("");
  const [collectionHeaderHost, setCollectionHeaderHost] = createSignal<HTMLElement>();
  const [preferredWidth, setPreferredWidth] = createSignal(readStoredWidth({ storageKey: storageKey(), legacyStorageKey: props.legacyStorageKey }));
  const [hostWidth, setHostWidth] = createSignal(0);
  const [present, setPresent] = createSignal(props.open && props.available !== false);
  const [gesturing, setGesturing] = createSignal(false);
  const [exitGeometry, setExitGeometry] = createSignal<{ width: number; expanded: boolean }>();
  const available = () => props.available !== false;
  const visible = () => available() && props.open;
  const expanded = () => visible() && props.expanded;
  const panelWidth = () => clampDetailPanelWidth(preferredWidth(), hostWidth());
  const activeEntityId = () => props.activeEntityId ?? null;
  const interaction: PanelInteractionOwner = { enabled: visible, elements: new Set() };
  const expandLabel = () => expanded() ? "Restore details" : "Expand details";
  const closeShortcut = designSystem.createShortcutLabel("shell.toggle_detail");
  const expandShortcut = designSystem.createShortcutLabel("shell.maximize_detail");
  let root: HTMLDivElement | undefined;
  let frame: HTMLDivElement | undefined;
  let workspace: HTMLDivElement | undefined;
  let divider: HTMLDivElement | undefined;
  let ready = false;
  let preserveOutsideFocus = false;
  let cancelResize: (() => void) | undefined;
  let timer: number | undefined;
  let run = 0;
  let disposed = false;
  let frameAnimation: AnimationPlaybackControls[] = [];
  const offset = motionValue(0), opacity = motionValue(visible() ? 1 : 0);
  let lastOpen = visible();
  let lastExpanded = expanded();

  function paint() {
    if (frame) { frame.style.transform = `translate3d(${offset.get()}px, 0, 0)`; frame.style.opacity = String(opacity.get()); }
  }
  function stopFrame() {
    run++;
    if (timer !== undefined) window.clearTimeout(timer);
    timer = undefined;
    frameAnimation.forEach((control) => control.stop());
    frameAnimation = [];
  }
  function focusInFrame() { return !isServer && Boolean(frame?.contains(document.activeElement)); }
  function restoreFocus() {
    if (isServer) return;
    queueMicrotask(() => {
      if (disposed) return;
      const id = activeEntityId();
      const candidates = root?.querySelectorAll<HTMLElement>("[data-detail-target-id], [data-row-id]");
      const target = Array.from(candidates ?? []).find((element) => element.dataset.detailTargetId === id || element.dataset.rowId === id)
        ?? root?.querySelector<HTMLElement>("[data-detail-focus-fallback]") ?? root;
      target?.focus({ preventScroll: true });
    });
  }
  function close(outside = false) {
    cancelResize?.();
    resetOutsideGesture();
    preserveOutsideFocus = outside;
    props.onOpenChange(false);
  }
  function toggleExpanded() {
    cancelResize?.();
    if (!visible()) return;
    props.onExpandedChange(!props.expanded);
  }
  // Observe external presses without taking capture from controls or canvas gestures.
  let outsidePress: { pointerId: number; x: number; y: number; moved: boolean } | undefined;
  let ignoreOutsideClick = false;
  let outsideClickReset: number | undefined;
  function resetOutsideGesture() {
    outsidePress = undefined;
    ignoreOutsideClick = false;
    if (outsideClickReset !== undefined) cancelAnimationFrame(outsideClickReset);
    outsideClickReset = undefined;
  }
  function resetOutsideClickAfterRelease() {
    if (outsideClickReset !== undefined) cancelAnimationFrame(outsideClickReset);
    outsideClickReset = requestAnimationFrame(() => { ignoreOutsideClick = false; outsideClickReset = undefined; });
  }
  function trackOutsidePress(event: PointerEvent) {
    rememberPanelInteraction(event.target);
    if (event.button !== 0 || event.isPrimary === false) return;
    resetOutsideGesture();
    if (visible()) outsidePress = { pointerId: event.pointerId, x: event.clientX, y: event.clientY, moved: false };
  }
  function trackOutsideMove(event: PointerEvent) {
    if (!outsidePress || event.pointerId !== outsidePress.pointerId) return;
    if (Math.hypot(event.clientX - outsidePress.x, event.clientY - outsidePress.y) >= 3) outsidePress.moved = true;
  }
  function releaseOutsidePress(event: PointerEvent) {
    if (!outsidePress || event.pointerId !== outsidePress.pointerId) return;
    trackOutsideMove(event);
    ignoreOutsideClick = outsidePress.moved;
    outsidePress = undefined;
    resetOutsideClickAfterRelease();
  }
  function cancelOutsidePress(event?: PointerEvent) {
    if (!outsidePress || (event && event.pointerId !== outsidePress.pointerId)) return;
    outsidePress = undefined;
    ignoreOutsideClick = true;
    resetOutsideClickAfterRelease();
  }
  function trackOutsideScroll() {
    if (outsidePress) outsidePress.moved = true;
  }
  function onOutsideClick(event: MouseEvent) {
    rememberPanelInteraction(event.target);
    if (!visible() || event.defaultPrevented) return;
    const fromPointer = event.detail > 0 || ("pointerType" in event && Boolean(event.pointerType));
    if (fromPointer && (ignoreOutsideClick || outsidePress?.moved || suppressClick)) return;
    if (panelOwnerAt(event.target) === interaction || (event.target instanceof Element && event.target.closest("[data-detail-opener]"))) return;
    close(true);
  }
  function onKeyDown(event: KeyboardEvent) {
    if (event.defaultPrevented) return;
    if (event.key === "Escape") {
      if (cancelResize) { event.preventDefault(); cancelResize(); return; }
      if (expanded() && (focusInFrame() || document.activeElement === document.body)) {
        event.preventDefault(); props.onExpandedChange(false); return;
      }
      if (focusInFrame()) { event.preventDefault(); restoreFocus(); }
      return;
    }
    if (event.repeat || !visible()) return;
    if (designSystem.matchesShortcut("shell.maximize_detail", event)) {
      event.preventDefault(); event.stopPropagation(); toggleExpanded();
    } else if (designSystem.matchesShortcut("shell.toggle_detail", event)) {
      event.preventDefault(); event.stopPropagation(); cancelResize?.();
      close();
    }
  }
  function persistWidth(width: number) {
    setPreferredWidth(width);
    if (!storageKey() || isServer) return;
    try { window.localStorage.setItem(storageKey()!, String(width)); } catch { /* Resizing remains usable without storage. */ }
  }
  let suppressClick = false;
  let clickReset: number | undefined;
  function beginResize(event: PointerEvent) {
    if (!visible() || expanded() || event.button !== 0 || cancelResize) return;
    event.preventDefault(); event.stopPropagation();
    const handle = event.currentTarget as HTMLElement;
    const pointerId = event.pointerId, startX = event.clientX;
    const saved = preferredWidth(), start = panelWidth();
    let moved = false;
    handle.focus({ preventScroll: true }); handle.setPointerCapture(pointerId);
    const move = (next: PointerEvent) => {
      if (next.pointerId !== pointerId) return;
      const dx = next.clientX - startX;
      if (!moved && Math.abs(dx) < 3) return;
      moved = true; setGesturing(true);
      setPreferredWidth(clampDetailPanelWidth(start - dx, hostWidth()));
    };
    const finish = (commit: boolean) => {
      handle.removeEventListener("pointermove", move); handle.removeEventListener("pointerup", up);
      handle.removeEventListener("pointercancel", cancel); handle.removeEventListener("lostpointercapture", cancel);
      cancelResize = undefined;
      if (handle.hasPointerCapture(pointerId)) handle.releasePointerCapture(pointerId);
      if (moved) {
        if (commit) persistWidth(panelWidth()); else setPreferredWidth(saved);
        suppressClick = true;
        if (clickReset !== undefined) cancelAnimationFrame(clickReset);
        clickReset = requestAnimationFrame(() => { suppressClick = false; clickReset = undefined; });
      }
      setGesturing(false);
    };
    const up = (next: PointerEvent) => { if (next.pointerId === pointerId) finish(true); };
    const cancel = (next: PointerEvent) => { if (next.pointerId === pointerId) finish(false); };
    cancelResize = () => finish(false);
    handle.addEventListener("pointermove", move); handle.addEventListener("pointerup", up);
    handle.addEventListener("pointercancel", cancel); handle.addEventListener("lostpointercapture", cancel);
  }
  function resizeByKey(event: KeyboardEvent) {
    if (expanded() || !visible() || cancelResize || event.altKey || event.ctrlKey || event.metaKey) return;
    const step = event.shiftKey ? 64 : 16;
    const next = event.key === "ArrowLeft" ? panelWidth() + step : event.key === "ArrowRight" ? panelWidth() - step
      : event.key === "Home" ? DETAIL_PANEL_MIN_WIDTH : event.key === "End" ? DETAIL_PANEL_MAX_WIDTH : undefined;
    if (next === undefined) return;
    event.preventDefault(); event.stopPropagation(); persistWidth(clampDetailPanelWidth(next, hostWidth()));
  }

  // Keep the departing frame and its column in the same geometry until the exit ends.
  createEffect(() => {
    const geometry = exitGeometry();
    const width = !visible() && geometry ? clampDetailPanelWidth(geometry.width, hostWidth()) : panelWidth();
    const reserved = visible() ? !expanded() : present() && geometry !== undefined && !geometry.expanded;
    root?.style.setProperty("--detail-panel-width", `${width}px`);
    root?.style.setProperty("--detail-panel-space", reserved ? `${width + 8}px` : "0px");
  });
  createEffect(() => {
    const open = visible(), full = expanded();
    if (!root || isServer) return;
    if (!open || full) untrack(() => cancelResize?.());
    if (!open && lastOpen && !preserveOutsideFocus
      && (untrack(focusInFrame) || document.activeElement === divider)) restoreFocus();
    if (!available() && (untrack(focusInFrame) || document.activeElement === divider)) restoreFocus();
    preserveOutsideFocus = false;
    const changed = open !== lastOpen || full !== lastExpanded;
    const previousExpanded = lastExpanded;
    lastOpen = open; lastExpanded = full;
    if (!available()) {
      stopFrame(); resetOutsideGesture(); offset.set(0); opacity.set(0);
      batch(() => { setPresent(false); setExitGeometry(undefined); });
      return;
    }
    if (!changed) return;
    stopFrame();
    if (open) {
      batch(() => { setPresent(true); setExitGeometry(undefined); });
      if (!ready) { offset.set(0); opacity.set(1); }
      else if (prefersReducedMotion()) {
        offset.set(0);
        frameAnimation = [animate(opacity, 1, spring.fast)];
      } else {
        if (opacity.get() === 0) offset.set(16);
        frameAnimation = [animate(offset, 0, spring.slow), animate(opacity, 1, spring.slow)];
      }
    } else {
      setExitGeometry({ width: untrack(panelWidth), expanded: previousExpanded });
      const currentRun = run;
      const reduced = prefersReducedMotion();
      const finish = () => {
        if (disposed || currentRun !== run || visible()) return;
        stopFrame(); offset.set(reduced ? 0 : 16); opacity.set(0);
        batch(() => { setPresent(false); setExitGeometry(undefined); });
      };
      if (!ready) finish();
      else {
        const transition = { type: "tween" as const, ...(reduced ? spring.fast.exit : spring.slow.exit) };
        if (reduced) offset.set(0);
        frameAnimation = [
          ...(!reduced ? [animate(offset, 16, transition)] : []),
          animate(opacity, 0, transition),
        ];
        void Promise.all(frameAnimation.map((animation) => animation.finished)).then(finish, finish);
        timer = window.setTimeout(finish, transition.duration * 1000);
      }
    }
    paint();
  });

  onMount(() => {
    if (!root) return;
    const unregister = registerPanelInteraction(interaction);
    if (frame) interaction.elements.add(frame);
    if (divider) interaction.elements.add(divider);
    const stops = [offset, opacity].map((value) => value.on("change", paint));
    const measure = () => {
      if (!workspace) return;
      const width = workspace.getBoundingClientRect().width;
      if (width !== hostWidth()) cancelResize?.();
      setHostWidth(width);
    };
    measure(); paint();
    const observer = typeof ResizeObserver === "undefined" ? undefined : new ResizeObserver(measure);
    if (workspace) observer?.observe(workspace);
    const readyFrame = requestAnimationFrame(() => { ready = true; });
    const cancelOnBlur = () => { cancelResize?.(); cancelOutsidePress(); };
    const rememberFocus = (event: FocusEvent) => rememberPanelInteraction(event.target);
    document.addEventListener("pointerdown", trackOutsidePress, true);
    document.addEventListener("pointermove", trackOutsideMove, true);
    document.addEventListener("pointerup", releaseOutsidePress, true);
    document.addEventListener("pointercancel", cancelOutsidePress, true);
    document.addEventListener("lostpointercapture", cancelOutsidePress, true);
    document.addEventListener("scroll", trackOutsideScroll, true);
    document.addEventListener("wheel", trackOutsideScroll, { capture: true, passive: true });
    document.addEventListener("click", onOutsideClick, true);
    document.addEventListener("focusin", rememberFocus);
    window.addEventListener("keydown", onKeyDown); window.addEventListener("resize", measure); window.addEventListener("blur", cancelOnBlur);
    const swallow = (event: MouseEvent) => { if (suppressClick) { event.preventDefault(); event.stopImmediatePropagation(); } };
    root.addEventListener("click", swallow, true);
    onCleanup(() => {
      disposed = true; cancelResize?.(); resetOutsideGesture(); stopFrame(); unregister(); observer?.disconnect();
      cancelAnimationFrame(readyFrame);
      if (clickReset !== undefined) cancelAnimationFrame(clickReset);
      stops.forEach((stop) => stop());
      [offset, opacity].forEach((value) => value.stop());
      document.removeEventListener("pointerdown", trackOutsidePress, true);
      document.removeEventListener("pointermove", trackOutsideMove, true);
      document.removeEventListener("pointerup", releaseOutsidePress, true);
      document.removeEventListener("pointercancel", cancelOutsidePress, true);
      document.removeEventListener("lostpointercapture", cancelOutsidePress, true);
      document.removeEventListener("scroll", trackOutsideScroll, true);
      document.removeEventListener("wheel", trackOutsideScroll, true);
      document.removeEventListener("click", onOutsideClick, true);
      document.removeEventListener("focusin", rememberFocus);
      window.removeEventListener("keydown", onKeyDown); window.removeEventListener("resize", measure); window.removeEventListener("blur", cancelOnBlur);
      root?.removeEventListener("click", swallow, true);
    });
  });
  const context: DetailPanelContextValue = { expanded, open: visible, available, activeEntityId, detailHost, setHeaderClass, collectionHeaderHost, interaction };
  return <DetailPanelContext.Provider value={context}>
    <div ref={root} tabIndex={-1} class={`pw-detail-panel-root ${props.class ?? ""}`}
      data-state={!available() ? "unavailable" : expanded() ? "expanded" : visible() ? "open" : "closed"}
      data-open={visible()} data-expanded={expanded()} data-gesturing={gesturing() ? "true" : undefined}>
      <div ref={setCollectionHeaderHost} class="pw-detail-panel__collection-header" />
      <div ref={workspace} class="pw-detail-panel__workspace">
        <div class="pw-detail-panel__collection-panel">{props.children}</div>
        <section ref={frame} id={DETAIL_PANEL_SIDEBAR_ID} class="pw-detail-panel__frame surface-panel"
          data-expanded={visible() ? expanded() : present() && exitGeometry()?.expanded === true}
          hidden={!present() || !available()} inert={!visible() ? true : undefined} aria-hidden={!visible() ? true : undefined}
          role="region" aria-label={props["aria-label"] ?? "Details"}
          onPointerDown={(event) => event.stopPropagation()} onWheel={(event) => event.stopPropagation()}>
          <header class={`pw-detail-panel__header ${headerClass()}`}>
            <Button variant="icon" aria-label="Close details" title={`Close details (${closeShortcut()})`} onClick={() => close()}>
              <X size={14} />
            </Button>
            <Button variant="icon" aria-label={expandLabel()} aria-pressed={expanded()} title={`${expandLabel()} (${expandShortcut()})`} onClick={toggleExpanded}>
              <Show when={expanded()} fallback={<ExpandIcon size={14} />}><CollapseIcon size={14} /></Show>
            </Button>
          </header>
          <div ref={setDetailHost} class="pw-detail-panel__surface" />
        </section>
        <div ref={divider} class="pw-detail-panel__resize-handle" hidden={!visible() || expanded()} role="separator" tabIndex={visible() && !expanded() ? 0 : -1}
          aria-label="Resize details" aria-orientation="vertical" aria-controls={DETAIL_PANEL_SIDEBAR_ID}
          aria-valuenow={Math.round(panelWidth())} aria-valuemin={Math.min(DETAIL_PANEL_MIN_WIDTH, panelWidth())}
          aria-valuemax={clampDetailPanelWidth(DETAIL_PANEL_MAX_WIDTH, hostWidth())}
          onPointerDown={beginResize} onKeyDown={resizeByKey} onDblClick={() => persistWidth(clampDetailPanelWidth(DETAIL_PANEL_DEFAULT_WIDTH, hostWidth()))} />
      </div>
    </div>
  </DetailPanelContext.Provider>;
}

function readStoredWidth(props: Pick<DetailPanelRootProps, "storageKey" | "legacyStorageKey">) {
  if (isServer) return DETAIL_PANEL_DEFAULT_WIDTH;
  try {
    const raw = props.storageKey ? window.localStorage.getItem(props.storageKey) : null;
    return readDetailPanelWidth(raw ?? (props.legacyStorageKey ? window.localStorage.getItem(props.legacyStorageKey) : null));
  } catch { return DETAIL_PANEL_DEFAULT_WIDTH; }
}
function DetailPanelCollection(props: { children: JSX.Element; class?: string }) {
  const context = useDetailPanel("Collection");
  return <div inert={context.expanded() ? true : undefined} class={`pw-detail-panel__collection ${props.class ?? ""}`}>{props.children}</div>;
}
function DetailPanelHeader(props: DetailPanelHeaderProps) {
  const context = useDetailPanel("Header");
  createEffect(() => context.setHeaderClass(props.class ?? ""));
  onCleanup(() => context.setHeaderClass(""));
  return null;
}

/** Entity identity belongs inside the scrollable detail content. */
function DetailPanelTitle(props: DetailPanelTitleProps) {
  return (
    <div class="pw-detail-panel__identity">
      <Show when={props.icon}>
        <span class="pw-detail-panel__header-icon" aria-hidden>{props.icon}</span>
      </Show>
      <h2 class="pw-detail-panel__title">{props.title}</h2>
      <Show when={props.meta}>
        <div class="pw-detail-panel__meta">{props.meta}</div>
      </Show>
    </div>
  );
}

function DetailPanelBody(props: DetailPanelBodyProps) {
  useDetailPanel("Body");
  const layout = () => props.layout ?? "record";
  return (
    <div
      class={`pw-detail-panel__body ${props.class ?? ""}`}
      data-layout={layout()}
    >
      <Show when={layout() === "record"} fallback={props.children}>
        <div class="pw-detail-panel__record">{props.children}</div>
      </Show>
    </div>
  );
}

function DetailPanelMain(props: { children: JSX.Element; class?: string }) {
  return (
    <div class={`pw-detail-panel__main ${props.class ?? ""}`}>{props.children}</div>
  );
}

function DetailPanelRail(props: { children: JSX.Element; class?: string }) {
  return (
    <aside class={`pw-detail-panel__rail ${props.class ?? ""}`}>{props.children}</aside>
  );
}

function DetailPanelSection(props: DetailPanelSectionProps) {
  const titleId = createUniqueId();
  return (
    <section
      class={`pw-detail-panel__section ${props.class ?? ""}`}
      aria-labelledby={props.title ? titleId : undefined}
    >
      <Show when={props.title || props.aside}>
        <div class="pw-detail-panel__section-heading">
          <h3 id={titleId} class="pw-detail-panel__section-title">
            {props.title}
          </h3>
          <Show when={props.aside}>{props.aside}</Show>
        </div>
      </Show>
      {props.children}
    </section>
  );
}

function DetailPanelContent(props: { children: JSX.Element; class?: string }) {
  const context = useDetailPanel("Content");
  return (
    <Show when={context.detailHost()}>
      {(mount) => (
        <Portal
          mount={mount()}
          ref={(container) => container.classList.add("pw-detail-panel__portal")}
        >
          <PanelInteractionContext.Provider value={context.interaction}>
            <div class={`pw-detail-panel__content ${props.class ?? ""}`}>{props.children}</div>
          </PanelInteractionContext.Provider>
        </Portal>
      )}
    </Show>
  );
}

export const DetailPanel = {
  Root: DetailPanelRoot,
  Collection: DetailPanelCollection,
  CollectionHeader: DetailPanelCollectionHeader,
  Header: DetailPanelHeader,
  Title: DetailPanelTitle,
  PrimaryAction: DetailPanelPrimaryAction,
  MenuItem: DetailPanelMenuItem,
  Body: DetailPanelBody,
  Main: DetailPanelMain,
  Rail: DetailPanelRail,
  Section: DetailPanelSection,
  Properties: DetailPanelProperties,
  Property: DetailPanelProperty,
  LinkedEntities: DetailPanelLinkedEntities,
  Content: DetailPanelContent,
};
