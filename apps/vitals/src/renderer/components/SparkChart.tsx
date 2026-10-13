import { createEffect, onCleanup, onMount } from "solid-js";
import uPlot from "uplot";
import "uplot/dist/uPlot.min.css";

const DEFAULT_HEIGHT = 96;

/** Short axis ticks (1.2K, 3.4M) so large byte rates fit the narrow axis instead of truncating. */
function compactTick(value: number): string {
  const abs = Math.abs(value);
  for (const [unit, size] of [["G", 1e9], ["M", 1e6], ["K", 1e3]] as const) {
    if (abs >= size) return `${Number((value / size).toFixed(1))}${unit}`;
  }
  return String(Number(value.toFixed(1)));
}

function tokenColor(name: string): string {
  return getComputedStyle(document.documentElement).getPropertyValue(name).trim();
}

export function SparkChart(props: { series: number[]; label: string; strokeToken?: string; height?: number }) {
  let host: HTMLDivElement | undefined;
  let plot: uPlot | undefined;

  const height = () => props.height ?? DEFAULT_HEIGHT;
  const strokeToken = () => props.strokeToken ?? "--color-chart-red";

  function aligned(): uPlot.AlignedData {
    const series = props.series.length ? props.series : [0];
    return [series.map((_, index) => index), series];
  }

  onMount(() => {
    const element = host;
    if (!element) return;
    const width = element.clientWidth || 280;
    plot = new uPlot(
      {
        width,
        height: height(),
        series: [
          {},
          {
            label: props.label,
            stroke: tokenColor(strokeToken()),
            width: 2,
          },
        ],
        axes: [
          { show: false },
          {
            stroke: tokenColor("--color-chart-axis"),
            grid: { show: true, stroke: tokenColor("--color-chart-grid") },
            size: 46,
            values: (_, splits) => splits.map(compactTick),
          },
        ],
        legend: { show: false },
        padding: [8, 8, 0, 0],
      },
      aligned(),
      element,
    );

    const observer = new ResizeObserver(() => {
      if (!plot || !host) return;
      plot.setSize({ width: host.clientWidth || 280, height: height() });
    });
    observer.observe(element);
    onCleanup(() => {
      observer.disconnect();
      plot?.destroy();
      plot = undefined;
    });
  });

  createEffect(() => {
    const next = aligned();
    const label = props.label;
    const stroke = tokenColor(strokeToken());
    if (!plot) return;
    const series = plot.series[1];
    if (series) {
      series.label = label;
      series.stroke = () => stroke; // uPlot normalizes stroke to a function
    }
    plot.setData(next);
  });

  return <div class="vitals-chart" ref={host} style={{ height: `${height()}px` }} role="img" aria-label={props.label} />;
}
