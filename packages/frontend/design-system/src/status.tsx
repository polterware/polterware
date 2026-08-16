import type { JSX } from "solid-js";
import "./status.css";

export type StatusTone =
  | "healthy"
  | "degraded"
  | "down"
  | "unknown"
  | "syncing"
  | "ok"
  | "warn"
  | "danger"
  | "muted";

/**
 * @deprecated StatusChip is always filled. Kept for call-site compatibility.
 * Pass nothing — the value is ignored.
 */
export type StatusChipAppearance = "default" | "filled";

const toneDot = {
  healthy: "bg-[var(--color-brand)]",
  ok: "bg-[var(--color-brand)]",
  degraded: "bg-[var(--color-warning)]",
  warn: "bg-[var(--color-warning)]",
  down: "bg-[var(--color-danger)]",
  danger: "bg-[var(--color-danger)]",
  unknown: "bg-[var(--color-text-disabled)]",
  muted: "bg-[var(--color-text-disabled)]",
  syncing: "bg-[var(--color-info)]",
} satisfies Record<StatusTone, string>;

const toneLabel = {
  healthy: "Healthy",
  ok: "Healthy",
  degraded: "Degraded",
  warn: "Warning",
  down: "Down",
  danger: "Danger",
  unknown: "Unknown",
  muted: "Unknown",
  syncing: "Syncing",
} satisfies Record<StatusTone, string>;

/**
 * Compact status indicator (dot ± label).
 * Use only when a full status badge would be too heavy — never as a labeled status badge.
 * Labeled status/result/severity/health/risk MUST use `StatusChip`.
 */
export function StatusDot(props: {
  ok?: boolean;
  tone?: StatusTone;
  label?: string;
  showLabel?: boolean;
  class?: string;
}) {
  const tone = (): StatusTone => {
    if (props.tone) return props.tone;
    if (props.ok === true) return "healthy";
    if (props.ok === false) return "down";
    return "unknown";
  };
  const label = () => props.label ?? toneLabel[tone()];
  const showLabel = () => props.showLabel !== false;

  return (
    <span
      class={`inline-flex items-center gap-1.5 text-xs text-[var(--color-text-secondary)] ${props.class ?? ""}`}
      role="status"
      aria-label={label()}
    >
      <span class={`inline-block h-1.5 w-1.5 shrink-0 rounded-full ${toneDot[tone()]}`} aria-hidden />
      {showLabel() ? <span>{label()}</span> : <span class="sr-only">{label()}</span>}
    </span>
  );
}

/**
 * Product status badge — always filled.
 * Opaque semantic fill, dark shade of the same tone for text, edge-light border, no leading dot.
 * MUST be used for status / result / severity / health / risk / environment kind labels.
 * MUST NOT hand-roll a second status pill or leading-dot chip.
 */
export function StatusChip(props: {
  tone?: StatusTone;
  /**
   * @deprecated Always filled. Accepted and ignored for backward compatibility.
   */
  appearance?: StatusChipAppearance;
  /** Prefer children; `label` accepted for call-site convenience. */
  label?: string;
  children?: JSX.Element;
  class?: string;
}) {
  const tone = () => props.tone ?? "unknown";
  return (
    <span
      class={`pw-status-chip ${props.class ?? ""}`}
      data-tone={tone()}
      data-appearance="filled"
      role="status"
    >
      {props.children ?? props.label ?? toneLabel[tone()]}
    </span>
  );
}
