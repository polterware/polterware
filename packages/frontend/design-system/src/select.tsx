import { createSignal, For, Show, type JSX } from "solid-js";
import { Check, ChevronDown } from "./hugeicons";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "./dropdown-menu";
import type { ControlFieldSize } from "./input";

function cn(...parts: Array<string | false | null | undefined>) {
  return parts.filter(Boolean).join(" ");
}

/**
 * Product select — Ark Menu behavior with the shared control-field trigger and
 * menu-panel surface. Use a searchable design-system picker for large datasets.
 */
export type SelectOption = {
  value: string;
  label: string;
  disabled?: boolean;
  separatorBefore?: boolean;
};

export type SelectProps = {
  options: readonly SelectOption[];
  value?: string;
  defaultValue?: string;
  onChange?: (value: string) => void;
  size?: ControlFieldSize;
  class?: string;
  disabled?: boolean;
  id?: string;
  title?: string;
  placeholder?: string;
  icon?: JSX.Element;
  "aria-label"?: string;
};

export function Select(props: SelectProps) {
  const [uncontrolledValue, setUncontrolledValue] = createSignal(props.defaultValue);
  const size = () => props.size ?? "sm";
  const value = () =>
    props.value ?? uncontrolledValue() ?? props.options[0]?.value ?? "";
  const selectedOption = () =>
    props.options.find((option) => option.value === value());

  const select = (nextValue: string) => {
    if (props.value === undefined) setUncontrolledValue(nextValue);
    props.onChange?.(nextValue);
  };

  return (
    <DropdownMenu>
      <DropdownMenuTrigger
        disabled={props.disabled}
        id={props.id}
        title={props.title}
        aria-label={props["aria-label"]}
        class={cn(
          "control-field w-full min-w-0 justify-between gap-2 text-[var(--color-text-primary)] focus-visible:outline-none",
          size() === "md" ? "control-field-md" : "control-field-sm",
          props.disabled ? "cursor-not-allowed" : "cursor-pointer",
          props.class,
        )}
      >
        <Show when={props.icon}>
          <span
            class="inline-flex shrink-0 items-center text-[var(--color-text-muted)]"
            aria-hidden
          >
            {props.icon}
          </span>
        </Show>
        <span class="min-w-0 truncate">
          {selectedOption()?.label ?? props.placeholder ?? "Select"}
        </span>
        <ChevronDown
          size={14}
          class="shrink-0 text-[var(--color-text-muted)]"
          aria-hidden
        />
      </DropdownMenuTrigger>
      <DropdownMenuContent
        class="w-[var(--reference-width)] max-w-[calc(100vw-24px)] px-1"
      >
        <For each={props.options}>
          {(option) => (
            <>
              <Show when={option.separatorBefore}>
                <DropdownMenuSeparator />
              </Show>
              <DropdownMenuItem
                value={option.value}
                disabled={option.disabled}
                class="justify-between"
                onSelect={() => select(option.value)}
              >
                <span class="min-w-0 truncate">{option.label}</span>
                <Show when={option.value === value()}>
                  <Check
                    size={14}
                    class="shrink-0 text-[var(--color-text-secondary)]"
                    aria-hidden
                  />
                </Show>
              </DropdownMenuItem>
            </>
          )}
        </For>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
