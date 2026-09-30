/**
 * CSS twins of the spring tiers, for motion that has to stay in CSS: Ark presence
 * unmounts on `animationend` (dialog, drawer), and the detail panel's settlement listens
 * for `transitionend`. Each tier's curve is sampled from the very spring in `springs.ts`
 * into a `linear()` easing, so CSS and JS motion share one feel and no duration is
 * written by hand.
 *
 *   --spring-slow-duration / --spring-slow-ease          enter (spring)
 *   --spring-slow-exit-duration                          exit (quicker tween)
 */
import { generateLinearEasing, spring as springGenerator } from "motion";
import { spring } from "./springs";

type Tier = keyof typeof spring;

const TIERS: readonly Tier[] = ["fast", "moderate", "slow"];

/** Samples a tier's enter spring (0 -> 1) as a CSS `linear()` easing. */
export function springLinearEasing(tier: Tier): string {
  const { duration, bounce } = spring[tier];
  const milliseconds = duration * 1000;
  const generator = springGenerator({ keyframes: [0, 1], duration: milliseconds, bounce });
  // A finer step than the default keeps the 80ms tier from collapsing to a few points.
  return generateLinearEasing(
    (progress) => generator.next(progress * milliseconds).value,
    milliseconds,
    Math.min(10, milliseconds / 12),
  );
}

/** Every tier's custom properties, as name -> value. */
export function springCssProperties(): Record<string, string> {
  const properties: Record<string, string> = {};
  for (const tier of TIERS) {
    properties[`--spring-${tier}-duration`] = `${spring[tier].duration * 1000}ms`;
    properties[`--spring-${tier}-ease`] = springLinearEasing(tier);
    properties[`--spring-${tier}-exit-duration`] = `${spring[tier].exit.duration * 1000}ms`;
  }
  return properties;
}

/** Publishes the tiers on `:root`. Runs once when the design system loads. */
export function installSpringCssProperties(root: HTMLElement = document.documentElement) {
  for (const [name, value] of Object.entries(springCssProperties())) {
    root.style.setProperty(name, value);
  }
}
