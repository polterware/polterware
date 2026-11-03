import {
  For,
  Show,
  children,
  createContext,
  createMemo,
  createSignal,
  createUniqueId,
  onCleanup,
  onMount,
  splitProps,
  useContext,
  type JSX,
} from "solid-js";
/**
 * Site navigation: **Ark UI NavigationMenu** (behavior) + Polter chrome (look), with the
 * shadcn composition. Ark owns hover intent, click, keyboard, focus return, Escape and the
 * measured viewport; contents declared inside their item are portalled into ONE shared
 * viewport, which springs between their sizes like a single morphing panel.
 *
 *   <NavigationMenu aria-label="Main">
 *     <NavigationMenuList>
 *       <NavigationMenuItem value="products">
 *         <NavigationMenuTrigger>Products</NavigationMenuTrigger>
 *         <NavigationMenuContent>…</NavigationMenuContent>
 *       </NavigationMenuItem>
 *       <NavigationMenuItem>
 *         <NavigationMenuLink href="/pricing" class={navigationMenuTriggerClass}>Pricing</NavigationMenuLink>
 *       </NavigationMenuItem>
 *     </NavigationMenuList>
 *   </NavigationMenu>
 *
 * One fluid pill glides across the top-level triggers and rests on the open one.
 * For public site headers; desktop product chrome uses DropdownMenu and Commander.
 */
import { NavigationMenu as ArkNavigationMenu } from "@ark-ui/solid/navigation-menu";
import { createFluidHover, FluidHoverHighlight, type FluidHover } from "./fluid-hover";
import { ChevronDown, type HugeiconComponent } from "./hugeicons";
import "./navigation-menu.css";

/** Classes for a top-level link that sits beside the triggers (shadcn's `navigationMenuTriggerStyle`). */
export const navigationMenuTriggerClass = "pw-navmenu-trigger";

type TopLevelContextValue = {
  hover: FluidHover;
  register: (element: HTMLElement, value?: string) => () => void;
};

const TopLevelContext = createContext<TopLevelContextValue>();
const ItemValueContext = createContext<string>();

const join = (...classes: Array<string | false | undefined>) => classes.filter(Boolean).join(" ");

export type NavigationMenuProps = {
  children: JSX.Element;
  class?: string;
  "aria-label"?: string;
  /** Where the shared viewport opens under the list. */
  viewportAlign?: "start" | "center" | "end";
  value?: string;
  defaultValue?: string;
  onValueChange?: (value: string) => void;
  /** Hover intent before opening (ms). */
  openDelay?: number;
  closeDelay?: number;
};

export function NavigationMenu(props: NavigationMenuProps) {
  const align = () => props.viewportAlign ?? "start";
  return (
    <ArkNavigationMenu.Root
      class={join("pw-navmenu", props.class)}
      aria-label={props["aria-label"]}
      value={props.value}
      defaultValue={props.defaultValue}
      onValueChange={(details) => props.onValueChange?.(details.value)}
      openDelay={props.openDelay ?? 120}
      closeDelay={props.closeDelay ?? 200}
    >
      {props.children}
      <ArkNavigationMenu.ViewportPositioner class="pw-navmenu-positioner" align={align()}>
        <ArkNavigationMenu.Viewport class="pw-navmenu-viewport menu-panel" />
      </ArkNavigationMenu.ViewportPositioner>
    </ArkNavigationMenu.Root>
  );
}

export function NavigationMenuList(props: { children: JSX.Element; class?: string }) {
  let list: HTMLDivElement | undefined;
  const hover = createFluidHover(() => list, { axis: "x", gapClick: false });
  const [values, setValues] = createSignal<ReadonlyMap<number, string>>(new Map());
  let next = 0;

  const register = (element: HTMLElement, value?: string) => {
    const index = next++;
    const unregister = hover.registerItem(index, element);
    if (value) setValues((current) => new Map(current).set(index, value));
    return () => {
      unregister();
      setValues((current) => {
        const copy = new Map(current);
        copy.delete(index);
        return copy;
      });
    };
  };

  return (
    <TopLevelContext.Provider value={{ hover, register }}>
      <ArkNavigationMenu.Context>
        {(api) => {
          // The pill follows the pointer, and rests on the open trigger otherwise.
          const openIndex = createMemo(() => {
            const open = api().value;
            if (!open) return null;
            for (const [index, value] of values()) if (value === open) return index;
            return null;
          });
          const targetIndex = () => hover.activeIndex() ?? openIndex();
          const rect = () => {
            const index = targetIndex();
            return index === null || !hover.isMeasured() ? null : (hover.rects()[index] ?? null);
          };
          return (
            <ArkNavigationMenu.List
              ref={list}
              class={join("pw-navmenu-list", props.class)}
              {...hover.handlers}
            >
              <FluidHoverHighlight
                class="pw-navmenu-pill"
                rect={rect()}
                targetKey={targetIndex()}
                session={hover.session()}
              />
              {props.children}
            </ArkNavigationMenu.List>
          );
        }}
      </ArkNavigationMenu.Context>
    </TopLevelContext.Provider>
  );
}

export function NavigationMenuItem(props: { children: JSX.Element; value?: string; class?: string }) {
  const fallback = createUniqueId();
  const value = () => props.value ?? `item-${fallback}`;
  return (
    <ItemValueContext.Provider value={value()}>
      <ArkNavigationMenu.Item value={value()} class={join("pw-navmenu-item", props.class)}>
        {props.children}
      </ArkNavigationMenu.Item>
    </ItemValueContext.Provider>
  );
}

