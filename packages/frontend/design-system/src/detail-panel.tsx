import {
  Show,
  createContext,
  createEffect,
  on,
  createSignal,
  createUniqueId,
  onCleanup,
  onMount,
  splitProps,
  useContext,
  type Accessor,
  type JSX,
} from "solid-js";
import { Portal, isServer } from "solid-js/web";
import { animate, motionValue } from "motion";
import { Button, type ButtonProps } from "./button";
import { spring } from "./springs";
import {
  ArrowUp,
  CollapseIcon,
  ExpandIcon,
  XIcon,
} from "./hugeicons";
import { Kbd } from "./kbd";
import { Tooltip } from "./tooltip";
import { DetailPanelLinkedEntities } from "./detail-panel-linked-entities";
import { DetailPanelOverflow, DetailPanelOverflowItem } from "./detail-panel-overflow";
import { DetailPanelPrimaryAction } from "./detail-panel-primary-action";
import { DetailPanelProperties, DetailPanelProperty } from "./detail-panel-properties";
import { createShortcutLabel, matchesShortcut } from "@polterware/frontend-state";
import "./detail-panel.css";

const DETAIL_PANEL_ACTION_TOOLTIP_DELAY = 900;

export type DetailPanelMode = "open" | "collapsed" | "maximized";

export type DetailPanelPresence = "hidden" | "visible" | "dismissing";

/** `bottom` = floating sheet over collections; `side` = right rail beside canvases. */
export type DetailPanelPlacement = "bottom" | "side";

export type DetailPanelRootProps = {
  mode: DetailPanelMode;
  placement?: DetailPanelPlacement;
  presence?: DetailPanelPresence;
  activeEntityId?: string | null;
  onCollapse: () => void;
  onMaximize: () => void;
  onRestore: () => void;
  onDismissComplete?: () => void;
  children: JSX.Element;
  class?: string;
  "aria-label"?: string;
};

export type DetailPanelActionTone = "neutral" | "primary";

export type DetailPanelBodyProps = {
  children: JSX.Element;
  class?: string;
  /** `record` = Main + optional Rail; `document` = knowledge writing canvas. */
  layout?: "record" | "document";
};

export type DetailPanelHeaderProps = {
  icon?: JSX.Element;
  title: JSX.Element;
  children?: JSX.Element;
  class?: string;
};

export type DetailPanelSectionProps = {
  children: JSX.Element;
  class?: string;
  title?: JSX.Element;
  /** Right-aligned element on the section title row. */
  aside?: JSX.Element;
};

export type DetailPanelActionProps = Omit<
  ButtonProps,
  "aria-label" | "children" | "size" | "title" | "variant"
> & {
  label: string;
  /** Optional keyboard shortcut shown in the hover tooltip. */
  shortcut?: string;
  tone?: DetailPanelActionTone;
  children: JSX.Element;
};

/** Slack past the slow spring before the fallback settles a run a stalled frame loop never ends. */
const DETAIL_PANEL_SETTLEMENT_SAFETY_MARGIN = 100;

type PanelPose = { x: number; y: number };

/**
 * The panel's resting pose for its current data attributes. The poses stay in CSS
 * (`cqh`/`cqw` geometry per mode, presence, and placement); this reads the computed
 * transform with the inline (JS-painted) transform lifted for one synchronous style read.
 */
export function readDetailPanelPose(panel: HTMLElement): PanelPose {
  const inline = panel.style.transform;
  panel.style.transform = "";
  const transform = window.getComputedStyle(panel).transform;
  panel.style.transform = inline;
  return parseTranslate(transform);
}

/** The translation of a computed `matrix()`/`matrix3d()`; `none` or unknown is the origin. */
export function parseTranslate(transform: string | null | undefined): PanelPose {
  const match = /^matrix(3d)?\(([^)]*)\)$/.exec(transform?.trim() ?? "");
  if (!match) return { x: 0, y: 0 };
  const values = match[2]!.split(",").map((value) => Number.parseFloat(value));
  const [x, y] = match[1] ? [values[12], values[13]] : [values[4], values[5]];
  return { x: Number.isFinite(x) ? x! : 0, y: Number.isFinite(y) ? y! : 0 };
}

const detailPanelModeRank = {
  collapsed: 0,
  open: 1,
  maximized: 2,
} satisfies Record<DetailPanelMode, number>;

