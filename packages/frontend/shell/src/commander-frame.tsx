import type { JSX } from "solid-js";
import { Dialog } from "@polterware/frontend-design-system";
import { cx } from "./classes";
import "./commander.css";

export function CommanderFrame(props: {
  open: boolean;
  title: string;
  description?: string;
  onClose: () => void;
  children: JSX.Element;
  class?: string;
}) {
  return (
    <Dialog
      open={props.open}
      title={props.title}
      onClose={props.onClose}
      description={props.description}
      class={props.class}
    >
      {props.children}
    </Dialog>
  );
}

export function CommanderBody(props: { children: JSX.Element; class?: string; ref?: HTMLDivElement | ((el: HTMLDivElement) => void); onKeyDown?: JSX.EventHandlerUnion<HTMLDivElement, KeyboardEvent> }) {
  return (
    <div ref={props.ref} onKeyDown={props.onKeyDown} class={cx("pw-commander-body", props.class)}>
      {props.children}
    </div>
  );
}

const searchInputClass =
  "pw-popover-search-input pw-commander-search-input w-full min-w-0 border-0 bg-transparent text-[13px] text-[var(--color-text-primary)] placeholder:text-[var(--color-text-disabled)]";

export function CommanderSearchField(props: {
  placeholder: string;
  value: string;
  onInput: (value: string) => void;
  autofocus?: boolean;
  "aria-label"?: string;
}) {
  return (
    <div class="shrink-0 border-b border-[var(--color-border-subtle)] px-3 py-2.5">
      <input
        class={searchInputClass}
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

export function PopoverSearchField(props: {
  placeholder: string;
  value: string;
  onInput: (value: string) => void;
  autofocus?: boolean;
  "aria-label"?: string;
}) {
  return <CommanderSearchField {...props} />;
}

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
        <p class="m-0 mb-2 text-[12px] font-medium text-[var(--color-text-primary)]">{props.title}</p>
      ) : null}
      <input
        class={searchInputClass}
        type="text"
        role="searchbox"
        placeholder={props.placeholder}
        value={props.value}
        onInput={(event) => props.onInput(event.currentTarget.value)}
        autofocus={props.autofocus}
        autocomplete="off"
        spellcheck={false}
        aria-label={props.placeholder}
      />
      {props.children}
    </div>
  );
}
