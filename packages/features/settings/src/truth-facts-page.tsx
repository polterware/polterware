import { createSignal } from "solid-js";
import {
  Button,
  PageContent,
  PageToolbar,
  RefreshCw,
  Tabs,
  type TabItem,
} from "@polterware/frontend-design-system";
import { PpmFactsPanel, type PpmJustifyChange } from "./ppm-facts-panel";

/** Truth → Facts: PPM facts and their evidence. */
export function TruthFactsPage(props: {
  projectId: string;
  tab: string;
  tabs: TabItem[];
  tabsLabel: string;
  onTabChange?: (tab: string) => void;
  justifyChange: PpmJustifyChange;
}) {
  const [entityId, setEntityId] = createSignal("");
  const [busy, setBusy] = createSignal(false);
  const [refreshToken, setRefreshToken] = createSignal(0);

  return (
    <div class="flex h-full min-h-0 flex-col">
      <PageToolbar.Root>
        <PageToolbar.Tabs>
          <Tabs
            items={[...props.tabs]}
            value={props.tab}
            onChange={(value) => props.onTabChange?.(value)}
            aria-label={props.tabsLabel}
          />
        </PageToolbar.Tabs>
        <PageToolbar.Actions>
          <Button
            size="sm"
            variant="ghost"
            disabled={busy()}
            onClick={() => setRefreshToken((value) => value + 1)}
          >
            <RefreshCw size={14} />
            Refresh
          </Button>
        </PageToolbar.Actions>
      </PageToolbar.Root>
      <PageContent class="flex min-h-0 flex-1 flex-col p-0">
        <PpmFactsPanel
          projectId={props.projectId}
          entityFilter={entityId()}
          busy={busy()}
          refreshToken={refreshToken()}
          justifyChange={props.justifyChange}
          onEntityFilterChange={setEntityId}
          onBusyChange={setBusy}
        />
      </PageContent>
    </div>
  );
}
