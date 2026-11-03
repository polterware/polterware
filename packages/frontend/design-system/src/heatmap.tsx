import { For } from "solid-js";
import "./heatmap.css";
export type HeatmapDay = { date: string; value: number };
export type HeatmapProps = { days: readonly HeatmapDay[]; label: string; thresholds?: readonly [number, number, number, number]; formatLabel?: (day: HeatmapDay) => string; class?: string };
export function heatmapLevel(value: number, thresholds: readonly number[] = [1, 3, 7, 14]): number {
  if (!Number.isFinite(value) || value <= 0) return 0;
  return Math.min(4, thresholds.filter(threshold => value >= threshold).length);
}
/** Read-only contribution calendar. Dates and complete weeks are supplied by the domain. */
export function Heatmap(props: HeatmapProps) {
  return <div class={`pw-heatmap ${props.class ?? ""}`} role="group" aria-label={props.label}>
    <For each={props.days}>{day => <div class="pw-heatmap__cell" data-level={heatmapLevel(day.value, props.thresholds)} role="img" aria-label={props.formatLabel?.(day) ?? `${day.date}: ${day.value}`} title={props.formatLabel?.(day) ?? `${day.date}: ${day.value}`} />}</For>
  </div>;
}
