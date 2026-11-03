import type { JSX } from "solid-js";
import { cx } from "./classes";
import "./chrome.css";

export function SidebarFrame(props: {
  children: JSX.Element;
  class?: string;
  "aria-label"?: string;
  as?: "aside" | "div";
}) {
  const className = () => cx("pw-shell-sidebar", props.class);
  if (props.as === "div") {
    return <div class={className()}>{props.children}</div>;
  }
  return (
    <aside class={className()} aria-label={props["aria-label"] ?? "Sidebar"}>
      {props.children}
    </aside>
  );
}

export function MainFrame(props: {
  children: JSX.Element;
  class?: string;
  as?: "div" | "main";
  glass?: boolean;
  ref?: HTMLElement | ((el: HTMLElement) => void);
  tabIndex?: number;
  "aria-label"?: string;
}) {
  const className = () =>
    cx(
      "pw-shell-main-frame",
      props.glass === false && "pw-shell-main-frame--opaque",
      (props.as ?? "div") === "div" && "flex h-full min-w-0 flex-1 flex-col",
      props.class,
    );
  if (props.as === "main") {
    return (
      <main
        ref={props.ref as ((el: HTMLElement) => void) | undefined}
        class={className()}
        tabIndex={props.tabIndex}
        aria-label={props["aria-label"]}
      >
        {props.children}
      </main>
    );
  }
  return <div class={className()}>{props.children}</div>;
}

export function MainSurfaceFrame(props: {
  children: JSX.Element;
  class?: string;
  as?: "main" | "div";
}) {
  const className = () =>
    cx("pw-main-surface", "surface-panel", "flex min-h-0 min-w-0 flex-1 flex-col overflow-hidden", props.class);
  if (props.as === "div") {
    return <div class={className()}>{props.children}</div>;
  }
  return <main class={className()}>{props.children}</main>;
}
