import { createEffect, onCleanup, onMount, type JSX } from "solid-js";
import { createDragMotion, createPlacementMotion, prefersReducedMotion } from "@polterware/frontend-design-system";

export function AtlasDropHighlight(props: {
  target: { x: number; y: number; width: number; height: number } | null;
}) {
  let frame: HTMLDivElement | undefined;
  let seen = false;
  let last = { x: 0, y: 0, width: 0, height: 0 };

  onMount(() => {
    if (!frame) return;
    const place = createPlacementMotion(frame);
    createEffect(() => {
      const target = props.target;
      if (target) last = target;
      const box = target ?? last;
      place.place(
        {
          x: box.x,
          y: box.y,
          width: box.width,
          height: box.height,
          opacity: target ? 1 : 0,
        },
        !seen || prefersReducedMotion(),
        "fast",
      );
      seen = true;
    });
    onCleanup(() => place.stop());
  });

  return (
    <div
      ref={frame}
      class="atlas-card-drop-target"
      style={{
        "z-index": "1",
        opacity: "0",
        "pointer-events": "none",
      }}
    />
  );
}

export function AtlasCardFrame(props: {
  x: number;
  y: number;
  width: number;
  height: number;
  depth: number;
  concealed: boolean;
  dragging: boolean;
  dragOffset: { dx: number; dy: number } | null;
  entering: boolean;
  onEnterEnd?: () => void;
  children: JSX.Element;
}) {
  let frame: HTMLDivElement | undefined;
  let dragLayer: HTMLDivElement | undefined;
  let seen = false;
  let prevX = 0;
  let prevY = 0;
  let lastOffset = { dx: 0, dy: 0 };

  onMount(() => {
    if (!frame || !dragLayer) return;
    const place = createPlacementMotion(frame);
    const drag = createDragMotion(dragLayer);
    prevX = props.x;
    prevY = props.y;

    createEffect(() => {
      const held = props.dragging;
      const offset = props.dragOffset;
      const x = props.x;
      const y = props.y;
      const jumped = seen && Math.hypot(x - prevX, y - prevY) > 0.5;
      place.place(
        {
          x,
          y,
          width: props.width,
          height: props.height,
          opacity: props.concealed ? 0 : 1,
        },
        !seen || held || jumped || prefersReducedMotion(),
      );
      if (held && offset) {
        drag.lift(true);
        drag.follow(offset.dx, offset.dy);
        lastOffset = offset;
      } else if (jumped) {
        const carryX = prevX + lastOffset.dx - x;
        const carryY = prevY + lastOffset.dy - y;
        drag.lift(false);
        drag.follow(carryX, carryY);
        void drag.settle(0, 0);
        lastOffset = { dx: 0, dy: 0 };
      } else {
        drag.lift(false);
        if (!seen) drag.follow(0, 0);
        else void drag.settle(0, 0);
        lastOffset = { dx: 0, dy: 0 };
      }
      prevX = x;
      prevY = y;
      seen = true;
    });

    createEffect(() => {
      if (!props.entering) return;
      void place.enter().then(() => props.onEnterEnd?.());
    });

    onCleanup(() => {
      place.stop();
      drag.stop();
    });
  });

  return (
    <div
      ref={frame}
      class="atlas-card-frame"
      style={{
        position: "absolute",
        left: "0px",
        top: "0px",
        width: `${props.width}px`,
        height: `${props.height}px`,
        transform: `translate3d(${props.x}px, ${props.y}px, 0)`,
        // Even layers: domains paint at 4, components at 6, so edges (5) run between them.
        "z-index": String(props.depth * 2 + 2),
        "pointer-events": props.concealed ? "none" : undefined,
        opacity: props.concealed ? "0" : undefined,
      }}
    >
      <div ref={dragLayer} class="h-full w-full">
        {props.children}
      </div>
    </div>
  );
}
