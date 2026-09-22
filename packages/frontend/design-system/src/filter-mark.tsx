import type { StatusTone } from "./status";
import "./filter-mark.css";

export type FilterMarkShape = "circle" | "square" | "diamond";

export type FilterMarkProps = {
  shape?: FilterMarkShape;
  tone?: StatusTone;
  /** Filled swatch (default) or outline ring for empty/negative states. */
  variant?: "filled" | "outline";
  class?: string;
};

/**
 * Compact geometric mark for filter menus when a Hugeicon would be noise
 * or when color/shape carries the meaning (status-adjacent, linked/unlinked).
 */
export function FilterMark(props: FilterMarkProps) {
  const shape = () => props.shape ?? "circle";
  const tone = () => props.tone ?? "muted";
  const variant = () => props.variant ?? "filled";

  return (
    <span
      class={`pw-filter-mark pw-filter-mark--${shape()} pw-filter-mark--${variant()} ${props.class ?? ""}`}
      data-tone={tone()}
      aria-hidden
    />
  );
}
