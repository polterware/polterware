import { Check } from "./hugeicons";
import type { JSX } from "solid-js";
import { Show, splitProps } from "solid-js";
import "./checkbox-row.css";

export type CheckboxRowProps = {
  checked: boolean;
  onCheckedChange: (checked: boolean) => void;
  label: JSX.Element;
  disabled?: boolean;
  name?: string;
  value?: string;
  type?: "checkbox" | "radio";
  leading?: JSX.Element;
  trailing?: JSX.Element;
  class?: string;
  onClick?: (event: MouseEvent) => void;
  ariaLabel?: string;
  "aria-label"?: string;
};

/**
 * Shared controlled CheckboxRow primitive for popovers, menus, and selection lists.
 */
export function CheckboxRow(props: CheckboxRowProps) {
  const [local, rest] = splitProps(props, [
    "checked",
    "onCheckedChange",
    "label",
    "disabled",
    "name",
    "value",
    "type",
    "leading",
    "trailing",
    "class",
    "onClick",
    "ariaLabel",
    "aria-label",
  ]);

  return (
    <label
      class={`pw-checkbox-row ${local.class ?? ""}`}
      classList={{ "pw-checkbox-row--disabled": local.disabled }}
    >
      <input
        class="peer sr-only"
        type={local.type ?? "checkbox"}
        name={local.name}
        value={local.value}
        checked={local.checked}
        disabled={local.disabled}
        aria-label={local.ariaLabel ?? local["aria-label"]}
        onClick={(event) => {
          local.onClick?.(event);
        }}
        onChange={(event) => {
          local.onCheckedChange(event.currentTarget.checked);
        }}
        {...rest}
      />
      <span class="pw-checkbox-row__check" aria-hidden="true">
        <Show when={local.checked}>
          <Check size={12} stroke-width={2.5} />
        </Show>
      </span>
      <Show when={local.leading}>
        <span class="pw-checkbox-row__leading">{local.leading}</span>
      </Show>
      <span class="pw-checkbox-row__label">{local.label}</span>
      <Show when={local.trailing}>
        <span class="pw-checkbox-row__trailing">{local.trailing}</span>
      </Show>
    </label>
  );
}
