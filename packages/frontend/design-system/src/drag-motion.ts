/**
 * Transform-only drag motion for a card: it follows the pointer 1:1, lifts a hair while held,
 * and settles on a spring. Interruptible: every call starts from the value on screen, so a
 * grab mid-settle carries on from where the card is. Also drives FLIP landings (`jump` to
 * the old spot, then `settle` to rest). Reduced motion drops the lift and snaps the settle.
 */
import { animate, motionValue, type AnimationPlaybackControls } from "motion";
import { prefersReducedMotion } from "./reduced-motion";
import { spring } from "./springs";

/** How far a held card rises, as a scale (default). */
const LIFT_SCALE = 1.02;

export type DragMotion = {
  /** Move to an offset from the resting position, no spring: the card tracks the pointer. */
  follow: (x: number, y: number) => void;
  /** Lift on grab, drop on release. */
  lift: (held: boolean) => void;
  /** Spring to an offset (default: rest). Resolves when the card has settled. */
  settle: (x?: number, y?: number) => Promise<void>;
  stop: () => void;
};

export function createDragMotion(element: HTMLElement, options: { liftScale?: number } = {}): DragMotion {
  const liftScale = options.liftScale ?? LIFT_SCALE;
  const x = motionValue(0);
  const y = motionValue(0);
  const scale = motionValue(1);
  let running: AnimationPlaybackControls[] = [];
  // The lift runs on its own track: following the pointer must not cut it short.
  let lifting: AnimationPlaybackControls | undefined;

  const paint = () => {
    const resting = x.get() === 0 && y.get() === 0 && scale.get() === 1;
    // No transform at rest, so the card keeps no stacking context of its own.
    element.style.transform = resting
      ? ""
      : `translate3d(${x.get()}px, ${y.get()}px, 0) scale(${scale.get()})`;
  };
  const unsubscribe = [x.on("change", paint), y.on("change", paint), scale.on("change", paint)];

  const stopRunning = () => {
    for (const controls of running) controls.stop();
    running = [];
  };
  const stopLifting = () => {
    lifting?.stop();
    lifting = undefined;
  };

  return {
    follow(nextX, nextY) {
      stopRunning();
      x.set(nextX);
      y.set(nextY);
    },
    lift(held) {
      const target = held && !prefersReducedMotion() ? liftScale : 1;
      stopLifting();
      lifting = animate(scale, target, spring.fast);
    },
    settle(toX = 0, toY = 0) {
      stopLifting();
      if (prefersReducedMotion()) {
        stopRunning();
        x.set(toX);
        y.set(toY);
        scale.set(1);
        return Promise.resolve();
      }
      stopRunning();
      const moves = [
        animate(x, toX, spring.moderate),
        animate(y, toY, spring.moderate),
        animate(scale, 1, spring.fast),
      ];
      running = moves;
      return Promise.all(moves.map((controls) => controls.finished.catch(() => undefined))).then(
        () => undefined,
      );
    },
    stop() {
      stopRunning();
      stopLifting();
      for (const off of unsubscribe) off();
    },
  };
}
