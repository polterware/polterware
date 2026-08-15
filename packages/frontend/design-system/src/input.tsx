import type { JSX } from "solid-js";
import { splitProps } from "solid-js";

export type ControlFieldSize = "sm" | "md";

function cn(...parts: Array<string | false | null | undefined>) {
  return parts.filter(Boolean).join(" ");
}

const fieldBase =
  "control-field w-full min-w-0 outline-none text-[var(--color-text-primary)]";

/**
 * Product text input — control-field edge-light chrome (same family as Card/Surface).
 * Prefer over ad-hoc border/rounded field classes.
 */
export type InputProps = JSX.InputHTMLAttributes<HTMLInputElement> & {
  size?: ControlFieldSize;
};

export function Input(props: InputProps) {
  const [local, rest] = splitProps(props, ["size", "class", "classList", "type"]);
  const size = () => local.size ?? "sm";
  return (
    <input
      type={local.type ?? "text"}
      {...rest}
      class={cn(
        fieldBase,
        size() === "md" ? "control-field-md" : "control-field-sm",
        local.class,
      )}
      classList={local.classList}
    />
  );
}

/**
 * Product textarea — same control-field material as Input.
 */
export type TextareaProps = JSX.TextareaHTMLAttributes<HTMLTextAreaElement> & {
  /** Visual density; height is min-height via control-field-area. */
  size?: ControlFieldSize;
};

export function Textarea(props: TextareaProps) {
  const [local, rest] = splitProps(props, ["size", "class", "classList"]);
  return (
    <textarea
      {...rest}
      class={cn(fieldBase, "control-field-area", local.class)}
      classList={local.classList}
    />
  );
}
