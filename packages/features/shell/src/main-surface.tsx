import { Show, type JSX } from "solid-js";
export { PageHeader, PageContent, PageToolbar } from "@polterware/frontend-design-system";
import {
  DetailPanel,
  PageError,
} from "@polterware/frontend-design-system";
import { createDetailPanelSelector, detailPanelStore } from "@polterware/frontend-state";
import "./main-surface.css";

/**
 * Large rounded content host — canonical `surface-panel` (edge-light rim).
 * See DESIGN.md Materials · surface-panel.
 */
export function MainSurface(props: { children: JSX.Element; class?: string }) {
  const mode = createDetailPanelSelector((state) => state.mode);
  const placement = createDetailPanelSelector((state) => state.placement);
  const presence = createDetailPanelSelector((state) => state.presence);
  const activeTarget = createDetailPanelSelector((state) => state.activeTarget);
  const outgoingTarget = createDetailPanelSelector((state) => state.outgoingTarget);
  const collectionStatus = createDetailPanelSelector((state) => state.collectionStatus);

  const detailTarget = () => activeTarget() ?? outgoingTarget();

  return (
    <main
      class={`pw-main-surface surface-panel flex min-h-0 min-w-0 flex-1 flex-col overflow-hidden ${props.class ?? ""}`}
    >
      <DetailPanel.Root
        mode={mode()}
        placement={placement()}
        presence={presence()}
        activeEntityId={detailTarget()?.entityId}
        onCollapse={() => detailPanelStore.getState().collapse()}
        onMaximize={() => detailPanelStore.getState().maximize()}
        onRestore={() => detailPanelStore.getState().restore()}
        onDismissComplete={() => {
          const t = detailTarget();
          if (t) detailPanelStore.getState().completeDismissal(t.scopeId);
        }}
        aria-label="Entity details"
      >
        <DetailPanel.Collection>{props.children}</DetailPanel.Collection>
        <Show when={collectionStatus() === "loading"}>
          <DetailPanel.Content>
            <DetailPanel.Header title="Loading details" />
            <DetailPanel.Body>
              <DetailPanel.Main>
                <div
                  class="space-y-3"
                  role="status"
                  aria-label="Loading details"
                  aria-live="polite"
                >
                  <span class="sr-only">Loading details…</span>
                  <div class="h-4 w-48 rounded-[var(--radius-control)] bg-[var(--color-surface-subtle)]" />
                  <div class="h-2 w-32 rounded-[var(--radius-control)] bg-[var(--color-surface-subtle)]" />
                  <div class="h-24 rounded-[var(--radius-control)] bg-[var(--color-surface-subtle)]" />
                </div>
              </DetailPanel.Main>
              <DetailPanel.Rail>
                <div class="h-32 rounded-[var(--radius-control)] bg-[var(--color-surface-subtle)]" />
              </DetailPanel.Rail>
            </DetailPanel.Body>
          </DetailPanel.Content>
        </Show>
        <Show when={collectionStatus() === "error" && !activeTarget()}>
          <DetailPanel.Content>
            <DetailPanel.Header title="Details unavailable" />
            <DetailPanel.Body>
              <DetailPanel.Main>
                <PageError message="The selected details could not be loaded." />
              </DetailPanel.Main>
            </DetailPanel.Body>
          </DetailPanel.Content>
        </Show>
      </DetailPanel.Root>
    </main>
  );
}
