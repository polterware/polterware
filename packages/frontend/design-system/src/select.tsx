import { createSignal, Index, onCleanup, Show, type JSX } from "solid-js";
import { ChevronDown } from "./hugeicons";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "./dropdown-menu";
import type { ControlFieldSize } from "./input";
import { SELECTION_ACK_MS, SelectionCheck } from "./menu-highlight";

function cn(...parts: Array<string | false | null | undefined>) {
  return parts.filter(Boolean).join(" ");
}

/**
 * Product select — Ark Menu behavior with the shared control-field trigger and
 * menu-panel surface. Use a searchable design-system picker for large datasets.
 * A pick holds the menu open for `SELECTION_ACK_MS` so the marker glide and the check
 * drawing in are seen; Escape, outside press, and the trigger close at once.
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

  const [open, setOpen] = createSignal(false);
  let acknowledgeTimer: ReturnType<typeof setTimeout> | undefined;
  const clearAcknowledge = () => {
    if (acknowledgeTimer !== undefined) clearTimeout(acknowledgeTimer);
    acknowledgeTimer = undefined;
  };
  onCleanup(clearAcknowledge);

  const select = (nextValue: string) => {
    if (props.value === undefined) setUncontrolledValue(nextValue);
    props.onChange?.(nextValue);
    clearAcknowledge();
    acknowledgeTimer = setTimeout(() => {
      acknowledgeTimer = undefined;
      setOpen(false);
    }, SELECTION_ACK_MS);
  };

  return (
    <DropdownMenu
      open={open()}
      onOpenChange={(next) => {
        clearAcknowledge();
        setOpen(next);
      }}
    >
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
        <Index each={props.options}>
          {(option) => (
            <>
              <Show when={option().separatorBefore}>
                <DropdownMenuSeparator />
              </Show>
              <DropdownMenuItem
                value={option().value}
                disabled={option().disabled}
                selected={option().value === value()}
                closeOnSelect={false}
                class="justify-between"
                onSelect={() => select(option().value)}
              >
                <span class="min-w-0 truncate">{option().label}</span>
                <SelectionCheck checked={option().value === value()} />
              </DropdownMenuItem>
            </>
          )}
        </Index>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
