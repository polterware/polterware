import type { JSX } from "solid-js";
import { cx } from "./classes";
import "./chrome.css";

export function ShellRoot(props: { children: JSX.Element; class?: string; glass?: boolean }) {
  return (
    <div
      class={cx(
        "pw-shell-root",
        props.glass === false && "pw-shell-root--opaque",
        "flex h-full flex-col text-[var(--color-text-primary)]",
        props.class,
      )}
    >
      {props.children}
    </div>
  );
}
