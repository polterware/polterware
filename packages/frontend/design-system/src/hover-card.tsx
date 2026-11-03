import { HoverCard as ArkHoverCard } from "@ark-ui/solid/hover-card";
import type { JSX } from "solid-js";
import { Portal } from "solid-js/web";
import { createPopupMotion } from "./popup-motion";
import { FloatingSurfaceProvider } from "./floating-surface";
import "./hover-card.css";

export type HoverCardProps = {
  /** The element a long hover opens the card from. */
  trigger: JSX.Element;
  /** Rendered only while the card is open, so a live feed starts on open and stops on close. */
  children: JSX.Element;
  placement?: "right-start" | "right" | "left-start" | "bottom-start" | "top-start";
  gutter?: number;
  /** A long hover by default: the card is a preview, not a tooltip. */
  openDelay?: number;
  closeDelay?: number;
  disabled?: boolean;
  class?: string;
};

/**
 * A preview card a long hover opens beside its trigger (Ark Hover Card for the behavior, the
 * float surface for the look). It holds richer content than a Tooltip and stays open while the
 * pointer moves into it.
 */
export function HoverCard(props: HoverCardProps) {
  return (
    <ArkHoverCard.Root
      lazyMount
      unmountOnExit
      disabled={props.disabled}
      openDelay={props.openDelay ?? 700}
      closeDelay={props.closeDelay ?? 150}
      positioning={{ placement: props.placement ?? "right-start", gutter: props.gutter ?? 12, flip: true }}
    >
      <ArkHoverCard.Trigger asChild={(trigger) => <div class="pw-hover-card-trigger" {...trigger()}>{props.trigger}</div>} />
      <Portal>
        <ArkHoverCard.Positioner class="pw-hover-card-positioner">
          <HoverCardContent class={props.class}>{props.children}</HoverCardContent>
        </ArkHoverCard.Positioner>
      </Portal>
    </ArkHoverCard.Root>
  );
}

function HoverCardContent(props: { class?: string; children: JSX.Element }) {
  // Fast-tier spring in with a short slide from the trigger side; quicker tween out.
  const motion = createPopupMotion();
  return (
    <ArkHoverCard.Content ref={motion.ref} class={`pw-hover-card surface-panel-float ${props.class ?? ""}`}>
      <FloatingSurfaceProvider>{props.children}</FloatingSurfaceProvider>
    </ArkHoverCard.Content>
  );
}