type DetailPanelContextValue = {
  mode: Accessor<DetailPanelMode>;
  viewportMode: Accessor<DetailPanelMode>;
  activeEntityId: Accessor<string | null>;
  detailHost: Accessor<HTMLElement | undefined>;
  collapse: () => void;
  maximize: () => void;
  restore: () => void;
};

const DetailPanelContext = createContext<DetailPanelContextValue>();

function useDetailPanel(part: string): DetailPanelContextValue {
  const context = useContext(DetailPanelContext);
  if (!context) throw new Error(`DetailPanel.${part} must be used inside DetailPanel.Root`);
  return context;
}

function DetailPanelRoot(props: DetailPanelRootProps) {
  const [detailHost, setDetailHost] = createSignal<HTMLElement>();
  const [isSettling, setIsSettling] = createSignal(false);
  const [viewportMode, setViewportMode] = createSignal<DetailPanelMode>(
    props.mode,
  );
  const [detailInteractionDisabled, setDetailInteractionDisabled] = createSignal(
    props.presence !== undefined
      ? props.presence !== "visible"
      : (props.activeEntityId ?? null) === null,
  );
  const [reducedMotion, setReducedMotion] = createSignal(false);
  let rootElement: HTMLDivElement | undefined;
  let panelElement: HTMLDivElement | undefined;
  let settlementTimer: number | undefined;
  let observedMode: DetailPanelMode | undefined;
  let pendingViewportMode: DetailPanelMode | undefined;
  let activeTransitionMode: DetailPanelMode | undefined;
  let motionQuery: MediaQueryList | undefined;
  let motionRun = 0;
  let panelMounted = false;
  // The panel's travel is a spring in JS (spring.slow), painted as an inline transform.
  const panelX = motionValue(0);
  const panelY = motionValue(0);

  const mode = () => props.mode;
  const activeEntityId = () => props.activeEntityId ?? null;
  const presence = (): DetailPanelPresence =>
    props.presence ?? (activeEntityId() === null ? "hidden" : "visible");

  function clearSettlementTimer() {
    if (settlementTimer !== undefined && !isServer) {
      window.clearTimeout(settlementTimer);
    }
    settlementTimer = undefined;
  }

  function clearSettlement() {
    clearSettlementTimer();
    setIsSettling(false);
  }

  function prefersReducedMotion() {
    return reducedMotion();
  }

  function commitTransition(transitionMode: DetailPanelMode) {
    if (activeTransitionMode !== transitionMode) return;
    if (pendingViewportMode === transitionMode) {
      setViewportMode(transitionMode);
      pendingViewportMode = undefined;
    }
    activeTransitionMode = undefined;
    clearSettlement();
  }

  function paintPanel() {
    if (panelElement) {
      panelElement.style.transform = `translate3d(${panelX.get()}px, ${panelY.get()}px, 0)`;
    }
  }

  function snapPanel(pose: PanelPose) {
    panelX.stop();
    panelY.stop();
    panelX.set(pose.x);
    panelY.set(pose.y);
  }

  /** The spring reached its pose (or the fallback fired): commit the geometry it was sequencing. */
  function settleRun(run: number) {
    if (run !== motionRun) return;
    // Settle once: the spring and the fallback timer may both report.
    motionRun++;
    commitTransition(activeTransitionMode ?? mode());
    if (presence() === "dismissing") props.onDismissComplete?.();
  }

  /**
   * Springs the panel to the pose its new attributes resolve to. A new run retargets the
   * running one from where it is, velocity included (motion values carry it), so an
   * interrupted collapse reverses instead of restarting. The viewport geometry waits for
   * the run to settle, exactly as it used to wait for `transitionend`.
   */
  function runPanelMotion() {
    if (!panelElement || isServer) return;
    const target = readDetailPanelPose(panelElement);
    const run = ++motionRun;
    clearSettlementTimer();
    // Reduced motion commits its geometry in the mode and presence effects; it only snaps here.
    if (prefersReducedMotion()) {
      snapPanel(target);
      return;
    }
    activeTransitionMode = mode();
    // Before the first paint, or with nothing to travel, the pose lands and commits at once.
    if (!panelMounted || (target.x === panelX.get() && target.y === panelY.get())) {
      snapPanel(target);
      settleRun(run);
      return;
    }
    setIsSettling(true);
    const travel = Promise.all([
      animate(panelX, target.x, spring.slow).finished,
      animate(panelY, target.y, spring.slow).finished,
    ]);
    void travel.then(() => settleRun(run));
    settlementTimer = window.setTimeout(
      () => settleRun(run),
      spring.slow.duration * 1000 + DETAIL_PANEL_SETTLEMENT_SAFETY_MARGIN,
    );
  }

  function restoreFocus(onRestored?: () => void) {
    const entityId = activeEntityId();
    if (isServer) {
      onRestored?.();
      return;
    }
    queueMicrotask(() => {
      let targetElement: HTMLElement | undefined;
      if (entityId) {
        const candidates = rootElement?.querySelectorAll<HTMLElement>(
          "[data-detail-target-id], [data-row-id]",
        );
        if (candidates) {
          for (const candidate of candidates) {
            if (
              candidate.dataset.detailTargetId === entityId
              || candidate.dataset.rowId === entityId
            ) {
              targetElement = candidate;
              break;
            }
          }
        }
      }
      if (!targetElement) {
        const fallback = rootElement?.querySelector<HTMLElement>(
          "[data-detail-focus-fallback]",
        );
        if (fallback) targetElement = fallback;
      }
      if (targetElement) {
        targetElement.focus();
      }
      onRestored?.();
    });
  }

  function collapseAndRestoreFocus() {
    if (mode() === "collapsed") return;
    props.onCollapse();
    restoreFocus();
  }

  function onWindowKeyDown(event: KeyboardEvent) {
    if (event.key === "Escape" && mode() === "maximized") {
      event.preventDefault();
      props.onRestore();
      return;
    }

    if (event.repeat) return;

    if (matchesShortcut("shell.maximize_detail", event)) {
      event.preventDefault();
      event.stopPropagation();
      if (mode() === "maximized") {
        props.onRestore();
      } else {
        props.onMaximize();
      }
      return;
    }

    if (matchesShortcut("shell.toggle_detail", event)) {
      event.preventDefault();
      event.stopPropagation();
      if (mode() === "collapsed") {
        props.onRestore();
      } else {
        collapseAndRestoreFocus();
      }
    }
  }

  function onMotionPreferenceChange(event: MediaQueryListEvent) {
    setReducedMotion(event.matches);
  }

  createEffect(() => {
    const nextMode = mode();
    const reducesMotion = prefersReducedMotion();
    if (observedMode === undefined) {
      observedMode = nextMode;
      setViewportMode(nextMode);
      return;
    }
    if (nextMode === observedMode) {
      if (reducesMotion) {
        pendingViewportMode = undefined;
        activeTransitionMode = undefined;
        clearSettlement();
        setViewportMode(nextMode);
      }
      return;
    }
    observedMode = nextMode;

    if (reducesMotion) {
      pendingViewportMode = undefined;
      activeTransitionMode = undefined;
      clearSettlement();
      setViewportMode(nextMode);
      return;
    }

    const viewportRank = detailPanelModeRank[viewportMode()];
    const targetRank = detailPanelModeRank[nextMode];
    if (targetRank > viewportRank) {
      pendingViewportMode = undefined;
      setViewportMode(nextMode);
    } else if (targetRank < viewportRank) {
      pendingViewportMode = nextMode;
    } else {
      pendingViewportMode = undefined;
    }
  });

  createEffect(() => {
    const nextPresence = presence();
    const reducesMotion = prefersReducedMotion();
    if (nextPresence === "visible") {
      setDetailInteractionDisabled(false);
      return;
    }
    if (nextPresence === "hidden") {
      setDetailInteractionDisabled(true);
      return;
    }
    restoreFocus(() => {
      if (presence() !== "dismissing") return;
      setDetailInteractionDisabled(true);
      if (reducesMotion) props.onDismissComplete?.();
    });
  });

  // Every change that moves the panel's CSS pose: mode, presence, placement. Declared
  // after the mode effect, so the viewport sequencing is decided before the run starts.
  createEffect(
    on(
      () => [mode(), presence(), props.placement, prefersReducedMotion()] as const,
      () => runPanelMotion(),
    ),
  );

  onMount(() => {
    const stopX = panelX.on("change", paintPanel);
    const stopY = panelY.on("change", paintPanel);
    // The motion effect already snapped the first pose; springs start after the first paint.
    paintPanel();
    const ready = requestAnimationFrame(() => {
      panelMounted = true;
    });
    // A resized container moves the CSS pose (cqh/cqw): snap at rest, retarget mid-run.
    const resizeObserver =
      typeof ResizeObserver === "undefined" || !rootElement
        ? undefined
        : new ResizeObserver(() => {
            if (!panelElement) return;
            const target = readDetailPanelPose(panelElement);
            if (isSettling() && !prefersReducedMotion()) {
              animate(panelX, target.x, spring.slow);
              animate(panelY, target.y, spring.slow);
            } else {
              snapPanel(target);
            }
          });
    if (rootElement) resizeObserver?.observe(rootElement);
    onCleanup(() => {
      motionRun++;
      cancelAnimationFrame(ready);
      resizeObserver?.disconnect();
      stopX();
      stopY();
      panelX.stop();
      panelY.stop();
    });
  });

  onMount(() => {
    if (window.matchMedia instanceof Function) {
      motionQuery = window.matchMedia("(prefers-reduced-motion: reduce)");
      setReducedMotion(motionQuery.matches);
      motionQuery.addEventListener("change", onMotionPreferenceChange);
    }
    window.addEventListener("keydown", onWindowKeyDown);
  });

  onCleanup(() => {
    clearSettlement();
    if (isServer) return;
    motionQuery?.removeEventListener("change", onMotionPreferenceChange);
    window.removeEventListener("keydown", onWindowKeyDown);
  });

  const context: DetailPanelContextValue = {
    mode,
    viewportMode,
    activeEntityId,
    detailHost,
    collapse: collapseAndRestoreFocus,
    maximize: () => {
      if (mode() !== "maximized") props.onMaximize();
    },
    restore: () => {
      if (mode() !== "open") props.onRestore();
    },
  };

  return (
    <DetailPanelContext.Provider value={context}>
      <div
        ref={(element) => {
          rootElement = element;
        }}
        class={`pw-detail-panel-root ${props.class ?? ""}`}
        data-panel-placement={props.placement ?? "bottom"}
        data-mode={mode()}
        data-viewport-mode={viewportMode()}
        data-presence={presence()}
        data-settling={isSettling() ? "true" : "false"}
      >
        <div class="pw-detail-panel-layout">
          <div class="pw-detail-panel__collection-panel">
            {props.children}
          </div>
          <div
            ref={(element) => {
              panelElement = element;
            }}
            class="pw-detail-panel__panel"
          >
            <div class="pw-detail-panel__viewport">
              <section
                ref={setDetailHost}
                class="pw-detail-panel__surface surface-panel-float"
                inert={detailInteractionDisabled() ? true : undefined}
                aria-hidden={detailInteractionDisabled() ? true : undefined}
                role="region"
                aria-label={props["aria-label"] ?? "Details"}
              />
            </div>
          </div>
        </div>
      </div>
    </DetailPanelContext.Provider>
  );
}

