import { Index, Show, onCleanup, type JSX } from "solid-js";
/**
 * Section navigation tabs: **Ark UI Tabs** (behavior) + fluid pills (look).
 * One active pill glides between tabs on `spring.moderate`; one hover pill is born at the
 * active pill, follows the pointer or keyboard focus on `spring.fast`, and returns to the
 * active pill as it fades. Labels change weight without reflow.
 * Public API: items / value / onChange — preserved for features.
 * **Not** for alternate collection views — operational collections use DataTable only.
 */
import { Tabs as ArkTabs } from "@ark-ui/solid/tabs";
import { createFluidHover, FluidHoverHighlight } from "./fluid-hover";
import {
  FloatingCountBadge,
  hasFloatingCount,
  type FloatingCountBadgeTone,
} from "./floating-count-badge";
import type { HugeiconComponent } from "./hugeicons";
import { spring } from "./springs";
import { Tooltip } from "./tooltip";
import "./tabs.css";
import "./tooltip.css";

/** Hold briefly so scanning tabs does not flash rich copy. */
const TAB_TOOLTIP_OPEN_DELAY = 400;
/** A click in the gap between tabs lands on the lit tab only this close to it (px). */
const TAB_GAP_CLICK_DISTANCE = 8;

export type TabTooltip = {
  title: string;
  description: string;
  icon: HugeiconComponent;
};

export type TabItem = {
  id: string;
  label: string;
  count?: number;
  countAriaLabel?: string;
  countTone?: FloatingCountBadgeTone;
  disabled?: boolean;
  closable?: boolean;
  tooltip?: TabTooltip;
};

function TabRichTooltipContent(props: { tooltip: TabTooltip }) {
  const Icon = props.tooltip.icon;
  return (
    <div class="pw-tab-tooltip">
      <div class="pw-tab-tooltip__header">
        <Icon size={14} class="pw-tab-tooltip__icon" />
        <span class="pw-tab-tooltip__title">{props.tooltip.title}</span>
      </div>
      <p class="pw-tab-tooltip__description">{props.tooltip.description}</p>
    </div>
  );
}

function TabTrigger(props: {
  item: TabItem;
  active: boolean;
  closable: boolean;
  ref: (el: HTMLElement) => void;
}) {
  const classList = () => ({
    "pw-tab": true,
    "pw-tab--active": props.active,
    "pw-tab--closable": props.closable,
    "pw-tab--disabled": Boolean(props.item.disabled),
    "pw-floating-count-host": hasFloatingCount(props.item.count),
  });

  // Two stacked copies: the hidden one reserves the active weight's width, so the
  // visible label can change weight without moving its neighbours.
  const label = (
    <>
      <span class="pw-tab__label">
        <span class="pw-tab__label-sizer" aria-hidden="true">
          {props.item.label}
        </span>
        <span class="pw-tab__label-text">{props.item.label}</span>
      </span>
      <Show when={hasFloatingCount(props.item.count)}>
        <FloatingCountBadge
          count={props.item.count!}
          ariaLabel={props.item.countAriaLabel}
          tone={props.item.countTone}
          class="pw-tab__count"
        />
      </Show>
    </>
  );

  return (
    <Show
      when={props.item.tooltip}
      fallback={
        <ArkTabs.Trigger
          ref={props.ref}
          value={props.item.id}
          disabled={props.item.disabled}
          id={`tab-${props.item.id}`}
          classList={classList()}
        >
          {label}
        </ArkTabs.Trigger>
      }
    >
      {(tooltip) => (
        <Tooltip.Root
          placement="bottom-start"
          disabled={props.item.disabled}
          openDelay={TAB_TOOLTIP_OPEN_DELAY}
        >
          <Tooltip.Trigger
            asChild={(triggerProps) => (
              <ArkTabs.Trigger
                {...triggerProps()}
                ref={props.ref}
                value={props.item.id}
                disabled={props.item.disabled}
                id={`tab-${props.item.id}`}
                classList={classList()}
              >
                {label}
              </ArkTabs.Trigger>
            )}
          />
          <Tooltip.Content class="pw-tooltip-content--rich">
            <TabRichTooltipContent tooltip={tooltip()} />
          </Tooltip.Content>
        </Tooltip.Root>
      )}
    </Show>
  );
}

