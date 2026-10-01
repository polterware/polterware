import { createContext, createEffect, createSignal, onCleanup, untrack, useContext, type Accessor } from "solid-js";

/** Portals keep their originating panel owner, including sibling callback dialogs. */
export type PanelInteractionOwner = { enabled: Accessor<boolean>; elements: Set<HTMLElement> };
export const PanelInteractionContext = createContext<PanelInteractionOwner>();
const owners = new Set<PanelInteractionOwner>();
let activationOwner: PanelInteractionOwner | undefined;

export function panelOwnerAt(target: EventTarget | null): PanelInteractionOwner | undefined {
  if (!(target instanceof Node)) return undefined;
  return Array.from(owners).find((owner) => owner.enabled() && Array.from(owner.elements).some((element) => element.contains(target)));
}

export function rememberPanelInteraction(target: EventTarget | null) {
  activationOwner = panelOwnerAt(target);
}

export function registerPanelInteraction(owner: PanelInteractionOwner) {
  owners.add(owner);
  return () => {
    owners.delete(owner);
    if (activationOwner === owner) activationOwner = undefined;
    owner.elements.clear();
  };
}

/** Register only while open; never treat every app dialog as part of a panel. */
export function createPanelPortalOwnership(open: Accessor<boolean>) {
  const context = useContext(PanelInteractionContext);
  const [elements, setElements] = createSignal<HTMLElement[]>([]);
  let current: PanelInteractionOwner | undefined;
  let disposed = false;
  const clear = () => {
    elements().forEach((element) => current?.elements.delete(element));
    current = undefined;
  };
  createEffect(() => {
    const active = open();
    const nodes = elements();
    if (!active) { clear(); return; }
    if (!current) {
      current = untrack(() => context?.enabled() ? context : panelOwnerAt(document.activeElement)
        ?? (activationOwner?.enabled() ? activationOwner : undefined));
    }
    nodes.forEach((element) => current?.elements.add(element));
  });
  onCleanup(() => { disposed = true; clear(); });
  return (element: HTMLElement) => {
    setElements((previous) => [...previous, element]);
    // Ark remounts portal parts on each entry; release detached nodes after DOM insertion.
    queueMicrotask(() => {
      if (disposed) return;
      const nodes = untrack(elements);
      const detached = nodes.filter((node) => !node.isConnected);
      if (!detached.length) return;
      detached.forEach((node) => current?.elements.delete(node));
      setElements(nodes.filter((node) => node.isConnected));
    });
  };
}
