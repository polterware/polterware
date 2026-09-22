import type { JSX } from "solid-js";
import { splitProps } from "solid-js";

import { Button, type ButtonProps } from "./button";
import { Tooltip } from "./tooltip";
import "./toolbar-icon-button.css";

const TOOLBAR_ICON_BUTTON_TOOLTIP_DELAY = 400;

export type ToolbarIconButtonProps = Omit<ButtonProps, "variant" | "size" | "children"> & {
  label: string;
  children: JSX.Element;
};

/**
 * Round primary icon button for PageToolbar actions (add / play / etc.).
 * Label drives aria-label and the shared Tooltip — no native title attribute.
 */
export function ToolbarIconButton(props: ToolbarIconButtonProps) {
  const [local, buttonProps] = splitProps(props, ["label", "children", "class"]);
  const className = () => ["!rounded-full", local.class].filter(Boolean).join(" ");

  return (
    <Tooltip.Root disabled={buttonProps.disabled} openDelay={TOOLBAR_ICON_BUTTON_TOOLTIP_DELAY}>
      <Tooltip.Trigger
        asChild={(triggerProps) => (
          <Button
            {...triggerProps({
              ...buttonProps,
              "aria-label": local.label,
            })}
            variant="primary"
            size="icon"
            class={className()}
          >
            {local.children}
          </Button>
        )}
      />
      <Tooltip.Content class="pw-toolbar-icon-button__tooltip">
        <span class="pw-toolbar-icon-button__tooltip-label">{local.label}</span>
      </Tooltip.Content>
    </Tooltip.Root>
  );
}