export function Tabs(props: {
  items: TabItem[];
  value: string;
  onChange: (id: string) => void;
  onClose?: (id: string) => void;
  class?: string;
  "aria-label"?: string;
}) {
  let list: HTMLDivElement | undefined;
  const hover = createFluidHover(() => list, {
    // Tabs wrap on narrow toolbars, so pick across rows too.
    axis: "xy",
    isItemDisabled: (element) => element.hasAttribute("data-disabled"),
    gapClick: { maxDistance: TAB_GAP_CLICK_DISTANCE },
  });

  const selectedIndex = () => props.items.findIndex((item) => item.id === props.value);
  const selectedRect = () => {
    const index = selectedIndex();
    return index < 0 ? null : (hover.rects()[index] ?? null);
  };
  // The hover pill hides over the selected tab; the active pill already marks it.
  const hoverIndex = () => {
    const index = hover.activeIndex();
    return index === null || index === selectedIndex() ? null : index;
  };
  const hoverRect = () => {
    const index = hoverIndex();
    return index === null || !hover.isMeasured() ? null : (hover.rects()[index] ?? null);
  };

  return (
    <ArkTabs.Root
      value={props.value}
      onValueChange={(details) => {
        if (details.value) props.onChange(details.value);
      }}
      orientation="horizontal"
      activationMode="manual"
    >
      <Tooltip.Group>
        <ArkTabs.List
          ref={list}
          class={`pw-tabs ${props.class ?? ""}`}
          aria-label={props["aria-label"] ?? "Page sections"}
          {...hover.handlers}
        >
          <FluidHoverHighlight
            class="pw-tab-indicator"
            rect={selectedRect()}
            targetKey={selectedIndex()}
            transition={spring.moderate}
          />
          <FluidHoverHighlight
            class="pw-tab-hover"
            rect={hoverRect()}
            targetKey={hoverIndex()}
            session={hover.session()}
            from={selectedRect()}
            // Leaving the list sends the pill home to the selected tab as it fades.
            exitTo={hover.activeIndex() === null ? selectedRect() : null}
          />
          <Index each={props.items}>
            {(item, index) => (
              <div class="pw-tab-item">
                <TabTrigger
                  ref={(el) => onCleanup(hover.registerItem(index, el))}
                  item={item()}
                  active={props.value === item().id}
                  closable={Boolean(item().closable && props.onClose)}
                />
                <Show when={item().closable && props.onClose}>
                  <button
                    type="button"
                    class="pw-tab-close"
                    aria-label={`Close ${item().label}`}
                    onClick={() => props.onClose?.(item().id)}
                  >
                    ×
                  </button>
                </Show>
              </div>
            )}
          </Index>
        </ArkTabs.List>
      </Tooltip.Group>
    </ArkTabs.Root>
  );
}

/**
 * Presentational panel association for section content.
 * Features own which panel is active via route/state (not Ark Tabs.Content),
 * so public TabPanel API stays simple.
 */
export function TabPanel(props: {
  id: string;
  active: boolean;
  children: JSX.Element;
  class?: string;
}) {
  return (
    <div
      role="tabpanel"
      id={`panel-${props.id}`}
      aria-labelledby={`tab-${props.id}`}
      hidden={!props.active}
      class={`min-h-0 flex-1 ${props.class ?? ""}`}
      style={{
        display: props.active ? "flex" : "none",
        "flex-direction": "column",
        "min-height": 0,
        flex: "1 1 auto",
      }}
    >
      {props.active ? props.children : null}
    </div>
  );
}
