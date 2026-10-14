import { For, createSignal } from "solid-js";
import { Dynamic } from "solid-js/web";
import { Button } from "./button";
import { Popover } from "./popover";
import { BookOpen, MemoryStick, CalendarClock, Check, Folder, House, Zap, MapPin, Bell } from "./hugeicons";
import "./picker-primitives.css";

export const PICKER_ICONS = {
  book: BookOpen,
  brain: MemoryStick,
  calendar: CalendarClock,
  check: Check,
  folder: Folder,
  home: House,
  place: MapPin,
  bell: Bell,
  energy: Zap,
};
export type PickerIconName = keyof typeof PICKER_ICONS;
export type IconPickerProps = {
  label: string;
  value?: PickerIconName;
  defaultValue?: PickerIconName;
  onValueChange?: (value: PickerIconName) => void;
  disabled?: boolean;
};

const ICON_NAMES = Object.keys(PICKER_ICONS) as PickerIconName[];

/** The icon option grid on its own, for popovers that host more than the icons. */
function IconPickerGrid(props: { label: string; value: PickerIconName; onValueChange: (value: PickerIconName) => void }) {
  return (
    <div class="pw-picker__grid pw-icon-picker__options" role="listbox" aria-label={props.label}>
      <For each={ICON_NAMES}>
        {(name) => {
          const Icon = PICKER_ICONS[name];
          return (
            <button type="button" role="option" data-value={name} aria-label={name} aria-selected={props.value === name} class="pw-primitive-button pw-icon-picker__option" onClick={() => props.onValueChange(name)}>
              <Icon size={16} />
            </button>
          );
        }}
      </For>
    </div>
  );
}

export function IconPicker(props: IconPickerProps) {
  const [internal, setInternal] = createSignal<PickerIconName>(props.defaultValue ?? "brain");
  const [open, setOpen] = createSignal(false);
  const value = () => props.value ?? internal();
  let trigger: HTMLButtonElement | undefined;
  const choose = (next: PickerIconName) => {
    if (props.value === undefined) setInternal(next);
    props.onValueChange?.(next);
    setOpen(false);
    trigger?.focus();
  };
  return (
    <>
      <Button
        ref={trigger}
        variant="secondary"
        disabled={props.disabled}
        aria-haspopup="dialog"
        aria-expanded={open()}
        onClick={() => setOpen(!open())}
      >
        <Dynamic component={PICKER_ICONS[value()]} size={14} />
        {props.label}
      </Button>
      <Popover
        open={open() && !props.disabled}
        onClose={() => {
          setOpen(false);
          trigger?.focus();
        }}
        anchorRef={() => trigger}
        title={props.label}
        width={280}
      >
        <div class="pw-picker__grid pw-icon-picker__options" role="listbox" aria-label={props.label}>
          <For each={ICON_NAMES}>
            {(name) => {
              const Icon = PICKER_ICONS[name];
              return (
                <button
                  type="button"
                  role="option"
                  data-value={name}
                  aria-label={name}
                  aria-selected={value() === name}
                  class="pw-primitive-button pw-icon-picker__option"
                  onClick={() => choose(name)}
                >
                  <Icon size={16} />
                </button>
              );
            }}
          </For>
        </div>
      </Popover>
    </>
  );
}
IconPicker.Grid = IconPickerGrid;
