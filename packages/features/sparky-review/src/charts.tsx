import { For, Show, createMemo, createSignal } from "solid-js";
import { Card, DataTable } from "@polterware/frontend-design-system";
import type { ReviewDayJSON } from "./bridge";
import {
  PERIODS,
  PERIOD_LABEL,
  activityValue,
  focusWeekBuckets,
  periodCount,
  weekdayLetter,
  weekdayName,
  type ReviewRange,
  type WeekStart,
} from "./model";

export function WeeklyActivityChart(props: { days: readonly ReviewDayJSON[] }) {
  const days = createMemo(() => props.days);
  const [selected, setSelected] = createSignal(days().at(-1)?.date ?? "");
  const current = createMemo(() => days().find(day => day.date === selected()) ?? days().at(-1));
  const maximum = createMemo(() => Math.max(1, ...days().map(day => day.completed ?? 0)));

  return (
    <Card>
      <h2 class="mb-2 text-[13px] font-medium text-[var(--color-text-secondary)]">Weekly activity</h2>
      <div class="sparky-review-legend" aria-label="Bar colors show morning, afternoon, evening, and night completions">
        <For each={PERIODS}>{period => (
          <span class="sparky-review-legend__item">
            <span class="sparky-review-legend__swatch" data-period={period} />
            {PERIOD_LABEL[period]}
          </span>
        )}</For>
      </div>
      <div class="sparky-review-bars" role="group" aria-label="Completions by day">
        <For each={days()}>{day => {
          const total = () => day.completed ?? 0;
          const height = () => Math.max(16, 64 * total() / maximum());
          return (
            <button
              type="button"
              class="sparky-review-bar"
              aria-pressed={current()?.date === day.date}
              aria-label={`${weekdayName(day.date ?? "")}: ${activityValue(day)}`}
              onClick={() => setSelected(day.date ?? "")}
            >
              <div class="sparky-review-bar__stack">
                <Show when={total() > 0} fallback={<span class="sparky-review-bar__empty" />}>
                  <For each={PERIODS.filter(period => periodCount(day, period) > 0)}>{period => (
                    <span
                      class="sparky-review-bar__seg"
                      data-period={period}
                      style={{ height: `${Math.max(5, height() * periodCount(day, period) / total())}px` }}
                    />
                  )}</For>
                </Show>
              </div>
              <span class="sparky-review-bar__label">{weekdayLetter(day.date ?? "")}</span>
              <Show when={current()?.date === day.date}><span class="sparky-review-bar__mark" /></Show>
            </button>
          );
        }}</For>
      </div>
      <p class="sparky-review-details" role="status">{current() ? `${weekdayName(current()!.date ?? "")}: ${activityValue(current()!)}` : ""}</p>
      <DataTable.Root rows={days()} getRowId={row => row.date ?? ""} class="mt-3">
        <DataTable.Table aria-label="Weekly activity as a table">
          <DataTable.Header>
            <DataTable.ColumnHeader id="date" sortable={false}>Date</DataTable.ColumnHeader>
            <DataTable.ColumnHeader id="morning" sortable={false}>Morning</DataTable.ColumnHeader>
            <DataTable.ColumnHeader id="afternoon" sortable={false}>Afternoon</DataTable.ColumnHeader>
            <DataTable.ColumnHeader id="evening" sortable={false}>Evening</DataTable.ColumnHeader>
            <DataTable.ColumnHeader id="night" sortable={false}>Night</DataTable.ColumnHeader>
            <DataTable.ColumnHeader id="total" sortable={false}>Completed</DataTable.ColumnHeader>
          </DataTable.Header>
          <DataTable.Body>{(row: ReviewDayJSON) => (
            <DataTable.Row row={row} rowId={row.date ?? ""}>
              <DataTable.PrimaryCell title={row.date ?? ""} />
              <DataTable.Cell>{row.morning ?? 0}</DataTable.Cell>
              <DataTable.Cell>{row.afternoon ?? 0}</DataTable.Cell>
              <DataTable.Cell>{row.evening ?? 0}</DataTable.Cell>
              <DataTable.Cell>{row.night ?? 0}</DataTable.Cell>
              <DataTable.Cell>{row.completed ?? 0}</DataTable.Cell>
            </DataTable.Row>
          )}</DataTable.Body>
        </DataTable.Table>
      </DataTable.Root>
    </Card>
  );
}

export function FocusMinutesChart(props: { days: readonly ReviewDayJSON[]; range: ReviewRange; weekStart: WeekStart; total: number }) {
  const buckets = createMemo(() => {
    if (props.range === "week") {
      return props.days.map(day => ({ id: day.date ?? "", label: weekdayLetter(day.date ?? ""), minutes: day.focusMinutes ?? 0, detail: `${day.date}: ${day.focusMinutes ?? 0} min` }));
    }
    return focusWeekBuckets(props.days, props.weekStart).map(bucket => ({
      id: bucket.id, label: bucket.start.slice(5), minutes: bucket.minutes, detail: `${bucket.label}: ${bucket.minutes} min`,
    }));
  });
  const maximum = createMemo(() => Math.max(1, ...buckets().map(bucket => bucket.minutes)));

  return (
    <Card>
      <h2 class="mb-2 text-[13px] font-medium text-[var(--color-text-secondary)]">Focus minutes</h2>
      <p class="sparky-review-details">{props.total} min in this range</p>
      <div class="sparky-review-focus" role="img" aria-label={`Focus minutes, ${props.total} in this range`}>
        <For each={buckets()}>{bucket => (
          <div class="sparky-review-focus__col">
            <span
              class="sparky-review-focus__bar"
              style={{ height: `${Math.max(4, 72 * bucket.minutes / maximum())}px` }}
              title={bucket.detail}
            />
            <span class="sparky-review-focus__label">{bucket.label}</span>
          </div>
        )}</For>
      </div>
      <DataTable.Root rows={buckets()} getRowId={row => row.id} class="mt-3">
        <DataTable.Table aria-label="Focus minutes as a table">
          <DataTable.Header>
            <DataTable.ColumnHeader id="when" sortable={false}>Period</DataTable.ColumnHeader>
            <DataTable.ColumnHeader id="minutes" sortable={false}>Minutes</DataTable.ColumnHeader>
          </DataTable.Header>
          <DataTable.Body>{(row: { id: string; detail: string; minutes: number }) => (
            <DataTable.Row row={row} rowId={row.id}>
              <DataTable.PrimaryCell title={row.detail} />
              <DataTable.Cell>{row.minutes}</DataTable.Cell>
            </DataTable.Row>
          )}</DataTable.Body>
        </DataTable.Table>
      </DataTable.Root>
    </Card>
  );
}
