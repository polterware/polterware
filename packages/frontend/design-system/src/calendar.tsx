import { DatePicker as ArkDatePicker, parseDate } from "@ark-ui/solid/date-picker";
import { For, Show, splitProps } from "solid-js";
import { ChevronLeft, ChevronRight } from "./hugeicons";
import "./calendar.css";

export type CalendarProps = {
  /** Selected day as `YYYY-MM-DD`; empty when none. */
  value?: string;
  onValueChange?: (value: string) => void;
  /** Earliest / latest selectable day as `YYYY-MM-DD`. */
  min?: string;
  max?: string;
  /** Shows a "Today" shortcut under the grid (default true). */
  showToday?: boolean;
  class?: string;
  "aria-label"?: string;
};

function localToday(): string {
  const now = new Date();
  return `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, "0")}-${String(now.getDate()).padStart(2, "0")}`;
}

function safeParse(value: string | undefined) {
  if (!value) return undefined;
  try { return parseDate(value); } catch { return undefined; }
}

/** Header, day/month/year views and the Today shortcut; render inside an Ark DatePicker root. */
export function CalendarBody(props: { showToday?: boolean }) {
  return (
        <ArkDatePicker.Context>{api => <>
          <ArkDatePicker.View view="day">
            <ArkDatePicker.ViewControl class="pw-calendar-picker__header">
              <ArkDatePicker.ViewTrigger class="pw-calendar-picker__title"><ArkDatePicker.RangeText /></ArkDatePicker.ViewTrigger>
              <div class="pw-calendar-picker__nav">
                <ArkDatePicker.PrevTrigger class="pw-calendar-picker__nav-button" aria-label="Previous month"><ChevronLeft size={14} /></ArkDatePicker.PrevTrigger>
                <Show when={props.showToday !== false}>
                  <button type="button" class="pw-calendar-picker__today" onClick={() => api().setValue([parseDate(localToday())])}>Today</button>
                </Show>
                <ArkDatePicker.NextTrigger class="pw-calendar-picker__nav-button" aria-label="Next month"><ChevronRight size={14} /></ArkDatePicker.NextTrigger>
              </div>
            </ArkDatePicker.ViewControl>
            <ArkDatePicker.Table class="pw-calendar-picker__table">
              <ArkDatePicker.TableHead><ArkDatePicker.TableRow>
                <For each={api().weekDays}>{day => <ArkDatePicker.TableHeader class="pw-calendar-picker__weekday">{day.narrow}</ArkDatePicker.TableHeader>}</For>
              </ArkDatePicker.TableRow></ArkDatePicker.TableHead>
              <ArkDatePicker.TableBody><For each={api().weeks}>{week => <ArkDatePicker.TableRow>
                <For each={week}>{day => <ArkDatePicker.TableCell value={day} class="pw-calendar-picker__cell">
                  <ArkDatePicker.TableCellTrigger class="pw-calendar-picker__day">{day.day}</ArkDatePicker.TableCellTrigger>
                </ArkDatePicker.TableCell>}</For>
              </ArkDatePicker.TableRow>}</For></ArkDatePicker.TableBody>
            </ArkDatePicker.Table>
          </ArkDatePicker.View>
          <ArkDatePicker.View view="month">
            <ArkDatePicker.ViewControl class="pw-calendar-picker__header">
              <ArkDatePicker.ViewTrigger class="pw-calendar-picker__title"><ArkDatePicker.RangeText /></ArkDatePicker.ViewTrigger>
              <div class="pw-calendar-picker__nav">
                <ArkDatePicker.PrevTrigger class="pw-calendar-picker__nav-button" aria-label="Previous year"><ChevronLeft size={14} /></ArkDatePicker.PrevTrigger>
                <ArkDatePicker.NextTrigger class="pw-calendar-picker__nav-button" aria-label="Next year"><ChevronRight size={14} /></ArkDatePicker.NextTrigger>
              </div>
            </ArkDatePicker.ViewControl>
            <ArkDatePicker.Table class="pw-calendar-picker__table pw-calendar-picker__table--months">
              <ArkDatePicker.TableBody><For each={api().getMonthsGrid({ columns: 3, format: "short" })}>{months => <ArkDatePicker.TableRow>
                <For each={months}>{month => <ArkDatePicker.TableCell value={month.value} class="pw-calendar-picker__cell">
                  <ArkDatePicker.TableCellTrigger class="pw-calendar-picker__day pw-calendar-picker__day--wide">{month.label}</ArkDatePicker.TableCellTrigger>
                </ArkDatePicker.TableCell>}</For>
              </ArkDatePicker.TableRow>}</For></ArkDatePicker.TableBody>
            </ArkDatePicker.Table>
          </ArkDatePicker.View>
          <ArkDatePicker.View view="year">
            <ArkDatePicker.ViewControl class="pw-calendar-picker__header">
              <ArkDatePicker.ViewTrigger class="pw-calendar-picker__title"><ArkDatePicker.RangeText /></ArkDatePicker.ViewTrigger>
              <div class="pw-calendar-picker__nav">
                <ArkDatePicker.PrevTrigger class="pw-calendar-picker__nav-button" aria-label="Previous years"><ChevronLeft size={14} /></ArkDatePicker.PrevTrigger>
                <ArkDatePicker.NextTrigger class="pw-calendar-picker__nav-button" aria-label="Next years"><ChevronRight size={14} /></ArkDatePicker.NextTrigger>
              </div>
            </ArkDatePicker.ViewControl>
            <ArkDatePicker.Table class="pw-calendar-picker__table pw-calendar-picker__table--months">
              <ArkDatePicker.TableBody><For each={api().getYearsGrid({ columns: 3 })}>{years => <ArkDatePicker.TableRow>
                <For each={years}>{year => <ArkDatePicker.TableCell value={year.value} class="pw-calendar-picker__cell">
                  <ArkDatePicker.TableCellTrigger class="pw-calendar-picker__day pw-calendar-picker__day--wide">{year.label}</ArkDatePicker.TableCellTrigger>
                </ArkDatePicker.TableCell>}</For>
              </ArkDatePicker.TableRow>}</For></ArkDatePicker.TableBody>
            </ArkDatePicker.Table>
          </ArkDatePicker.View>
        </>}</ArkDatePicker.Context>
  );
}

/**
 * Inline month calendar (Ark DatePicker, always open) styled with the shared tokens.
 * Day, month and year views share one header; pair it with `Popover` for a field.
 */
export function Calendar(props: CalendarProps) {
  const [local] = splitProps(props, ["value", "onValueChange", "min", "max", "showToday", "class"]);
  const selected = () => { const day = safeParse(local.value); return day ? [day] : []; };
  return (
    <ArkDatePicker.Root
      inline
      class={`pw-calendar-picker ${local.class ?? ""}`}
      aria-label={props["aria-label"] ?? "Calendar"}
      value={selected()}
      min={safeParse(local.min)}
      max={safeParse(local.max)}
      onValueChange={details => local.onValueChange?.(details.valueAsString[0] ?? "")}
    >
      <ArkDatePicker.Content class="pw-calendar-picker__content">
        <CalendarBody showToday={local.showToday} />
      </ArkDatePicker.Content>
    </ArkDatePicker.Root>
  );
}
