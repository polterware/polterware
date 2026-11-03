import type { JSX } from "solid-js";
import { cx } from "./classes";
import "./chrome.css";

export function TitleBarFrame(props: {
  class?: string;
  "aria-label"?: string;
  children?: JSX.Element;
  leading?: JSX.Element;
  center?: JSX.Element;
  trailing?: JSX.Element;
}) {
  return (
    <header
      class={cx("window-title-bar", "pw-shell-titlebar", props.class)}
      role="banner"
      aria-label={props["aria-label"]}
    >
      {props.children ?? (
        <>
          <div class="pw-shell-titlebar-slot pw-shell-titlebar-slot--leading">{props.leading}</div>
          <div class="pw-shell-titlebar-slot pw-shell-titlebar-slot--center">{props.center}</div>
          <div class="pw-shell-titlebar-spacer" aria-hidden />
          <div class="pw-shell-titlebar-slot pw-shell-titlebar-slot--trailing">{props.trailing}</div>
        </>
      )}
    </header>
  );
}
