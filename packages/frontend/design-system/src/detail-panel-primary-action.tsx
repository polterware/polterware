import { Show, type JSX } from "solid-js";
import { Button } from "./button";
import { Tooltip } from "./tooltip";

export type DetailPanelPrimaryActionProps = {
  /** Accessible name and tooltip text; the button itself shows only the icon. */
  label: string;
  icon: JSX.Element;
  disabled?: boolean;
  /** Shown in the tooltip while the action is disabled. */
  disabledReason?: string;
  onClick?: () => void;
};

export function DetailPanelPrimaryAction(props: DetailPanelPrimaryActionProps) {
  const button = (triggerProps?: JSX.HTMLAttributes<HTMLButtonElement>) => (
    <Button
      {...triggerProps}
      variant="secondary"
      size="icon"
      class="pw-detail-panel__primary-action"
      aria-label={props.label}
      disabled={props.disabled}
      onClick={() => props.onClick?.()}
    >
      {props.icon}
    </Button>
  );

  return (
    <Show when={props.disabled && props.disabledReason} fallback={
      <Tooltip.Root>
        <Tooltip.Trigger asChild={(triggerProps) => button(triggerProps())} />
        <Tooltip.Content>{props.label}</Tooltip.Content>
      </Tooltip.Root>
    }>
      <Tooltip.Root>
        <Tooltip.Trigger
          asChild={(triggerProps) => (
            <span {...triggerProps({ tabIndex: 0 })} class="inline-flex">
              {button()}
            </span>
          )}
        />
        <Tooltip.Content>{props.disabledReason}</Tooltip.Content>
      </Tooltip.Root>
    </Show>
  );
}
