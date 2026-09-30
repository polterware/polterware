import type { JSX } from "solid-js";
import {
  createContext,
  createEffect,
  createSignal,
  onCleanup,
  onMount,
  useContext,
} from "solid-js";
/**
 * Shared menu row motion for Ark menus (`DropdownMenu`, `ContextMenu`, `Select`) and
 * presentational popover lists (`MenuList` + `MenuItem`), after fluid-functionalism's
 * Dropdown: one hover highlight glides between rows (grown from the checked row on a fresh
 * entry), and one selected marker springs between checked rows on `spring.moderate`.
 * Rows join through context, so a row outside a menu list keeps its own fills.
 */
import { animate, motionValue } from "motion";
import { Check } from "./hugeicons";
import { createFluidHover, FluidHoverHighlight, type FluidHover } from "./fluid-hover";
import { prefersReducedMotion } from "./reduced-motion";
import { spring } from "./springs";
import "./menu-highlight.css";

/**
 * How long a pick holds its popup open before closing, so the check drawing in and the
 * marker springing to the picked row are seen instead of cut by the close. Escape, outside
 * press, and the trigger still close immediately.
 */
export const SELECTION_ACK_MS = 300;

/** Rows the highlight skips: disabled by whichever attribute the primitive uses. */
function isDisabledRow(element: HTMLElement) {
  return (
    element.getAttribute("aria-disabled") === "true" ||
    element.hasAttribute("data-disabled") ||
    (element instanceof HTMLButtonElement && element.disabled)
  );
}

type MenuRowsContextValue = {
  /** Joins a row; returns its id and the function that removes it. */
  register: (element: HTMLElement, value?: string) => { id: number; dispose: () => void };
  setSelected: (id: number, selected: boolean) => void;
};

const MenuRowsContext = createContext<MenuRowsContextValue>();

export type MenuRows = {
  hover: FluidHover;
  selectedId: () => number | null;
  idOf: (value: string) => number | undefined;
  valueOf: (id: number) => string | undefined;
  context: MenuRowsContextValue;
};

export function createMenuRows(container: () => HTMLElement | undefined): MenuRows {
  const hover = createFluidHover(container, { isItemDisabled: isDisabledRow });
  const [selectedId, setSelectedId] = createSignal<number | null>(null);
  const valueToId = new Map<string, number>();
  const idToValue = new Map<number, string>();
  // Ids are reused so a filtering list keeps its rect array short.
  const freeIds: number[] = [];
  let nextId = 0;

  const context: MenuRowsContextValue = {
    register: (element, value) => {
      const id = freeIds.pop() ?? nextId++;
      if (value !== undefined) {
        valueToId.set(value, id);
        idToValue.set(id, value);
      }
      const unregister = hover.registerItem(id, element);
      return {
        id,
        dispose: () => {
          unregister();
          if (value !== undefined && valueToId.get(value) === id) valueToId.delete(value);
          idToValue.delete(id);
          if (selectedId() === id) setSelectedId(null);
          freeIds.push(id);
        },
      };
    },
    setSelected: (id, selected) => {
      if (selected) setSelectedId(id);
      else if (selectedId() === id) setSelectedId(null);
    },
  };

  return {
    hover,
    selectedId,
    idOf: (value) => valueToId.get(value),
    valueOf: (id) => idToValue.get(id),
    context,
  };
}

export function MenuRowsProvider(props: { rows: MenuRows; children: JSX.Element }) {
  return (
    <MenuRowsContext.Provider value={props.rows.context}>{props.children}</MenuRowsContext.Provider>
  );
}

/** The hover highlight and the selected marker; render first inside the list container. */
export function MenuRowsLayer(props: { rows: MenuRows }) {
  const selectedRect = () => {
    const id = props.rows.selectedId();
    return id === null ? null : (props.rows.hover.rects()[id] ?? null);
  };
  return (
    <>
      <FluidHoverHighlight class="pw-menu-hover" hover={props.rows.hover} from={selectedRect()} />
      <FluidHoverHighlight
        class="pw-menu-selected"
        rect={selectedRect()}
        targetKey={props.rows.selectedId()}
        transition={spring.moderate}
      />
    </>
  );
}

/**
 * Joins the enclosing menu list, if any. Returns whether it did, so a row can drop its
 * own hover and selected fills only when the shared layer paints them.
 */
export function useMenuRow(
  element: () => HTMLElement | undefined,
  options: { value?: () => string | undefined; selected?: () => boolean | undefined } = {},
) {
  const context = useContext(MenuRowsContext);
  onMount(() => {
    const el = element();
    if (!context || !el) return;
    // Re-joins if the row's value changes (an `Index` row showing another option).
    createEffect(() => {
      const row = context.register(el, options.value?.());
      onCleanup(row.dispose);
      createEffect(() => context.setSelected(row.id, Boolean(options.selected?.())));
    });
  });
  return Boolean(context);
}

/**
 * A check that draws itself in (stroke 0 -> 1 over `spring.fast`, ease-out) and erases
 * faster (`spring.fast.exit`, ease-in). Its slot is always rendered at a fixed width, so
 * a selection landing never resizes the row.
 */
export function SelectionCheck(props: { checked: boolean; size?: number; class?: string }) {
  let slot: HTMLSpanElement | undefined;
  let mounted = false;
  const progress = motionValue(0);
  const draw = { type: "tween", duration: spring.fast.duration, ease: "easeOut" } as const;
  const erase = { type: "tween", ...spring.fast.exit, ease: "easeIn" } as const;
  let paths: SVGGeometryElement[] = [];

  const paint = () => {
    const offset = String(1 - progress.get());
    for (const path of paths) path.style.strokeDashoffset = offset;
  };

  onMount(() => {
    paths = [...(slot?.querySelectorAll<SVGGeometryElement>("path, polyline, line") ?? [])];
    for (const path of paths) {
      path.setAttribute("pathLength", "1");
      path.style.strokeDasharray = "1";
    }
    const stop = progress.on("change", paint);
    paint();
    const ready = requestAnimationFrame(() => {
      mounted = true;
    });
    onCleanup(() => {
      cancelAnimationFrame(ready);
      stop();
      progress.stop();
    });
  });

  createEffect(() => {
    const to = props.checked ? 1 : 0;
    // First paint snaps; reduced motion drops the stroke drawing (a shape change).
    if (!mounted || prefersReducedMotion()) {
      progress.set(to);
      return;
    }
    animate(progress, to, props.checked ? draw : erase);
  });

  const size = () => props.size ?? 14;
  return (
    <span
      ref={slot}
      aria-hidden="true"
      class={`pw-selection-check ${props.class ?? ""}`}
      style={{ width: `${size()}px`, height: `${size()}px` }}
    >
      <Check size={size()} />
    </span>
  );
}
