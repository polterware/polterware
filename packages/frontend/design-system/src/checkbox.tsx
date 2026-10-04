import { Checkbox as ArkCheckbox, type CheckboxRootProps } from "@ark-ui/solid/checkbox";
import { Show, splitProps, type JSX } from "solid-js";
import { Check, MinusSignIcon } from "./hugeicons";
import "./form-primitives.css";

export type CheckboxProps = Omit<CheckboxRootProps, "children"> & { label?: JSX.Element; shape?: "round" | "square" };
export function Checkbox(props: CheckboxProps) {
  const [local, rest] = splitProps(props, ["label", "shape", "class"]);
  return <ArkCheckbox.Root {...rest} class={`pw-checkbox ${local.class ?? ""}`} data-shape={local.shape ?? "square"}>
    <ArkCheckbox.HiddenInput />
    <ArkCheckbox.Control class="pw-checkbox__control">
      <ArkCheckbox.Indicator><Check size={12} /></ArkCheckbox.Indicator>
      <ArkCheckbox.Indicator indeterminate><MinusSignIcon size={12} /></ArkCheckbox.Indicator>
    </ArkCheckbox.Control>
    <Show when={local.label}><ArkCheckbox.Label>{local.label}</ArkCheckbox.Label></Show>
  </ArkCheckbox.Root>;
}
