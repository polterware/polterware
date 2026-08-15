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
import "./tabs.css";

export type TabItem = {
  id: string;
  label: string;
  count?: number;
  countAriaLabel?: string;
  countTone?: FloatingCountBadgeTone;
  disabled?: boolean;
  closable?: boolean;
};

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
      <ArkTabs.List
        class={`pw-tabs ${props.class ?? ""}`}
        aria-label={props["aria-label"] ?? "Page sections"}
      >
        <For each={props.items}>
          {(item) => {
            const active = () => props.value === item.id;
            return (
              <div class="pw-tab-item">
                <ArkTabs.Trigger
                  value={item.id}
                  disabled={item.disabled}
                  id={`tab-${item.id}`}
                  classList={{
                    "pw-tab": true,
                    "pw-tab--active": active(),
                    "pw-tab--closable": Boolean(item.closable && props.onClose),
                    "pw-tab--disabled": Boolean(item.disabled),
                    "pw-floating-count-host": hasFloatingCount(item.count),
                  }}
                >
                  {item.label}
                  <Show when={hasFloatingCount(item.count)}>
                    <FloatingCountBadge
                      count={item.count!}
                      ariaLabel={item.countAriaLabel}
                      tone={item.countTone}
                    />
                  </Show>
                </ArkTabs.Trigger>
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
