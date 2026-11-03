import type { JSX } from "solid-js";
import { splitProps } from "solid-js";
import "./badge.css";

export type BadgeTone = "ok" | "warn" | "danger" | "muted" | "info";

/**
 * Solid metadata badge (tags, labels, clickable relations).
 * Not status — use `StatusChip` for status/result/severity/health/risk.
 * Not a ghost filter — use `FilterChip` for secondary filters under toolbars.
 */
export function Badge(
  props: {
    children: JSX.Element;
    tone?: BadgeTone;
    class?: string;
    /** When set, renders as a button with the same solid badge chrome. */
    onClick?: () => void;
  } & Omit<JSX.ButtonHTMLAttributes<HTMLButtonElement>, "children" | "class" | "onClick">,
) {
  const [local, rest] = splitProps(props, ["children", "tone", "class", "onClick"]);
  const tone = () => local.tone ?? "muted";
  const className = () =>
    `pw-badge ${local.class ?? ""}`;

  if (local.onClick) {
    return (
      <button
        type="button"
        class={className()}
        data-tone={tone()}
        data-interactive="true"
        onClick={local.onClick}
        {...rest}
      >
        {local.children}
      </button>
    );
  }

  return (
    <span class={className()} data-tone={tone()}>
      {local.children}
    </span>
  );
}
