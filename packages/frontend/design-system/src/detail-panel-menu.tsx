import { Show, type JSX } from "solid-js";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "./dropdown-menu";
import { CollapseIcon, ExpandIcon, MoreHorizontal } from "./hugeicons";
import { Kbd } from "./kbd";

export type DetailPanelMenuItemProps = {
  value: string;
  children: JSX.Element;
  danger?: boolean;
  disabled?: boolean;
  onSelect?: () => void;
};

export type DetailPanelMenuProps = {
  expanded: boolean;
  expandShortcut?: string;
  onToggleExpanded: () => void;
  /** Feature commands; destructive items go last. */
  children?: JSX.Element;
};

/** The detail's "More actions" menu: the built-in expand command, then feature commands. */
export function DetailPanelMenu(props: DetailPanelMenuProps) {
  return (
    <DropdownMenu positioning={{ placement: "bottom-end" }}>
      <DropdownMenuTrigger
        class="pw-detail-panel__menu-trigger control-field control-field-sm h-[var(--control-height-sm)] w-[var(--control-height-sm)] justify-center !rounded-full !p-0 text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)]"
        aria-label="More actions"
      >
        <MoreHorizontal size={14} aria-hidden />
      </DropdownMenuTrigger>
      <DropdownMenuContent class="min-w-[200px]">
        <DropdownMenuItem value="detail-panel:expand" onSelect={props.onToggleExpanded}>
          <Show when={props.expanded} fallback={<ExpandIcon size={14} aria-hidden />}>
            <CollapseIcon size={14} aria-hidden />
          </Show>
          <span class="min-w-0 flex-1">{props.expanded ? "Restore width" : "Expand details"}</span>
          <Show when={props.expandShortcut}>
            <Kbd>{props.expandShortcut}</Kbd>
          </Show>
        </DropdownMenuItem>
        <Show when={props.children}>
          <DropdownMenuSeparator />
          {props.children}
        </Show>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}

export function DetailPanelMenuItem(props: DetailPanelMenuItemProps) {
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
