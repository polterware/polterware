import type { JSX } from "solid-js";
import { splitProps } from "solid-js";

export type FadeEdgeBarProps = JSX.HTMLAttributes<HTMLDivElement> & {
  children: JSX.Element;
  /** Keep the gradient one pixel away from a host surface rim. */
  insetEdges?: boolean;
};

/**
 * Surface-backed bar that fades to transparent at its bottom edge.
 * Positioning and spacing stay with the consumer so it works in flow or as an overlay.
 */
export function FadeEdgeBar(props: FadeEdgeBarProps) {
  const [local, rest] = splitProps(props, ["children", "class", "insetEdges"]);

  return (
    <div
      {...rest}
      class={`pw-fade-edge-bar ${local.insetEdges ? "pw-fade-edge-bar--inset-edges" : ""} ${local.class ?? ""}`}
    >
      {local.children}
    </div>
  );
}
