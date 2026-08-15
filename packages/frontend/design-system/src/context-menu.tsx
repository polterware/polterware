import type { JSX } from "solid-js";
import { Portal } from "solid-js/web";
import { Menu as ArkMenu } from "@ark-ui/solid/menu";
import { DropdownMenuItem, DropdownMenuSeparator } from "./dropdown-menu";
import "./dropdown-menu.css";

function cn(...parts: Array<string | false | null | undefined>) {
  return parts.filter(Boolean).join(" ");
}

export type ContextMenuRootProps = {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  anchorPoint?: { x: number; y: number } | null;
  children: JSX.Element;
};

function ContextMenuRoot(props: ContextMenuRootProps) {
  return (
    <ArkMenu.Root
      open={props.open}
      onOpenChange={(details) => props.onOpenChange(details.open)}
      positioning={{
        getAnchorRect: () =>
          props.anchorPoint
            ? { ...props.anchorPoint, width: 0, height: 0 }
            : null,
        placement: "bottom-start",
        strategy: "fixed",
        gutter: 0,
      }}
      lazyMount
      unmountOnExit
    >
      {props.children}
    </ArkMenu.Root>
  );
}

function ContextMenuContent(props: { children: JSX.Element; class?: string }) {
  return (
    <Portal>
      <ArkMenu.Positioner class="pw-dropdown-positioner">
        <ArkMenu.Content class="pw-dropdown-presence outline-none">
          <div
            class={cn(
              "pw-dropdown-surface menu-panel min-w-[160px] py-1 text-[12px] text-[var(--color-text-primary)] outline-none",
              props.class,
            )}
          >
            {props.children}
          </div>
        </ArkMenu.Content>
      </ArkMenu.Positioner>
    </Portal>
  );
}

export const ContextMenu = {
  Root: ContextMenuRoot,
  Content: ContextMenuContent,
  Item: DropdownMenuItem,
  Separator: DropdownMenuSeparator,
};
