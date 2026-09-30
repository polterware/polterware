import type { WorkStage } from "@polterware/feature-shell";

/** Pointer travel before a press becomes a drag, so a plain click never moves a card. */
export const DRAG_DEAD_ZONE = 4;

/** A column's horizontal span in board content coordinates (scroll-independent). */
export type ColumnSpan = { stage: WorkStage; left: number; right: number };

/** The column under `x`; between columns or past either end, the nearest one. */
export function columnAt(spans: readonly ColumnSpan[], x: number): WorkStage | null {
  let best: WorkStage | null = null;
  let bestDistance = Number.POSITIVE_INFINITY;
  for (const span of spans) {
    const distance = x < span.left ? span.left - x : x > span.right ? x - span.right : 0;
    if (distance < bestDistance) {
      best = span.stage;
      bestDistance = distance;
    }
  }
  return best;
}
