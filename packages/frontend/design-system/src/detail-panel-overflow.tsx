import type { JSX } from "solid-js";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "./dropdown-menu";
import { MoreHorizontal } from "./hugeicons";

export type DetailPanelOverflowProps = {
  children: JSX.Element;
  label?: string;
};

export type DetailPanelOverflowItemProps = {
  value: string;
  children: JSX.Element;
  danger?: boolean;
  disabled?: boolean;
  onSelect?: () => void;
};

export function DetailPanelOverflow(props: DetailPanelOverflowProps) {
  return (
    <DropdownMenu positioning={{ placement: "bottom-end" }}>
      <DropdownMenuTrigger
        class="pw-detail-panel__action pw-detail-panel__overflow-trigger control-field control-field-sm !rounded-full !p-0"
        aria-label={props.label ?? "More actions"}
      >
        <MoreHorizontal size={14} aria-hidden />
      </DropdownMenuTrigger>
      <DropdownMenuContent class="min-w-[160px]">{props.children}</DropdownMenuContent>
    </DropdownMenu>
  );
}

export function DetailPanelOverflowItem(props: DetailPanelOverflowItemProps) {
  return (
    <DropdownMenuItem
      value={props.value}
      danger={props.danger}
      disabled={props.disabled}
      onSelect={props.onSelect}
    >
      {props.children}
    </DropdownMenuItem>
  );
}
