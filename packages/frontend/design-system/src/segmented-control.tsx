import { SegmentGroup, type SegmentGroupRootProps } from "@ark-ui/solid/segment-group";
import { Index, createEffect, onCleanup, onMount, splitProps } from "solid-js";
import { animate, motionValue } from "motion";
import { prefersReducedMotion } from "./reduced-motion";
import { spring } from "./springs";
import "./form-primitives.css";

export type SegmentOption = { value: string; label: string; disabled?: boolean };
export type SegmentedControlProps = Omit<SegmentGroupRootProps, "children"> & { options: readonly SegmentOption[] };
export function SegmentedControl(props: SegmentedControlProps) {
  const [local, rest] = splitProps(props, ["options", "class"]);
  let root: HTMLDivElement | undefined;
  let indicator: HTMLDivElement | undefined;
  const x = motionValue(0), width = motionValue(0);
  let initialized = false;
  const paint = () => { if (indicator) { indicator.style.transform = `translateX(${x.get()}px)`; indicator.style.width = `${width.get()}px`; } };
  onMount(() => {
    const sync = () => {
      const selected = root?.querySelector<HTMLElement>('[data-part="item"][data-state="checked"]');
      if (!selected || !root) { if (indicator) indicator.hidden = true; return; }
      if (indicator) indicator.hidden = false;
      const targetX = selected.offsetLeft, targetWidth = selected.offsetWidth;
      if (!initialized || prefersReducedMotion()) { x.jump(targetX); width.jump(targetWidth); }
      else { animate(x, targetX, spring.moderate); animate(width, targetWidth, spring.moderate); }
      initialized = true;
      paint();
    };
    const stops = [x, width].map(value => value.on("change", paint));
    const observer = new MutationObserver(sync);
    observer.observe(root!, { subtree: true, attributes: true, attributeFilter: ["data-state"] });
    const resize = new ResizeObserver(sync); resize.observe(root!);
    createEffect(() => { local.options; rest.value; queueMicrotask(sync); });
    sync();
    onCleanup(() => { observer.disconnect(); resize.disconnect(); stops.forEach(stop => stop()); x.stop(); width.stop(); });
  });
  return <SegmentGroup.Root {...rest} ref={root} class={`pw-segmented ${local.class ?? ""}`}>
    <div ref={indicator} class="pw-segmented__indicator" aria-hidden="true" />
    <Index each={local.options}>{option => <SegmentGroup.Item value={option().value} disabled={option().disabled} class="pw-segmented__item">
      <SegmentGroup.ItemHiddenInput /><SegmentGroup.ItemText>{option().label}</SegmentGroup.ItemText>
    </SegmentGroup.Item>}</Index>
  </SegmentGroup.Root>;
}
