import type { JSX } from "solid-js";
import { Show } from "solid-js";
import "./field.css";

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
    <label class={`pw-field ${props.class ?? ""}`} for={props.for}>
      <span class="pw-field__label">{props.label}</span>
      {props.children}
      <Show when={props.error}>
        <span class="pw-field__error">{props.error}</span>
      </Show>
      <Show when={props.hint && !props.error}>
        <span class="pw-field__hint">{props.hint}</span>
      </Show>
    </label>
  );
}
