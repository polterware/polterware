import type { JSX } from "solid-js";
import { createEffect, onCleanup, onMount } from "solid-js";
/**
 * Two glyphs in one fixed icon cell that crossfade, after fluid-functionalism's icon swap
 * recipe: both stay mounted, the arriving glyph rides the fast tier's full duration
 * (ease-out) and the leaving one its quicker exit (ease-in), so an appear always
 * outlasts a disappear. Both are tweens on purpose: a critically damped spring settles
 * before its nominal duration and would invert that order. The leaving glyph fades,
 * shrinks to 0.6, and blurs to 4px; the arriving one resolves back to sharp. The cell
 * never resizes. Reduced motion keeps the fade, drops the scale and blur; the first
 * paint snaps.
 */
import { animate } from "motion";
import { prefersReducedMotion } from "./reduced-motion";
import { spring } from "./springs";
import "./icon-swap.css";

const enter = { type: "tween", duration: spring.fast.duration, ease: "easeOut" } as const;
const leave = { type: "tween", ...spring.fast.exit, ease: "easeIn" } as const;

export type IconSwapProps = {
  /** false shows `from`, true shows `to`. */
  swapped: boolean;
  from: JSX.Element;
  to: JSX.Element;
  size?: number;
  class?: string;
};

export function IconSwap(props: IconSwapProps) {
  let fromSlot: HTMLSpanElement | undefined;
  let toSlot: HTMLSpanElement | undefined;
  let mounted = false;

  const pose = (shown: boolean) => {
    const still = shown || prefersReducedMotion();
    return {
      opacity: shown ? 1 : 0,
      scale: still ? 1 : 0.6,
      filter: still ? "blur(0px)" : "blur(4px)",
    };
  };

  createEffect(() => {
    const swapped = props.swapped;
    if (!fromSlot || !toSlot) return;
    const instant = !mounted;
    animate(fromSlot, pose(!swapped), instant ? { duration: 0 } : swapped ? leave : enter);
    animate(toSlot, pose(swapped), instant ? { duration: 0 } : swapped ? enter : leave);
  });

  onMount(() => {
    const ready = requestAnimationFrame(() => {
      mounted = true;
    });
    onCleanup(() => cancelAnimationFrame(ready));
  });

  const size = () => props.size ?? 14;
  return (
    <span
      aria-hidden="true"
      class={`pw-icon-swap ${props.class ?? ""}`}
      style={{ width: `${size()}px`, height: `${size()}px` }}
    >
      <span ref={fromSlot} class="pw-icon-swap__glyph">
        {props.from}
      </span>
      <span ref={toSlot} class="pw-icon-swap__glyph">
        {props.to}
      </span>
    </span>
  );
}
