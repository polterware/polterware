import { ColorPicker, parseColor, type ColorPickerRootProps } from "@ark-ui/solid/color-picker";
import { For, splitProps } from "solid-js";
import { Check } from "./hugeicons";
import "./picker-primitives.css";
export { parseColor } from "@ark-ui/solid/color-picker";
export type ColorSwatchOption = { value: string; label: string };
export type ColorSwatchPickerProps = Omit<ColorPickerRootProps, "children"> & { label: string; swatches: readonly ColorSwatchOption[] };
export function ColorSwatchPicker(props: ColorSwatchPickerProps) {
  const [local, rest] = splitProps(props, ["label", "swatches", "class"]);
  return <ColorPicker.Root {...rest} class={`pw-color-swatches ${local.class ?? ""}`}>
    <ColorPicker.Label>{local.label}</ColorPicker.Label>
    <ColorPicker.SwatchGroup class="pw-picker__grid" aria-label={local.label}>
      <For each={local.swatches}>{swatch => <ColorPicker.SwatchTrigger value={parseColor(swatch.value)} class="pw-color-swatches__button" aria-label={swatch.label}>
        <ColorPicker.Swatch value={parseColor(swatch.value)} class="pw-color-swatches__swatch">
          <ColorPicker.SwatchIndicator class="pw-color-swatches__indicator"><Check size={14} /></ColorPicker.SwatchIndicator>
        </ColorPicker.Swatch>
      </ColorPicker.SwatchTrigger>}</For>
    </ColorPicker.SwatchGroup>
  </ColorPicker.Root>;
}
