import type { JSX } from "solid-js";
import { Show } from "solid-js";

/**
 * Labeled form field shell. Put Input / Select / Textarea as children.
 */
export function Field(props: {
  label: string;
  children: JSX.Element;
  class?: string;
  hint?: string;
  error?: string;
  /** Associate label via htmlFor when the control has an id. */
  for?: string;
}) {
  return (
    <label class={`block space-y-1 ${props.class ?? ""}`} for={props.for}>
      <span class="text-[11px] font-medium tracking-wide text-[var(--color-text-muted)]">
        {props.label}
      </span>
      {props.children}
      <Show when={props.error}>
        <span class="block text-[11px] text-[var(--color-danger)]">{props.error}</span>
      </Show>
      <Show when={props.hint && !props.error}>
        <span class="block text-[11px] text-[var(--color-text-muted)]">{props.hint}</span>
      </Show>
    </label>
  );
}
