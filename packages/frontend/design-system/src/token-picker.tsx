import { For, Show, createMemo, createSignal, type JSX } from "solid-js";
import { Button } from "./button";
import { ArrowUpRight01Icon, Plus, X } from "./hugeicons";
import { PropertyPicker, type PropertyOption } from "./property-picker";
import "./active-filter-bar.css";
import "./token-picker.css";

export type TokenPickerProps = {
  /** What the tokens are, such as "Context"; it names the picker and the add button. */
  label: string;
  /** Shown on the add button while nothing is picked. */
  icon?: JSX.Element;
  options: readonly PropertyOption[];
  values: readonly string[];
  onChange: (values: string[]) => void;
  /** What an empty list means, such as "Whole project"; shown only while nothing is picked. */
  emptyLabel?: string;
  disabled?: boolean;
  /** Typed text that matches no option becomes a token of its own, such as a technology. */
  creatable?: boolean;
  /** Opens a token for editing, such as a deploy in its Composer; the token's value becomes a button. */
  onEdit?: (value: string) => void;
  /** Replaces the option list: the add button calls this, such as opening a Composer for a new item. */
  onAdd?: () => void;
  /**
   * A token that leads somewhere outside, such as a deploy's page on its platform: an
   * open button between the value and remove, named by `label`.
   */
  opener?: (value: string) => { label: string; open: () => void } | undefined;
};

/**
 * A list of values edited as tokens, the way the filter menu adds them: each picked value
 * is a pill with its icon and a remove button, and the add button opens the shared
 * PropertyPicker with the values not picked yet.
 */
export function TokenPicker(props: TokenPickerProps) {
  const [open, setOpen] = createSignal(false);
  let add: HTMLButtonElement | undefined;
  const picked = createMemo(() =>
    props.values.map((value) => props.options.find((option) => option.value === value) ?? { value, label: value }),
  );
  const remaining = createMemo(() => props.options.filter((option) => !props.values.includes(option.value)));
  const empty = () => props.values.length === 0;
  return (
    <div class="pw-token-picker" role="group" aria-label={props.label}>
      <For each={picked()}>
        {(option) => (
          <span class="pw-filter-pill pw-texture-button pw-token-picker__token">
            <Show when={props.onEdit} fallback={<span class="pw-filter-pill__seg pw-filter-pill__value"><TokenContent option={option} /></span>}>
              <button type="button" class="pw-filter-pill__seg pw-filter-pill__value pw-filter-pill__button" aria-label={`Edit ${option.label}`}
                disabled={props.disabled} onClick={() => props.onEdit!(option.value)}>
                <TokenContent option={option} />
              </button>
            </Show>
            <Show when={props.opener?.(option.value)}>
              {(target) => (
                <button type="button" class="pw-filter-pill__remove pw-token-picker__open" aria-label={target().label} title={target().label}
                  onClick={() => target().open()}>
                  <ArrowUpRight01Icon size={12} aria-hidden />
                </button>
              )}
            </Show>
            <button
              type="button"
              class="pw-filter-pill__remove"
              aria-label={`Remove ${option.label}`}
              disabled={props.disabled}
              onClick={() => props.onChange(props.values.filter((value) => value !== option.value))}
            >
              <X size={12} aria-hidden />
            </button>
          </span>
        )}
      </For>
      <Show when={empty() || remaining().length > 0 || props.creatable || props.onAdd}>
        <Button
          ref={(element: HTMLButtonElement) => { add = element; }}
          size={empty() ? "sm" : "icon"}
          variant="texture"
          aria-label={empty() ? `${props.label}: ${props.emptyLabel ?? "none"}` : `Add ${props.label.toLowerCase()}`}
          aria-expanded={open()}
          disabled={props.disabled || (remaining().length === 0 && !props.creatable && !props.onAdd)}
          onClick={() => props.onAdd ? props.onAdd() : setOpen((current) => !current)}
        >
          <Show when={empty()} fallback={<Plus size={14} aria-hidden />}>
            {props.icon}
            {props.emptyLabel ?? props.label}
          </Show>
        </Button>
      </Show>
      <PropertyPicker
        label={props.label}
        options={remaining()}
        onChange={(value) => { if (!props.values.includes(value)) props.onChange([...props.values, value]); }}
        onCreate={props.creatable ? (text) => text : undefined}
        open={open() && !props.onAdd}
        onClose={() => setOpen(false)}
        anchorRef={() => add}
      />
    </div>
  );
}

/** A token's icon and label, then its hint muted, such as a deploy's state. */
function TokenContent(props: { option: { label: string; icon?: JSX.Element; hint?: string } }) {
  return <>
    <Show when={props.option.icon}>
      <span class="pw-token-picker__icon">{props.option.icon}</span>
    </Show>
    <span class="truncate">{props.option.label}</span>
    <Show when={props.option.hint}>
      <span class="pw-token-picker__hint">{props.option.hint}</span>
    </Show>
  </>;
}
