import { For, Show, type JSX } from "solid-js";
/**
 * Section navigation tabs: **Ark UI Tabs** (behavior) + pill CSS (look).
 * Public API: items / value / onChange — preserved for features.
 * **Not** for alternate collection views — operational collections use DataTable only.
 */
import { Tabs as ArkTabs } from "@ark-ui/solid/tabs";
import {
  FloatingCountBadge,
  hasFloatingCount,
  type FloatingCountBadgeTone,
} from "./floating-count-badge";
import type { HugeiconComponent } from "./hugeicons";
import { Tooltip } from "./tooltip";
import "./tabs.css";
import "./tooltip.css";

/** Hold briefly so scanning tabs does not flash rich copy. */
const TAB_TOOLTIP_OPEN_DELAY = 400;
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
}) {
  const classList = () => ({
    "pw-tab": true,
    "pw-tab--active": props.active,
    "pw-tab--closable": props.closable,
    "pw-tab--disabled": Boolean(props.item.disabled),
    "pw-floating-count-host": hasFloatingCount(props.item.count),
  });

  const label = (
    <>
      {props.item.label}
      <Show when={hasFloatingCount(props.item.count)}>
        <FloatingCountBadge
          count={props.item.count!}
          ariaLabel={props.item.countAriaLabel}
          tone={props.item.countTone}
        />
      </Show>
    </>
  );

  const tooltip = props.item.tooltip;
  if (!tooltip) {
    return (
      <ArkTabs.Trigger
        value={props.item.id}
        disabled={props.item.disabled}
        id={`tab-${props.item.id}`}
        classList={classList()}
      >
        {label}
      </ArkTabs.Trigger>
    );
  }

  return (
    <Tooltip.Root
      placement="bottom-start"
      disabled={props.item.disabled}
      openDelay={TAB_TOOLTIP_OPEN_DELAY}
    >
      <Tooltip.Trigger
        asChild={(triggerProps) => (
          <ArkTabs.Trigger
            {...triggerProps()}
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
        <TabRichTooltipContent tooltip={tooltip} />
      </Tooltip.Content>
    </Tooltip.Root>
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
          class={`pw-tabs ${props.class ?? ""}`}
          aria-label={props["aria-label"] ?? "Page sections"}
        >
          <For each={props.items}>
            {(item) => {
              const active = () => props.value === item.id;
              return (
                <div class="pw-tab-item">
                  <TabTrigger
                    item={item}
                    active={active()}
                    closable={Boolean(item.closable && props.onClose)}
                  />
                  <Show when={item.closable && props.onClose}>
                    <button
                      type="button"
                      class="pw-tab-close"
                      aria-label={`Close ${item.label}`}
                      onClick={() => props.onClose?.(item.id)}
                    >
                      ×
                    </button>
                  </Show>
                </div>
              );
            }}
          </For>
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
