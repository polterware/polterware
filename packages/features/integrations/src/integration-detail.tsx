import { For, Show, createEffect, createMemo, createSignal } from "solid-js";
import {
  DataTable,
  DetailPanel,
  Plug,
  RefreshCw,
  StatusChip,
  Tabs,
  pushToast,
} from "@polterware/frontend-design-system";
import {
  TerminalTranscript,
  type WorkspaceCatalogProvider,
  type WorkspaceIntegration,
  type WorkspaceIntegrationResource,
  type WorkspaceIntegrationSyncRun,
} from "@polterware/feature-shell";
import { NodeIcon } from "@polterware/feature-architecture/icons";
import { providerIconKey } from "./provider-icon";
import { cliStateLabel, cliStateTone } from "./setup-wizard-helpers";

const DETAIL_SECTIONS = [
  { id: "overview", label: "Overview" },
  { id: "resources", label: "Resources" },
  { id: "cli", label: "CLI" },
  { id: "sync", label: "Sync" },
] as const;

export type IntegrationDetailSection = (typeof DETAIL_SECTIONS)[number]["id"];

const DETAIL_SECTION_IDS = {
  overview: true,
  resources: true,
  cli: true,
  sync: true,
} satisfies Record<IntegrationDetailSection, boolean>;

function isIntegrationDetailSection(value: string): value is IntegrationDetailSection {
  return Object.prototype.hasOwnProperty.call(DETAIL_SECTION_IDS, value);
}

function integrationTone(status: WorkspaceIntegration["status"]): "healthy" | "danger" | "warn" | "muted" {
  if (status === "connected") return "healthy";
  if (status === "error") return "danger";
  if (status === "disabled") return "warn";
  return "muted";
}

function syncTone(result: WorkspaceIntegrationSyncRun["result"]): "healthy" | "warn" | "danger" | "syncing" {
  if (result === "success") return "healthy";
  if (result === "partial") return "warn";
  if (result === "failed") return "danger";
  return "syncing";
}

