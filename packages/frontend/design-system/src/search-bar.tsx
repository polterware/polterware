import { Input } from "./input";
import type { ControlFieldSize } from "./input";
import "./search-bar.css";

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
      class={`block w-[200px] max-w-[28vw] min-w-0 ${props.class ?? ""}`}
    >
      <span class="sr-only">{props["aria-label"] ?? props.placeholder}</span>
      <Input
        size={props.size ?? "sm"}
        class="pw-search-bar-input !rounded-full !px-3.5"
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
