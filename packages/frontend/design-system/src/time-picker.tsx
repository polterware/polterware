import { createSignal } from "solid-js";
import { Button } from "./button";
import { Popover } from "./popover";
import { NumberStepper } from "./number-stepper";
import "./picker-primitives.css";
export type TimePickerProps = { label: string; value?: string; defaultValue?: string; onValueChange?: (value: string) => void; disabled?: boolean; minuteStep?: number };
export function parseTime(value: string): [number, number] | undefined {
  const match = /^(\d{2}):(\d{2})$/.exec(value);
  if (!match || Number(match[1]) > 23 || Number(match[2]) > 59) return undefined;
  return [Number(match[1]), Number(match[2])];
}
/** Local wall-clock HH:mm only; timezone and schedule semantics belong to the caller. */
export function TimePicker(props: TimePickerProps) {
  const [internal, setInternal] = createSignal(props.defaultValue ?? "09:00");
  const [open, setOpen] = createSignal(false);
  let trigger: HTMLButtonElement | undefined;
  const parts = () => parseTime(props.value ?? internal()) ?? [9, 0];
  const update = (part: 0 | 1, number: number) => {
    if (!Number.isFinite(number)) return;
    const next = [...parts()]; next[part] = Math.max(0, Math.min(part === 0 ? 23 : 59, Math.trunc(number)));
    const value = next.map(value => String(value).padStart(2, "0")).join(":");
    if (props.value === undefined) setInternal(value);
    props.onValueChange?.(value);
  };
  return <>
    <Button ref={trigger} variant="secondary" disabled={props.disabled} aria-haspopup="dialog" aria-expanded={open()} onClick={() => setOpen(!open())}>{props.label}: {props.value ?? internal()}</Button>
    <Popover open={open() && !props.disabled} onClose={() => { setOpen(false); trigger?.focus(); }} anchorRef={() => trigger} title={props.label} width={240}>
      <div class="pw-picker__body">
        <NumberStepper label="Hours" value={String(parts()[0])} min={0} max={23} onValueChange={detail => update(0, detail.valueAsNumber)} />
        <NumberStepper label="Minutes" value={String(parts()[1])} min={0} max={59} step={props.minuteStep ?? 1} onValueChange={detail => update(1, detail.valueAsNumber)} />
        <Button variant="secondary" onClick={() => { setOpen(false); trigger?.focus(); }}>Done</Button>
      </div>
    </Popover>
  </>;
}
