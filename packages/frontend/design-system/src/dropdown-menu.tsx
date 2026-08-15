import type { JSX } from "solid-js";
import { Portal } from "solid-js/web";
/**
 * Compact dropdown: **Ark UI Menu** (behavior) + `.menu-panel` Polter rows (look).
 * For searchable switchers use Popover + presentational MenuItem instead.
 *
 * Portal + Content z-index: Zag floating positioners take z-index from Content
 * via `--z-index`; without a portal, panels stack under later shell siblings.
 */
import { Menu as ArkMenu, type MenuRootProps } from "@ark-ui/solid/menu";
import "./dropdown-menu.css";

function cn(...parts: Array<string | false | null | undefined>) {
  return parts.filter(Boolean).join(" ");
}

export type DropdownMenuProps = {
  children: JSX.Element;
  open?: boolean;
  onOpenChange?: (open: boolean) => void;
  /** Uncontrolled initial open */
  defaultOpen?: boolean;
  positioning?: MenuRootProps["positioning"];
};

export function DropdownMenu(props: DropdownMenuProps) {
  return (
    <ArkMenu.Root
      open={props.open}
      defaultOpen={props.defaultOpen}
      onOpenChange={(details) => props.onOpenChange?.(details.open)}
      positioning={props.positioning}
      lazyMount
      unmountOnExit
    >
      {props.children}
    </ArkMenu.Root>
  );
}

export function DropdownMenuTrigger(props: {
  children: JSX.Element;
  class?: string;
  disabled?: boolean;
  id?: string;
  title?: string;
  "aria-label"?: string;
}) {
  return (
    <ArkMenu.Trigger
      class={cn(
        "inline-flex cursor-pointer items-center outline-none",
        props.class,
      )}
      disabled={props.disabled}
      id={props.id}
      title={props.title}
      aria-label={props["aria-label"]}
    >
      {props.children}
    </ArkMenu.Trigger>
  );
}

export function DropdownMenuContent(props: {
  children: JSX.Element;
  class?: string;
}) {
  return (
    <Portal>
      {/* z-index on Content: Zag copies it to Positioner via --z-index */}
      <ArkMenu.Positioner class="pw-dropdown-positioner">
        <ArkMenu.Content class="pw-dropdown-presence outline-none">
          <div
            class={cn(
              "pw-dropdown-surface menu-panel py-1 text-[12px] text-[var(--color-text-primary)] outline-none",
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

export function DropdownMenuItem(props: {
  children: JSX.Element;
  class?: string;
  /** Required for Ark Menu selection identity */
  value: string;
  disabled?: boolean;
  danger?: boolean;
  onSelect?: () => void;
}) {
  return (
    <ArkMenu.Item
      value={props.value}
      disabled={props.disabled}
      class={cn(
        "flex w-full cursor-pointer items-center gap-2 rounded-[var(--radius-item)] px-2 py-1.5 text-left outline-none transition-colors duration-[var(--duration-fast)] ease-[var(--ease-standard)]",
        props.danger
          ? "text-[var(--color-danger)]"
          : "text-[var(--color-text-secondary)] data-[highlighted]:bg-[var(--color-surface-hover)] data-[highlighted]:text-[var(--color-text-primary)]",
        "data-[disabled]:pointer-events-none data-[disabled]:text-[var(--color-text-disabled)]",
        props.class,
      )}
      onSelect={() => props.onSelect?.()}
    >
      {props.children}
    </ArkMenu.Item>
  );
}

export function DropdownMenuSeparator(props: { class?: string }) {
  return (
    <ArkMenu.Separator
      class={cn("my-1 border-t border-[var(--color-border-subtle)]", props.class)}
    />
  );
}
