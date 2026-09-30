import type { JSX } from "solid-js";
import { splitProps } from "solid-js";
import { createMenuRows, MenuRowsLayer, MenuRowsProvider, useMenuRow } from "./menu-highlight";

function cn(...parts: Array<string | false | null | undefined>) {
  return parts.filter(Boolean).join(" ");
}

/**
 * Compact panel surface for lists inside Popover (searchable switchers).
 * **Presentational only** — no fake menu keyboard model.
 * For true compact dropdowns use `DropdownMenu` (Ark Menu).
 */
export function MenuPanel(props: {
  children: JSX.Element;
  class?: string;
  /** Soft outer lift for floating menus over canvas/content. Default true. */
  elevated?: boolean;
}) {
  return (
    <div
      class={cn(
        "menu-panel py-1 text-[12px] text-[var(--color-text-primary)]",
        props.elevated === false && "shadow-none",
        props.class,
      )}
    >
      {props.children}
    </div>
  );
}

/**
 * List container for `MenuItem` rows inside a Popover (switchers, pickers). One hover
 * highlight glides between its rows (pointer and keyboard focus) and one selected marker
 * springs to the `selected` row. Put a divider between different kinds of rows by
 * starting a new `MenuList`. Use it as the scroll container when the list scrolls.
 */
export function MenuList(props: { children: JSX.Element; class?: string }) {
  let list: HTMLDivElement | undefined;
  const rows = createMenuRows(() => list);
  return (
    <div ref={list} class={cn("pw-menu-rows", props.class)} {...rows.hover.handlers}>
      <MenuRowsLayer rows={rows} />
      <MenuRowsProvider rows={rows}>{props.children}</MenuRowsProvider>
    </div>
  );
}

export type MenuItemProps = JSX.ButtonHTMLAttributes<HTMLButtonElement> & {
  /** Selected / checked row (neutral raised). */
  selected?: boolean;
  /** Destructive action styling. */
  danger?: boolean;
};

/**
 * Presentational action row for Popover lists (org/project switchers).
 * Styles only — not a headless menuitem. Use DropdownMenu.Item for real menus.
 * Inside a `MenuList` the shared highlight and marker paint its fills.
 */
export function MenuItem(props: MenuItemProps) {
  const [local, rest] = splitProps(props, [
    "class",
    "classList",
    "selected",
    "danger",
    "type",
    "children",
    "ref",
  ]);
  let button: HTMLButtonElement | undefined;
  const joined = useMenuRow(() => button, { selected: () => local.selected });
  return (
    <button
      type={local.type ?? "button"}
      {...rest}
      ref={(element) => {
        button = element;
        if (typeof local.ref === "function") local.ref(element);
      }}
      class={cn(
        "relative flex w-full cursor-pointer items-center gap-2 rounded-[var(--radius-item)] px-2 py-1.5 text-left outline-none transition-colors duration-[var(--duration-fast)] ease-[var(--ease-standard)]",
        joined
          ? local.selected
            ? "text-[var(--color-text-primary)]"
            : "text-[var(--color-text-secondary)] data-[fluid-hover-active]:text-[var(--color-text-primary)]"
          : local.selected
            ? "bg-[var(--color-surface-selected)] text-[var(--color-text-primary)]"
            : "text-[var(--color-text-secondary)] hover:bg-[var(--color-surface-hover)] hover:text-[var(--color-text-primary)]",
        local.danger && "text-[var(--color-danger)] hover:text-[var(--color-danger)]",
        "disabled:pointer-events-none disabled:text-[var(--color-text-disabled)]",
        local.class,
      )}
      classList={local.classList}
    >
      {local.children}
    </button>
  );
}