function DetailPanelCollection(props: { children: JSX.Element; class?: string }) {
  const context = useDetailPanel("Collection");
  return (
    <div
      inert={context.viewportMode() === "maximized"}
      class={`pw-detail-panel__collection ${props.class ?? ""}`}
    >
      {props.children}
    </div>
  );
}

function DetailPanelHeader(props: DetailPanelHeaderProps) {
  const context = useDetailPanel("Header");
  const toggleShortcut = createShortcutLabel("shell.toggle_detail");
  const maximizeShortcut = createShortcutLabel("shell.maximize_detail");
  return (
    <header class={`pw-detail-panel__header ${props.class ?? ""}`}>
      <div class="pw-detail-panel__identity">
        <div class="pw-detail-panel__identity-content !rounded-full">
          <Show when={props.icon}>
            <span class="pw-detail-panel__header-icon" aria-hidden>
              {props.icon}
            </span>
          </Show>
          <h2 class="pw-detail-panel__title">{props.title}</h2>
        </div>
        <Show when={context.mode() === "collapsed"}>
          <button
            type="button"
            class="pw-detail-panel__collapsed-trigger"
            data-detail-panel-collapsed-trigger
            aria-label="Open details"
            onClick={context.restore}
          />
        </Show>
      </div>
      <Tooltip.Group>
        {props.children}
        <div class="pw-detail-panel__global-actions">
          <Show when={context.mode() === "collapsed"}>
            <DetailPanelAction
              label="Maximize details"
              shortcut={maximizeShortcut()}
              onClick={context.maximize}
            >
              <ExpandIcon size={14} aria-hidden />
            </DetailPanelAction>
            <DetailPanelAction
              label="Open details"
              shortcut={toggleShortcut()}
              onClick={context.restore}
            >
              <ArrowUp size={14} aria-hidden />
            </DetailPanelAction>
          </Show>
          <Show when={context.mode() === "open"}>
            <DetailPanelAction
              label="Maximize details"
              shortcut={maximizeShortcut()}
              onClick={context.maximize}
            >
              <ExpandIcon size={14} aria-hidden />
            </DetailPanelAction>
            <DetailPanelAction
              label="Collapse details"
              shortcut={toggleShortcut()}
              onClick={context.collapse}
            >
              <XIcon size={14} aria-hidden />
            </DetailPanelAction>
          </Show>
          <Show when={context.mode() === "maximized"}>
            <DetailPanelAction
              label="Restore details"
              shortcut={maximizeShortcut()}
              onClick={context.restore}
            >
              <CollapseIcon size={14} aria-hidden />
            </DetailPanelAction>
            <DetailPanelAction
              label="Collapse details"
              shortcut={toggleShortcut()}
              onClick={context.collapse}
            >
              <XIcon size={14} aria-hidden />
            </DetailPanelAction>
          </Show>
        </div>
      </Tooltip.Group>
    </header>
  );
}

