import { DatePicker as ArkDatePicker, type DatePickerRootProps } from "@ark-ui/solid/date-picker";
import { For, splitProps } from "solid-js";
import { Portal } from "solid-js/web";
import { CalendarClock, ChevronLeft, ChevronRight } from "./hugeicons";
import { createPopupMotion } from "./popup-motion";
import { createPanelPortalOwnership } from "./panel-interaction";
import "./picker-primitives.css";
import "./popover.css";
export { parseDate, type DateValue } from "@ark-ui/solid/date-picker";
export type DatePickerProps = Omit<DatePickerRootProps, "children"> & { label: string };
export function DatePicker(props: DatePickerProps) {
  const [local, rest] = splitProps(props, ["label", "class"]);
  const ownPortal = createPanelPortalOwnership(() => true);
  return <ArkDatePicker.Root {...rest} class={`pw-date-picker ${local.class ?? ""}`} lazyMount unmountOnExit>
    <ArkDatePicker.Label>{local.label}</ArkDatePicker.Label>
    <ArkDatePicker.Control class="pw-picker__control">
      <ArkDatePicker.Input class="control-field control-field-sm" />
      <ArkDatePicker.Trigger class="pw-primitive-button" aria-label="Choose date"><CalendarClock size={14} /></ArkDatePicker.Trigger>
      <ArkDatePicker.ClearTrigger class="pw-primitive-button" aria-label="Clear date">Clear</ArkDatePicker.ClearTrigger>
    </ArkDatePicker.Control>
    <Portal><ArkDatePicker.Positioner class="pw-popover-positioner">
      <ArkDatePicker.Content ref={ownPortal} class="pw-popover-presence">
        <DatePickerSurface />
      </ArkDatePicker.Content>
    </ArkDatePicker.Positioner></Portal>
  </ArkDatePicker.Root>;
}
function DatePickerSurface() {
  const motion = createPopupMotion();
  return <div ref={motion.ref} class="surface-panel-float pw-date-picker__surface">
    <ArkDatePicker.ViewControl class="pw-picker__control">
      <ArkDatePicker.PrevTrigger class="pw-primitive-button" aria-label="Previous month"><ChevronLeft size={14} /></ArkDatePicker.PrevTrigger>
      <ArkDatePicker.RangeText />
      <ArkDatePicker.NextTrigger class="pw-primitive-button" aria-label="Next month"><ChevronRight size={14} /></ArkDatePicker.NextTrigger>
    </ArkDatePicker.ViewControl>
    <ArkDatePicker.MonthSelect class="control-field control-field-sm" aria-label="Month" />
    <ArkDatePicker.YearSelect class="control-field control-field-sm" aria-label="Year" />
    <ArkDatePicker.View view="day"><ArkDatePicker.Context>{context => <ArkDatePicker.Table>
      <ArkDatePicker.TableHead><ArkDatePicker.TableRow>
        <For each={context().weekDays}>{day => <ArkDatePicker.TableHeader>{day.short}</ArkDatePicker.TableHeader>}</For>
      </ArkDatePicker.TableRow></ArkDatePicker.TableHead>
      <ArkDatePicker.TableBody><For each={context().weeks}>{week => <ArkDatePicker.TableRow>
        <For each={week}>{day => <ArkDatePicker.TableCell value={day}><ArkDatePicker.TableCellTrigger class="pw-date-picker__day">{day.day}</ArkDatePicker.TableCellTrigger></ArkDatePicker.TableCell>}</For>
      </ArkDatePicker.TableRow>}</For></ArkDatePicker.TableBody>
    </ArkDatePicker.Table>}</ArkDatePicker.Context></ArkDatePicker.View>
  </div>;
}
