import type { JSX } from "solid-js";
import "./floating-count-badge.css";

export type FloatingCountBadgeProps = {
  count: number;
  ariaLabel?: string;
  tone?: FloatingCountBadgeTone;
  class?: string;
};

export type FloatingCountBadgeTone = "info" | "warning" | "success" | "danger";

export function hasFloatingCount(count: number | undefined): count is number {
  return count != null;
}

export function getFloatingCountAriaLabel(count: number, ariaLabel?: string): string {
  return ariaLabel ?? `${count} items`;
}

/** Count attached to the top-right corner of an interactive control. */
export function FloatingCountBadge(props: FloatingCountBadgeProps): JSX.Element {
  return (
    <span
      class={`pw-floating-count-badge pw-floating-count-badge--${props.tone ?? "info"} ${props.class ?? ""}`}
      aria-label={getFloatingCountAriaLabel(props.count, props.ariaLabel)}
    >
      {props.count}
    </span>
  );
}
