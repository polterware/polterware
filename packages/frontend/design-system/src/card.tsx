import type { JSX } from "solid-js";

/**
 * Quiet nested card — hairline + whisper outer lift (`.card-panel`).
 * Prefer this over ad-hoc border + bg cards in features.
 */
export function Card(props: {
  children: JSX.Element;
  class?: string;
  ref?: HTMLDivElement | ((element: HTMLDivElement) => void);
}) {
  return (
    <div ref={props.ref} class={`card-panel p-3 ${props.class ?? ""}`}>
      {props.children}
    </div>
  );
}

/**
 * Elevated host surface — same chrome as MainSurface / dialog panel.
 * Use for large quiet panels; use `float` for overlays (dialogs already use surface-panel-float).
 */
export function Surface(props: {
  children: JSX.Element;
  class?: string;
  /** Add outer lift for floating panels (not nested in MainSurface). */
  float?: boolean;
}) {
  return (
    <div
      class={`${props.float ? "surface-panel-float" : "surface-panel"} overflow-hidden ${props.class ?? ""}`}
    >
      {props.children}
    </div>
  );
}