/** Joins the top-level pill; a no-op outside a NavigationMenuList. */
function useTopLevelRegistration(value?: () => string | undefined) {
  const context = useContext(TopLevelContext);
  return (element: HTMLElement) => {
    if (!context) return;
    onCleanup(context.register(element, value?.()));
  };
}

export function NavigationMenuTrigger(props: { children: JSX.Element; class?: string }) {
  const value = useContext(ItemValueContext);
  const register = useTopLevelRegistration(() => value);
  return (
    <ArkNavigationMenu.Trigger ref={register} class={join(navigationMenuTriggerClass, props.class)}>
      <span class="pw-navmenu-trigger-label">{props.children}</span>
      <ChevronDown class="pw-navmenu-chevron" size={14} aria-hidden="true" />
    </ArkNavigationMenu.Trigger>
  );
}

/**
 * Content mounts on the client only: Ark renders it in place or portals it into the
 * viewport depending on client state, which cannot match server-rendered HTML. It is
 * closed until a trigger opens it, so nothing visible is lost before hydration.
 */
export function NavigationMenuContent(props: { children: JSX.Element; class?: string }) {
  const [mounted, setMounted] = createSignal(false);
  onMount(() => setMounted(true));
  return (
    <Show when={mounted()}>
      <ArkNavigationMenu.Content class={join("pw-navmenu-content", props.class)}>
        {props.children}
      </ArkNavigationMenu.Content>
    </Show>
  );
}

export type NavigationMenuLinkProps = Omit<JSX.AnchorHTMLAttributes<HTMLAnchorElement>, "onSelect"> & {
  href: string;
  /** Marks the link for the current page (`aria-current="page"`). */
  active?: boolean;
  /** Close the menu after activation (default true). */
  closeOnClick?: boolean;
};

/**
 * A native anchor. As a top-level link (with `navigationMenuTriggerClass`) it joins the
 * trigger pill; inside a content it is a plain row the content styles.
 */
export function NavigationMenuLink(props: NavigationMenuLinkProps) {
  const [local, rest] = splitProps(props, ["class", "active", "closeOnClick", "children", "ref"]);
  const topLevel = () => (local.class ?? "").split(" ").includes(navigationMenuTriggerClass);
  const register = useTopLevelRegistration();
  return (
    <ArkNavigationMenu.Link
      {...rest}
      ref={(element: HTMLAnchorElement) => {
        if (topLevel()) register(element);
        if (typeof local.ref === "function") (local.ref as (element: HTMLAnchorElement) => void)(element);
      }}
      current={local.active}
      closeOnClick={local.closeOnClick ?? true}
      class={join(!topLevel() && "pw-navmenu-link", local.class)}
    >
      {local.children}
    </ArkNavigationMenu.Link>
  );
}

export function NavigationMenuIndicator(props: { class?: string }) {
  return <ArkNavigationMenu.Indicator class={join("pw-navmenu-indicator", props.class)} />;
}

export type NavigationMenuPreviewItem = {
  /** Stable id; also the key of its preview. */
  id: string;
  title: string;
  description: string;
  href: string;
  icon?: HugeiconComponent;
  /** Rendered in the preview pane while this row is hovered or focused. */
  preview: () => JSX.Element;
};

/**
 * A content layout after xAI's product menu: rows with a title and a description on the
 * left, and a live preview of the hovered or focused row on the right. The lit row and its
 * preview change together; leaving the rows keeps the last preview.
 */
export function NavigationMenuPreviewPanel(props: {
  items: NavigationMenuPreviewItem[];
  footer?: JSX.Element;
  class?: string;
}) {
  let rows: HTMLDivElement | undefined;
  const hover = createFluidHover(() => rows, { gapClick: false, keepOnLeave: true });
  const activeIndex = () => hover.activeIndex() ?? 0;
  const active = () => props.items[activeIndex()];
  const footer = children(() => props.footer);
  const rect = () => {
    const index = activeIndex();
    return hover.isMeasured() ? (hover.rects()[index] ?? null) : null;
  };

  return (
    <div class={join("pw-navmenu-preview", props.class)}>
      <div
        class="pw-navmenu-preview-rows"
        ref={rows}
        {...hover.handlers}
        // Rows register while the content is still closed (no layout box), so the first
        // measurement can give up; measure again once the pointer or focus arrives.
        onPointerEnter={(event) => {
          if (!hover.isMeasured()) hover.remeasure();
          hover.handlers.onPointerEnter(event);
        }}
        onFocusIn={(event) => {
          if (!hover.isMeasured()) hover.remeasure();
          hover.handlers.onFocusIn(event);
        }}
      >
        <FluidHoverHighlight class="pw-navmenu-row-pill" rect={rect()} targetKey={activeIndex()} />
        <For each={props.items}>
          {(item, index) => (
            <NavigationMenuLink
              href={item.href}
              class="pw-navmenu-preview-row"
              data-active={activeIndex() === index() ? "" : undefined}
              ref={(element: HTMLAnchorElement) => onCleanup(hover.registerItem(index(), element))}
            >
              <span class="pw-navmenu-row-title">
                <Show when={item.icon}>{(Icon) => {
                  const IconComponent = Icon();
                  return <IconComponent size={16} aria-hidden="true" />;
                }}</Show>
                {item.title}
              </span>
              <span class="pw-navmenu-row-description">{item.description}</span>
            </NavigationMenuLink>
          )}
        </For>
      </div>
      <div class="pw-navmenu-preview-pane" aria-hidden="true">
        <Show when={active()} keyed>
          {(item) => <div class="pw-navmenu-preview-frame">{item.preview()}</div>}
        </Show>
      </div>
      <Show when={footer()}>
        <div class="pw-navmenu-preview-footer">{footer()}</div>
      </Show>
    </div>
  );
}
