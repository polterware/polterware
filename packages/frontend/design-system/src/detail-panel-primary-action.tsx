import { Show, type JSX } from "solid-js";
import { Button } from "./button";
import { Tooltip } from "./tooltip";

export type DetailPanelPrimaryActionProps = {
  label: string;
  icon?: JSX.Element;
  disabled?: boolean;
  /** Shown in a tooltip while the action is disabled. */
  disabledReason?: string;
  onClick?: () => void;
};

export function DetailPanelPrimaryAction(props: DetailPanelPrimaryActionProps) {
  const button = () => (
    <Button
      variant="secondary"
      size="sm"
      class="pw-detail-panel__primary-action"
      disabled={props.disabled}
      onClick={() => props.onClick?.()}
    >
      {props.icon}
      {props.label}
    </Button>
  );

  return (
    <Show when={props.disabled && props.disabledReason} fallback={button()}>
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
