import { createSignal, getOwner, onCleanup, type Accessor } from "solid-js";
import { createStore } from "zustand/vanilla";

export function createSelectionStore() {
  const [systemId, setSystemId] = createSignal<string | null>("sys-local");
  const [panel, setPanel] = createSignal<"architecture" | "ops" | "logs" | "settings" | "knowledge">("architecture");
  return { systemId, setSystemId, panel, setPanel };
}

export type DetailPanelMode = "open" | "collapsed" | "maximized";

export type DetailPanelPresence = "hidden" | "visible" | "dismissing";

export type FixtureMode = "force" | "fallback" | "off";

export type DetailTarget = {
  scopeId: string;
  entityId: string;
  entityKind: string;
};

export type DetailCollectionStatus = "idle" | "loading" | "ready" | "error";

export type DetailPanelPlacement = "bottom" | "side";

export type DetailCollectionRegistration = {
  scopeId: string;
  status: DetailCollectionStatus;
  targets: readonly DetailTarget[];
  /** Where the shared DetailPanel docks while this collection is active. */
  placement?: DetailPanelPlacement;
  /**
   * Closing the panel dismisses it instead of collapsing it to the header strip, and
   * nothing opens until the user picks an entity. The panel is only open or maximized.
   */
  dismissible?: boolean;
};

type DetailPanelSessionState = {
  mode: DetailPanelMode;
  placement: DetailPanelPlacement;
  dismissible: boolean;
  activeScope: string | null;
  activeTarget: DetailTarget | null;
  outgoingTarget: DetailTarget | null;
  pendingTarget: DetailTarget | null;
  collectionStatus: DetailCollectionStatus;
  restoreMode: "open" | "collapsed";
  presence: DetailPanelPresence;
};

export type DetailPanelState = DetailPanelSessionState & {
  registerCollection: (collection: DetailCollectionRegistration) => void;
  unregisterCollection: (scopeId: string) => void;
  requestTarget: (target: DetailTarget) => void;
  activateTarget: (target: DetailTarget) => void;
  dismissTarget: (scopeId: string) => void;
  completeDismissal: (scopeId: string) => void;
  collapse: () => void;
  maximize: () => void;
  restore: () => void;
};

export const detailPanelDefaultState: DetailPanelSessionState = {
  mode: "open",
  placement: "bottom",
  dismissible: false,
  activeScope: null,
  activeTarget: null,
  outgoingTarget: null,
  pendingTarget: null,
  collectionStatus: "idle",
  restoreMode: "open",
  presence: "hidden",
};

function targetsMatch(left: DetailTarget | null, right: DetailTarget | null): boolean {
  return left !== null
    && right !== null
    && left.scopeId === right.scopeId
    && left.entityId === right.entityId
    && left.entityKind === right.entityKind;
}

function containsTarget(targets: readonly DetailTarget[], target: DetailTarget | null, scopeId: string): target is DetailTarget {
  return target !== null
    && target.scopeId === scopeId
    && targets.some((candidate) => targetsMatch(candidate, target));
}

/** Open/collapsed chrome preference for the session; never returns maximized. */
function sessionLayoutMode(state: DetailPanelSessionState): "open" | "collapsed" {
  if (state.mode === "maximized") {
    return state.restoreMode === "collapsed" ? "collapsed" : "open";
  }
  return state.mode === "collapsed" ? "collapsed" : "open";
}

