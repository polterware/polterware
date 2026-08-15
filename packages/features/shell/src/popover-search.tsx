import type { JSX } from "solid-js";
import "./popover-search.css";

/**
 * Flat search for title-bar / shell popovers (org, project, terminal, …).
 * Matches Commander: no control-field chrome — only section border-b, no focus ring.
 */
export function PopoverSearchField(props: {
  placeholder: string;
  value: string;
  onInput: (value: string) => void;
  autofocus?: boolean;
  "aria-label"?: string;
}) {
  return (
    <div class="shrink-0 border-b border-[var(--color-border-subtle)] px-3 py-2.5">
      <input
        class="pw-popover-search-input w-full min-w-0 border-0 bg-transparent text-[13px] text-[var(--color-text-primary)] placeholder:text-[var(--color-text-disabled)]"
        type="text"
        role="searchbox"
        placeholder={props.placeholder}
        value={props.value}
        onInput={(event) => props.onInput(event.currentTarget.value)}
        autofocus={props.autofocus}
        autocomplete="off"
        spellcheck={false}
        aria-label={props["aria-label"] ?? props.placeholder}
      />
    </div>
  );
}

/** Optional title row above flat search (e.g. terminal picker). */
export function PopoverSearchHeader(props: {
  title?: string;
  placeholder: string;
  value: string;
  onInput: (value: string) => void;
  autofocus?: boolean;
  children?: JSX.Element;
}) {
  return (
    <div class="shrink-0 border-b border-[var(--color-border-subtle)] px-3 py-2.5">
      {props.title ? (
        <p class="m-0 mb-2 text-[12px] font-medium text-[var(--color-text-primary)]">
          {props.title}
        </p>
      ) : null}
      <input
        class="pw-popover-search-input w-full min-w-0 border-0 bg-transparent text-[13px] text-[var(--color-text-primary)] placeholder:text-[var(--color-text-disabled)]"
        type="text"
        role="searchbox"
        placeholder={props.placeholder}
        value={props.value}
        onInput={(e) => props.onInput(e.currentTarget.value)}
        autofocus={props.autofocus}
        autocomplete="off"
        spellcheck={false}
        aria-label={props.placeholder}
      />
      {props.children}
    </div>
  );
}
