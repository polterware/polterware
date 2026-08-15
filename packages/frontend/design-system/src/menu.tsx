import type { JSX } from "solid-js";
import { splitProps } from "solid-js";

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

export type MenuItemProps = JSX.ButtonHTMLAttributes<HTMLButtonElement> & {
  /** Selected / checked row (neutral raised). */
  selected?: boolean;
  /** Destructive action styling. */
  danger?: boolean;
};

/**
 * Presentational action row for Popover lists (org/project switchers).
 * Styles only — not a headless menuitem. Use DropdownMenu.Item for real menus.
 */
export function MenuItem(props: MenuItemProps) {
  const [local, rest] = splitProps(props, [
    "class",
    "classList",
    "selected",
    "danger",
    "type",
    "children",
  ]);
  return (
    <button
      type={local.type ?? "button"}
      {...rest}
      class={cn(
        "flex w-full cursor-pointer items-center gap-2 rounded-[var(--radius-item)] px-2 py-1.5 text-left outline-none transition-colors duration-[var(--duration-fast)] ease-[var(--ease-standard)]",
        local.selected
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
