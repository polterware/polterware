/**
 * Whether the OS asks for reduced motion. Read it where you call `animate`, not once at
 * module load, so a change in system settings applies to the next animation.
 * Reduced motion means fewer and gentler, not none: snap travel, scale, and stretch;
 * keep opacity and colour fades.
 */
export const prefersReducedMotion = () =>
  typeof matchMedia === "function" && matchMedia("(prefers-reduced-motion: reduce)").matches;
