import type { JSX } from "solid-js";
import "./form-stack.css";

/** Spaces a column of `Field`s. Use instead of `space-y-*` or per-app gap overrides. */
export function FormStack(props: { children: JSX.Element; class?: string }) {
  return <div class={`pw-form ${props.class ?? ""}`}>{props.children}</div>;
}