const detailPanelIconControlClass =
  "pw-detail-panel__action control-field control-field-sm !rounded-full !p-0 focus-visible:outline-none";

function DetailPanelAction(props: DetailPanelActionProps) {
  useDetailPanel("Action");
  const [local, buttonProps] = splitProps(props, [
    "label",
    "shortcut",
    "tone",
    "children",
    "class",
  ]);
  const className = () => [
    detailPanelIconControlClass,
    local.class,
  ].filter(Boolean).join(" ");

  return (
    <Tooltip.Root disabled={buttonProps.disabled} openDelay={DETAIL_PANEL_ACTION_TOOLTIP_DELAY}>
      <Tooltip.Trigger
        asChild={(triggerProps) => (
          <Button
            {...triggerProps({
              ...buttonProps,
              "aria-label": local.label,
            })}
            variant="icon"
            size="icon"
            class={className()}
            data-tone={local.tone ?? "neutral"}
          >
            {local.children}
          </Button>
        )}
      />
      <Tooltip.Content class="pw-detail-panel__action-tooltip">
        <span class="pw-detail-panel__action-tooltip-label">{local.label}</span>
        <Show when={local.shortcut}>
          <Kbd>{local.shortcut}</Kbd>
        </Show>
      </Tooltip.Content>
    </Tooltip.Root>
  );
}

