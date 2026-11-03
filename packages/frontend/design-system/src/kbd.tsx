import type { JSX } from "solid-js";
import "./kbd.css";

export type KbdProps = {
  children: JSX.Element;
  class?: string;
};

/** Compact keyboard key chip for tooltips and shortcut hints. */
export function Kbd(props: KbdProps) {
  return <kbd class={`pw-kbd ${props.class ?? ""}`}>{props.children}</kbd>;
}
