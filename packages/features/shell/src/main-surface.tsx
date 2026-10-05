import { Show, type JSX } from "solid-js";
export { PageHeader, PageContent, PageToolbar } from "@polterware/frontend-design-system";
import {
  DetailPanel,
  PageEmpty,
  PageError,
  PageSkeleton,
} from "@polterware/frontend-design-system";
import { MainSurfaceFrame } from "@polterware/frontend-shell";
import { createDetailPanelSelector, detailPanelStore } from "@polterware/frontend-state";

/**
 * Large rounded content host — canonical `surface-panel` (edge-light rim).
 * See DESIGN.md Materials · surface-panel.
 */
export function MainSurface(props: { children: JSX.Element; class?: string }) {
  const open = createDetailPanelSelector((state) => state.open);
  const expanded = createDetailPanelSelector((state) => state.expanded);
  const available = createDetailPanelSelector((state) => state.available);
  const activeTarget = createDetailPanelSelector((state) => state.activeTarget);
  const collectionStatus = createDetailPanelSelector((state) => state.collectionStatus);

  return (
    <MainSurfaceFrame class={props.class}>
      <DetailPanel.Root
        open={open()}
        expanded={expanded()}
        available={available()}
        activeEntityId={activeTarget()?.entityId}
        onOpenChange={(next) => {
          const state = detailPanelStore.getState();
          if (next) state.show();
          else state.hide();
        }}
        onExpandedChange={(next) => {
          const state = detailPanelStore.getState();
          if (next) state.expand();
          else state.restore();
        }}
        storageKey="polterware.shell.detail.sidebar.width"
        legacyStorageKey="polterware.shell.detail.width"
        aria-label="Entity details"
      >
        <DetailPanel.Collection>{props.children}</DetailPanel.Collection>
        <Show when={collectionStatus() === "loading" && !activeTarget()}>
          <DetailPanel.Content>
            <DetailPanel.Body>
              <DetailPanel.Main>
                <PageSkeleton layout="detail" label="Loading details…" class="min-h-0 p-0" />
              </DetailPanel.Main>
            </DetailPanel.Body>
          </DetailPanel.Content>
        </Show>
        <Show when={collectionStatus() === "error" && !activeTarget()}>
          <DetailPanel.Content>
            <DetailPanel.Body layout="document">
              <PageError message="The selected details could not be loaded." />
            </DetailPanel.Body>
          </DetailPanel.Content>
        </Show>
        <Show when={(collectionStatus() === "ready" || collectionStatus() === "idle") && !activeTarget()}>
          <DetailPanel.Content>
            <DetailPanel.Body layout="document">
              <PageEmpty title="No selection" description="Select an item to see its details." />
            </DetailPanel.Body>
          </DetailPanel.Content>
        </Show>
      </DetailPanel.Root>
    </MainSurfaceFrame>
  );
}
