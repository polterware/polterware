import { createPanelPortalOwnership } from "./panel-interaction";
import type { JSX } from "solid-js";
import { createEffect, untrack } from "solid-js";
import { Portal } from "solid-js/web";
/**
 * Compact dropdown: **Ark UI Menu** (behavior) + `.menu-panel` Polter rows (look).
 * For searchable switchers use Popover + `MenuList` / `MenuItem` instead.
 *
 * Motion (fluid-functionalism): the surface springs in from its anchor side
 * (`createPopupMotion`); one hover highlight glides between rows, driven by the pointer
 * and by Ark's keyboard highlight alike; checked rows share one sliding marker.
 *
 * Portal + Content z-index: Zag floating positioners take z-index from Content
 * via `--z-index`; without a portal, panels stack under later shell siblings.
 */
import { Menu as ArkMenu, useMenuContext, type MenuRootProps } from "@ark-ui/solid/menu";
import { createMenuRows, MenuRowsLayer, MenuRowsProvider, useMenuRow } from "./menu-highlight";
import { createPopupMotion } from "./popup-motion";
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

/**
 * The menu surface inside Ark Content: popup presence, the shared row highlight, and the
 * two-way sync between it and Ark's highlighted item (so the keyboard moves the same
 * highlight as the pointer, and Enter picks what is lit).
 */
export function MenuSurface(props: { children: JSX.Element; class?: string }) {
  let surface: HTMLDivElement | undefined;
  const rows = createMenuRows(() => surface);
  const menu = useMenuContext();
  const motion = createPopupMotion({ scale: true });

  createEffect(() => {
    rows.hover.rects(); // rows may register after Ark highlights one on open
    const value = menu().highlightedValue;
    if (value == null) return;
    const id = rows.idOf(value);
    if (id !== undefined) rows.hover.setActiveIndex(id);
  });

  createEffect(() => {
    const id = rows.hover.activeIndex();
    if (id === null) return;
    const value = rows.valueOf(id);
    const api = untrack(menu);
    if (value !== undefined && api.highlightedValue !== value) api.setHighlightedValue(value);
  });

  return (
    <div
      ref={(element) => {
        surface = element;
        motion.ref(element);
      }}
      class={cn(
        "pw-dropdown-surface pw-menu-rows menu-panel p-1.5 text-[12px] text-[var(--color-text-primary)] outline-none",
        props.class,
      )}
      {...rows.hover.handlers}
    >
      <MenuRowsLayer rows={rows} />
      <MenuRowsProvider rows={rows}>{props.children}</MenuRowsProvider>
    </div>
  );
}

export function DropdownMenuContent(props: {
  children: JSX.Element;
  class?: string;
}) {
  const menu = useMenuContext();
  const ownPortal = createPanelPortalOwnership(() => menu().open);
  return (
    <Portal>
      {/* z-index on Content: Zag copies it to Positioner via --z-index */}
      <ArkMenu.Positioner class="pw-dropdown-positioner">
        <ArkMenu.Content ref={ownPortal} class="pw-dropdown-presence outline-none">
          <MenuSurface class={props.class}>{props.children}</MenuSurface>
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
  /** Checked row: the shared selected marker slides to it. */
  selected?: boolean;
  /** Keep the menu open after a pick (the caller closes it, e.g. after acknowledgment). */
  closeOnSelect?: boolean;
  onSelect?: () => void;
}) {
  let element: HTMLDivElement | undefined;
  const joined = useMenuRow(() => element, {
    value: () => props.value,
    selected: () => props.selected,
  });
  return (
    <ArkMenu.Item
      ref={element}
      value={props.value}
      disabled={props.disabled}
      closeOnSelect={props.closeOnSelect}
      class={cn(
        "relative flex w-full cursor-pointer items-center gap-2 rounded-[var(--radius-item)] px-2.5 py-2 text-left outline-none transition-colors duration-[var(--duration-fast)] ease-[var(--ease-standard)]",
        props.danger
          ? "text-[var(--color-danger)]"
          : joined
            ? "text-[var(--color-text-secondary)] data-[fluid-hover-active]:text-[var(--color-text-primary)] data-[highlighted]:text-[var(--color-text-primary)]"
            : "text-[var(--color-text-secondary)] data-[highlighted]:bg-[var(--color-surface-hover)] data-[highlighted]:text-[var(--color-text-primary)]",
        props.selected && !props.danger && "text-[var(--color-text-primary)]",
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
