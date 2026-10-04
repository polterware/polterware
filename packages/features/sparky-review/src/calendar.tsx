import { For, Show, createEffect, createMemo, createSignal } from "solid-js";
import { Card, DataTable, Heatmap, heatmapLevel } from "@polterware/frontend-design-system";
import type { ReviewDayJSON } from "./bridge";
import {
  HEATMAP_THRESHOLDS,
  heatmapLabel,
  moveHeatmapIndex,
  padContributionWeeks,
  weekdayLabels,
  type HeatmapCell,
  type WeekStart,
} from "./model";

export function ContributionCalendar(props: { days: readonly ReviewDayJSON[]; weekStart: WeekStart }) {
  const cells = createMemo(() => padContributionWeeks(props.days, props.weekStart));
  const lastInRange = () => {
    const list = cells();
    for (let i = list.length - 1; i >= 0; i--) if (list[i]?.inRange) return i;
    return Math.max(0, list.length - 1);
  };
  const [index, setIndex] = createSignal(lastInRange());
  const selected = createMemo(() => cells()[index()] ?? cells()[lastInRange()] ?? null);
  createEffect(() => { cells(); setIndex(lastInRange()); });
  let host: HTMLDivElement | undefined;

  createEffect(() => {
    const current = index();
    const nodes = host?.querySelectorAll<HTMLElement>(".pw-heatmap__cell") ?? [];
    nodes.forEach((node, i) => node.setAttribute("data-review-focus", i === current ? "true" : "false"));
  });

  const onKeyDown = (event: KeyboardEvent) => {
    const next = moveHeatmapIndex(index(), event.key, cells().length);
    if (next === index() && event.key !== "Home" && event.key !== "End") return;
    event.preventDefault();
    setIndex(next);
  };

  const onMouseOver = (event: MouseEvent) => {
    const cell = (event.target as HTMLElement | null)?.closest(".pw-heatmap__cell");
    if (!cell || !host) return;
    const next = [...host.querySelectorAll(".pw-heatmap__cell")].indexOf(cell);
    if (next >= 0) setIndex(next);
  };

  return (
    <Card>
      <h2 class="mb-2 text-[13px] font-medium text-[var(--color-text-secondary)]">Contribution</h2>
      <div class="sparky-review-calendar">
        <div class="sparky-review-weekdays" aria-hidden="true">
          <For each={weekdayLabels(props.weekStart)}>{label => <span>{label ?? ""}</span>}</For>
        </div>
        <div
          ref={host}
          class="sparky-review-heatmap"
          tabIndex={0}
          onKeyDown={onKeyDown}
          onMouseOver={onMouseOver}
        >
          <Heatmap
            label="12-month contribution calendar"
            days={cells()}
            thresholds={HEATMAP_THRESHOLDS}
            formatLabel={day => {
              const cell = cells().find(item => item.date === day.date);
              return heatmapLabel(cell ?? { date: day.date, value: day.value, completed: day.value, focusMinutes: 0, inRange: true });
            }}
          />
        </div>
      </div>
      <p class="sparky-review-details" role="status">{selected() ? heatmapLabel(selected()!) : ""}</p>
      <div class="sparky-review-scale" aria-label="Contribution intensity from less to more">
        <span>Less</span>
        <For each={[0, 1, 2, 3, 4]}>{level => <span class="sparky-review-scale__cell" data-level={level} />}</For>
        <span>More</span>
      </div>
      <Show when={cells().some(cell => cell.inRange)}>
        <DataTable.Root rows={cells().filter(cell => cell.inRange)} getRowId={row => row.date} class="mt-3">
          <DataTable.Table aria-label="Contribution calendar as a table">
            <DataTable.Header>
              <DataTable.ColumnHeader id="date" sortable={false}>Date</DataTable.ColumnHeader>
              <DataTable.ColumnHeader id="completed" sortable={false}>Completions</DataTable.ColumnHeader>
              <DataTable.ColumnHeader id="focus" sortable={false}>Focus minutes</DataTable.ColumnHeader>
              <DataTable.ColumnHeader id="level" sortable={false}>Level</DataTable.ColumnHeader>
            </DataTable.Header>
            <DataTable.Body>{(row: HeatmapCell) => (
              <DataTable.Row row={row} rowId={row.date}>
                <DataTable.PrimaryCell title={row.date} />
                <DataTable.Cell>{row.completed}</DataTable.Cell>
                <DataTable.Cell>{row.focusMinutes}</DataTable.Cell>
                <DataTable.Cell>{heatmapLevel(row.value, HEATMAP_THRESHOLDS)}</DataTable.Cell>
              </DataTable.Row>
            )}</DataTable.Body>
          </DataTable.Table>
        </DataTable.Root>
      </Show>
    </Card>
  );
}
