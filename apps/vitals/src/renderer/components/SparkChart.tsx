import { createEffect, onCleanup, onMount } from "solid-js";
import uPlot from "uplot";
import "uplot/dist/uPlot.min.css";

const CHART_HEIGHT = 120;

function tokenColor(name: string): string {
  return getComputedStyle(document.documentElement).getPropertyValue(name).trim();
}

export function SparkChart(props: { series: number[]; label: string; strokeToken?: string }) {
  let host: HTMLDivElement | undefined;
  let plot: uPlot | undefined;

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
        height: CHART_HEIGHT,
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
            size: 40,
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
      plot.setSize({ width: host.clientWidth || 280, height: CHART_HEIGHT });
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

  return <div class="vitals-chart" ref={host} role="img" aria-label={props.label} />;
}
