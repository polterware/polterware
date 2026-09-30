/**
 * Resting placement for a canvas card: position on the compositor, size as a
 * measured pixel box. The first call snaps. Later calls spring, unless the
 * caller is dragging or the person asked for reduced motion.
 */
import { animate, motionValue, type AnimationPlaybackControls } from "motion";
import { prefersReducedMotion } from "./reduced-motion";
import { spring } from "./springs";

export type Placement = {
  x: number;
  y: number;
  width: number;
  height: number;
  opacity: number;
};

export type PlacementMotion = {
  place: (next: Placement, snapMove: boolean, tier?: "moderate" | "fast") => void;
  enter: () => Promise<void>;
  stop: () => void;
};

export function createPlacementMotion(element: HTMLElement): PlacementMotion {
  const x = motionValue(0);
  const y = motionValue(0);
  const width = motionValue(0);
  const height = motionValue(0);
  const opacity = motionValue(1);
  const scale = motionValue(1);
  let running: AnimationPlaybackControls[] = [];
  let entered = false;

  const paint = () => {
    element.style.width = `${width.get()}px`;
    element.style.height = `${height.get()}px`;
    element.style.opacity = opacity.get() === 1 ? "" : String(opacity.get());
    const atRest = scale.get() === 1;
    element.style.transform = atRest
      ? `translate3d(${x.get()}px, ${y.get()}px, 0)`
      : `translate3d(${x.get()}px, ${y.get()}px, 0) scale(${scale.get()})`;
  };
  const unsubscribe = [x, y, width, height, opacity, scale].map((value) => value.on("change", paint));

  const stopRunning = () => {
    for (const controls of running) controls.stop();
    running = [];
  };

  return {
    place(next, snapMove, tier = "moderate") {
      stopRunning();
      const reduced = prefersReducedMotion();
      const motion = tier === "fast" ? spring.fast : spring.moderate;
      const snapSize = snapMove || reduced || !entered;
      if (snapMove || reduced || !entered) {
        x.set(next.x);
        y.set(next.y);
      } else {
        running.push(animate(x, next.x, motion), animate(y, next.y, motion));
      }
      if (snapSize) {
        width.set(next.width);
        height.set(next.height);
      } else {
        running.push(
          animate(width, next.width, motion),
          animate(height, next.height, motion),
        );
      }
      if (!entered) opacity.set(next.opacity);
      else running.push(animate(opacity, next.opacity, spring.fast));
      entered = true;
      paint();
    },
    enter() {
      if (prefersReducedMotion()) {
        scale.set(1);
        paint();
        return Promise.resolve();
      }
      scale.set(0.97);
      paint();
      const controls = animate(scale, 1, spring.fast);
      return controls.finished.then(() => undefined);
    },
    stop() {
      stopRunning();
      for (const off of unsubscribe) off();
      x.stop();
      y.stop();
      width.stop();
      height.stop();
      opacity.stop();
      scale.stop();
    },
  };
}