function DetailPanelActions(props: { children: JSX.Element; class?: string }) {
  useDetailPanel("Actions");
  return (
    <div class={`pw-detail-panel__actions ${props.class ?? ""}`}>
      {props.children}
    </div>
  );
}

function DetailPanelBody(props: DetailPanelBodyProps) {
  const context = useDetailPanel("Body");
  const layout = () => props.layout ?? "record";
  return (
    <div
      class={`pw-detail-panel__body ${props.class ?? ""}`}
      data-layout={layout()}
      hidden={context.viewportMode() === "collapsed"}
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
          <div class={`pw-detail-panel__content ${props.class ?? ""}`}>
            {props.children}
          </div>
        </Portal>
      )}
    </Show>
  );
}

export const DetailPanel = {
  Root: DetailPanelRoot,
  Collection: DetailPanelCollection,
  Header: DetailPanelHeader,
  Actions: DetailPanelActions,
  Action: DetailPanelAction,
  PrimaryAction: DetailPanelPrimaryAction,
  Overflow: DetailPanelOverflow,
  OverflowItem: DetailPanelOverflowItem,
  Body: DetailPanelBody,
  Main: DetailPanelMain,
  Rail: DetailPanelRail,
  Section: DetailPanelSection,
  Properties: DetailPanelProperties,
  Property: DetailPanelProperty,
  LinkedEntities: DetailPanelLinkedEntities,
  Content: DetailPanelContent,
};