export function IntegrationDetail(props: {
  integration: WorkspaceIntegration | null | undefined;
  providers?: WorkspaceCatalogProvider[];
  resources?: WorkspaceIntegrationResource[];
  syncRuns?: WorkspaceIntegrationSyncRun[];
  status?: WorkspaceIntegration["status"];
  section?: IntegrationDetailSection;
  onSync?: () => void;
  onReconfigure?: () => void;
  onOpenResource?: (resourceId: string) => void;
}) {
  const [section, setSection] = createSignal<IntegrationDetailSection>("overview");
  createEffect(() => props.section && setSection(props.section));

  const resources = createMemo(() => (props.resources ?? []).filter(
    (resource) => resource.integrationId === props.integration?.id,
  ));
  const syncRuns = createMemo(() => (props.syncRuns ?? []).filter(
    (run) => run.integrationId === props.integration?.id,
  ));
  const provider = createMemo(() => (props.providers ?? []).find(
    (item) => item.id === props.integration?.providerId,
  ));

  return (
    <Show when={props.integration}>
      {(integration) => (
        <DetailPanel.Content>
          <DetailPanel.Header
            icon={<NodeIcon name={providerIconKey(integration())} size={16} />}
            title={integration().name}
          >
            <DetailPanel.Actions>
              <DetailPanel.Action label="Refresh fixture" onClick={() => {
                props.onSync?.();
                pushToast("Fixture metadata refreshed in memory.", "success");
              }}>
                <RefreshCw />
              </DetailPanel.Action>
              <DetailPanel.Action label="Open CLI flow" onClick={() => props.onReconfigure?.()}>
                <Plug />
              </DetailPanel.Action>
            </DetailPanel.Actions>
          </DetailPanel.Header>
          <DetailPanel.Body layout="sections">
            <DetailPanel.Grid>
              <DetailPanel.Section>
                <Tabs items={[...DETAIL_SECTIONS]} value={section()} onChange={(value) => {
                  if (isIntegrationDetailSection(value)) setSection(value);
                }} aria-label="Integration detail" />
              </DetailPanel.Section>

              <Show when={section() === "overview"}>
                <DetailPanel.Section title="Connection">
                  <div class="grid gap-4 sm:grid-cols-2">
                    <div><p class="m-0 text-[11px] text-[var(--color-text-muted)]">Status</p><StatusChip tone={integrationTone(props.status ?? integration().status)} label={props.status ?? integration().status} /></div>
                    <div><p class="m-0 text-[11px] text-[var(--color-text-muted)]">Connection fixture</p><p class="m-0 mt-1 text-[13px]">{integration().connectionLabel}</p></div>
                    <div><p class="m-0 text-[11px] text-[var(--color-text-muted)]">Environment</p><p class="m-0 mt-1 text-[13px]">{integration().environment}</p></div>
                    <div><p class="m-0 text-[11px] text-[var(--color-text-muted)]">Last fixture refresh</p><p class="m-0 mt-1 text-[13px]">{integration().lastSync}</p></div>
                  </div>
                  <p class="m-0 mt-4 text-[12px] text-[var(--color-text-secondary)]">{integration().summary}</p>
                </DetailPanel.Section>
              </Show>

              <Show when={section() === "resources"}>
                <DetailPanel.Section title="Imported resources">
                  <DataTable.Root rows={resources()} getRowId={(item) => item.id}>
                    <DataTable.Table aria-label={`${integration().name} resources`}>
                      <DataTable.Header>
                        <DataTable.ColumnHeader<WorkspaceIntegrationResource> id="name">Resource</DataTable.ColumnHeader>
                        <DataTable.ColumnHeader<WorkspaceIntegrationResource> id="kind">Kind</DataTable.ColumnHeader>
                        <DataTable.ColumnHeader<WorkspaceIntegrationResource> id="environment">Environment</DataTable.ColumnHeader>
                        <DataTable.ColumnHeader<WorkspaceIntegrationResource> id="status">Status</DataTable.ColumnHeader>
                      </DataTable.Header>
                      <DataTable.Body<WorkspaceIntegrationResource>>
                        {(resource) => (
                          <DataTable.Row row={resource} rowId={resource.id} onActivate={() => resource.inventoryResourceId && props.onOpenResource?.(resource.inventoryResourceId)}>
                            <DataTable.PrimaryCell icon={<Plug size={16} />} iconTone="integration" title={resource.name} />
                            <DataTable.Cell>{resource.kind}</DataTable.Cell>
                            <DataTable.Cell>{resource.environment}</DataTable.Cell>
                            <DataTable.Cell><StatusChip tone="healthy" label={resource.status} /></DataTable.Cell>
                          </DataTable.Row>
                        )}
                      </DataTable.Body>
                    </DataTable.Table>
                  </DataTable.Root>
                </DetailPanel.Section>
              </Show>

              <Show when={section() === "cli" && provider()}>
                {(item) => (
                  <DetailPanel.Section title="CLI authentication">
                    <div class="mb-3 flex items-center justify-between gap-3">
                      <code class="text-[12px]">{item().authCommands.join(" · ")}</code>
                      <StatusChip tone={cliStateTone(item().state)} label={cliStateLabel(item().state)} />
                    </div>
                    <TerminalTranscript lines={item().transcript} label={`${item().name} CLI transcript`} class="min-h-[220px] rounded-[var(--radius-card)]" />
                    <p class="m-0 mt-3 text-[11px] text-[var(--color-text-muted)]">Visual transcript only. No command, browser, PTY, or authentication request is started.</p>
                  </DetailPanel.Section>
                )}
              </Show>

              <Show when={section() === "sync"}>
                <DetailPanel.Section title="Fixture refresh history">
                  <div class="space-y-3">
                    <For each={syncRuns()}>{(run) => (
                      <div class="flex flex-wrap items-center justify-between gap-3">
                        <div><p class="m-0 text-[13px]">{run.message}</p><p class="m-0 mt-1 text-[11px] text-[var(--color-text-muted)]">{run.startedAt} · {run.imported} items</p></div>
                        <StatusChip tone={syncTone(run.result)} label={run.result} />
                      </div>
                    )}</For>
                  </div>
                </DetailPanel.Section>
              </Show>
            </DetailPanel.Grid>
          </DetailPanel.Body>
        </DetailPanel.Content>
      )}
    </Show>
  );
}
