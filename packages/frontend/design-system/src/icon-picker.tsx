import { createSignal } from "solid-js";
import { Dynamic } from "solid-js/web";
import { Button } from "./button";
import { Popover } from "./popover";
import { SegmentedControl } from "./segmented-control";
import { BookOpen, MemoryStick, CalendarClock, Check, Folder, House, Zap, MapPin, Bell } from "./hugeicons";
import "./picker-primitives.css";
export const PICKER_ICONS = { book: BookOpen, brain: MemoryStick, calendar: CalendarClock, check: Check, folder: Folder, home: House, place: MapPin, bell: Bell, energy: Zap };
export type PickerIconName = keyof typeof PICKER_ICONS;
export type IconPickerProps = { label: string; value?: PickerIconName; defaultValue?: PickerIconName; onValueChange?: (value: PickerIconName) => void; disabled?: boolean };
const options = Object.keys(PICKER_ICONS).map(value => ({ value, label: value[0]!.toUpperCase() + value.slice(1) }));
export function IconPicker(props: IconPickerProps) {
  const [internal, setInternal] = createSignal<PickerIconName>(props.defaultValue ?? "brain");
  const [open, setOpen] = createSignal(false);
  const value = () => props.value ?? internal();
  let trigger: HTMLButtonElement | undefined;
  return <>
    <Button ref={trigger} variant="secondary" disabled={props.disabled} aria-haspopup="dialog" aria-expanded={open()} onClick={() => setOpen(!open())}>
      <Dynamic component={PICKER_ICONS[value()]} size={14} />{props.label}
    </Button>
    <Popover open={open() && !props.disabled} onClose={() => { setOpen(false); trigger?.focus(); }} anchorRef={() => trigger} title={props.label} width={320}>
      <div class="pw-picker__body"><SegmentedControl class="pw-icon-picker__options" aria-label={props.label} options={options} value={value()} onValueChange={detail => {
        const next = detail.value as PickerIconName;
        if (!next || !Object.hasOwn(PICKER_ICONS, next)) return;
        if (props.value === undefined) setInternal(next);
        props.onValueChange?.(next); setOpen(false); trigger?.focus();
      }} /></div>
    </Popover>
  </>;
}
