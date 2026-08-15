import { Search } from "./hugeicons";
import { Input } from "./input";
import type { ControlFieldSize } from "./input";

export type SearchBarProps = {
  placeholder: string;
  value: string;
  onInput: (value: string) => void;
  autofocus?: boolean;
  size?: ControlFieldSize;
  class?: string;
  "aria-label"?: string;
};

/** Search control for PageToolbar slots; defaults to the 28px toolbar density. */
export function SearchBar(props: SearchBarProps) {
  return (
    <label
      class={`relative block w-[240px] max-w-[28vw] min-w-0 ${props.class ?? ""}`}
    >
      <span class="sr-only">{props["aria-label"] ?? props.placeholder}</span>
      <Search
        size={14}
        class="pointer-events-none absolute top-1/2 left-3.5 z-[1] -translate-y-1/2 text-[var(--color-text-muted)]"
        aria-hidden
      />
      <Input
        size={props.size ?? "sm"}
        class="!rounded-full !pr-3.5 !pl-10"
        type="search"
        role="searchbox"
        placeholder={props.placeholder}
        value={props.value}
        onInput={(event) => props.onInput(event.currentTarget.value)}
        autofocus={props.autofocus}
        autocomplete="off"
        spellcheck={false}
        aria-label={props["aria-label"] ?? props.placeholder}
      />
    </label>
  );
}