export const detailPanelStore = createStore<DetailPanelState>()(
  (set, get) => ({
      ...detailPanelDefaultState,

      registerCollection: ({ scopeId, status, targets, placement = "bottom", dismissible = false }) => {
        if (get().placement !== placement) set({ placement });
        if (get().dismissible !== dismissible) set({ dismissible });
        const state = get();

        if (status === "loading") {
          const preservesExplicitEmptySelection = state.activeScope === scopeId
            && state.activeTarget === null
            && (state.presence === "hidden" || state.presence === "dismissing");
          if (preservesExplicitEmptySelection) {
            set({
              activeScope: scopeId,
              activeTarget: null,
              collectionStatus: "loading",
              mode: "collapsed",
            });
            return;
          }
          const isScopeChange = state.activeScope !== scopeId;
          const layoutMode = sessionLayoutMode(state);
          // Keep the panel off-screen during loading so the open animation does
          // not compete with workspace hydrate / window show. Reveal on ready.
          // Same-scope refresh while already visible keeps the panel open.
          const nextPresence =
            !isScopeChange && state.presence === "visible" ? "visible" : "hidden";
          set({
            activeScope: scopeId,
            activeTarget: null,
            collectionStatus: "loading",
            mode: isScopeChange ? layoutMode : state.mode,
            restoreMode: isScopeChange ? layoutMode : state.restoreMode,
            outgoingTarget: null,
            presence: nextPresence,
          });
          return;
        }

        if (status === "ready") {
          if (targets.length === 0) {
            const outgoingTarget = state.activeTarget?.scopeId === scopeId
              ? state.activeTarget
              : state.outgoingTarget?.scopeId === scopeId
                ? state.outgoingTarget
                : null;
            set({
              activeScope: scopeId,
              activeTarget: null,
              outgoingTarget,
              pendingTarget: state.pendingTarget?.scopeId === scopeId ? null : state.pendingTarget,
              collectionStatus: "ready",
              mode: "collapsed",
              presence: outgoingTarget ? "dismissing" : "hidden",
            });
            return;
          }

          const pendingTarget = containsTarget(targets, state.pendingTarget, scopeId) ? state.pendingTarget : null;
          const activeTarget = containsTarget(targets, state.activeTarget, scopeId) ? state.activeTarget : null;
          const remainingPendingTarget = state.pendingTarget?.scopeId === scopeId
            ? null
            : state.pendingTarget;
          const isScopeChange = state.activeScope !== scopeId;
          const keepsExplicitEmptySelection = !isScopeChange
            && state.activeTarget === null
            && (state.presence === "hidden" || state.presence === "dismissing");
          const nextTarget = pendingTarget
            ?? activeTarget
            ?? (keepsExplicitEmptySelection || dismissible ? null : targets[0]!);

          if (!nextTarget) {
            set({
              activeScope: scopeId,
              activeTarget: null,
              pendingTarget: remainingPendingTarget,
              collectionStatus: "ready",
              mode: "collapsed",
              restoreMode: isScopeChange ? sessionLayoutMode(state) : state.restoreMode,
              presence: isScopeChange ? "hidden" : state.presence,
            });
            return;
          }

          // A dismissible panel is never shown collapsed: it opens or stays maximized.
          const layoutMode = dismissible ? "open" : sessionLayoutMode(state);
          const keptMode = dismissible && state.mode === "collapsed" ? "open" : state.mode;

          set({
            activeScope: scopeId,
            activeTarget: nextTarget,
            pendingTarget: pendingTarget ? null : remainingPendingTarget,
            collectionStatus: "ready",
            mode: isScopeChange ? layoutMode : keptMode,
            restoreMode: isScopeChange ? layoutMode : state.restoreMode,
            outgoingTarget: null,
            presence: "visible",
          });
          return;
        }

        if (status === "error") {
          set({
            activeScope: scopeId,
            activeTarget: state.activeTarget?.scopeId === scopeId ? state.activeTarget : null,
            collectionStatus: "error",
            mode: "open",
            outgoingTarget: null,
            presence: "visible",
          });
          return;
        }

        set({
          activeScope: scopeId,
          collectionStatus: "idle",
        });
      },

      unregisterCollection: (scopeId) => {
        const state = get();
        if (state.activeScope !== scopeId) {
          return;
        }

        const pendingTarget = state.pendingTarget?.scopeId === scopeId ? null : state.pendingTarget;
        const layoutMode = sessionLayoutMode(state);
        set({
          activeScope: null,
          activeTarget: null,
          outgoingTarget: null,
          pendingTarget,
          placement: "bottom",
          dismissible: false,
          collectionStatus: "idle",
          mode: pendingTarget ? "open" : layoutMode,
          restoreMode: pendingTarget ? "open" : layoutMode,
          presence: pendingTarget ? "visible" : "hidden",
        });
      },

      requestTarget: (target) => {
        set({
          pendingTarget: target,
          mode: "open",
          outgoingTarget: null,
          presence: "visible",
        });
      },

      activateTarget: (target) => {
        const state = get();
        set({
          activeScope: target.scopeId,
          activeTarget: target,
          pendingTarget: null,
          outgoingTarget: null,
          presence: "visible",
          mode: state.mode === "maximized" ? "maximized" : "open",
          restoreMode: state.mode === "maximized" ? state.restoreMode : "open",
        });
      },

      dismissTarget: (scopeId) => {
        const state = get();
        const outgoingTarget = state.activeTarget?.scopeId === scopeId
          ? state.activeTarget
          : state.presence === "dismissing" && state.outgoingTarget?.scopeId === scopeId
            ? state.outgoingTarget
            : null;
        if (!outgoingTarget) {
          return;
        }
        set({
          outgoingTarget,
          activeTarget: null,
          pendingTarget: state.pendingTarget?.scopeId === scopeId ? null : state.pendingTarget,
          mode: "collapsed",
          presence: "dismissing",
        });
      },

      completeDismissal: (scopeId) => {
        const state = get();
        if (
          state.presence === "dismissing"
          && state.activeTarget === null
          && state.outgoingTarget?.scopeId === scopeId
        ) {
          set({
            outgoingTarget: null,
            presence: "hidden",
          });
        }
      },

      collapse: () => {
        const state = get();
        if (state.dismissible) {
          if (state.activeScope) state.dismissTarget(state.activeScope);
          return;
        }
        set({ mode: "collapsed", restoreMode: "collapsed" });
      },

      maximize: () => {
        const state = get();
        if (state.activeScope === null) {
          return;
        }

        set({
          mode: "maximized",
          restoreMode: state.mode === "collapsed" ? "collapsed" : "open",
        });
      },

      restore: () => {
        const state = get();
        if (state.mode === "maximized") {
          const next = state.restoreMode === "collapsed" ? "collapsed" : "open";
          set({ mode: next, restoreMode: next });
          return;
        }
        set({ mode: "open", restoreMode: "open" });
      },
    }),
);

export function createDetailPanelSelector<T>(
  selector: (state: DetailPanelState) => T,
  equals: (left: T, right: T) => boolean = Object.is,
): Accessor<T> {
  let current = selector(detailPanelStore.getState());
  const [selected, setSelected] = createSignal(current, { equals: false });
  const unsubscribe = detailPanelStore.subscribe((state) => {
    const next = selector(state);
    if (equals(current, next)) {
      return;
    }

    current = next;
    setSelected(() => next);
  });

  if (getOwner()) {
    onCleanup(unsubscribe);
  }

  return selected;
}

export * from "./shortcuts";
