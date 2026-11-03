import { Show, createMemo, createSignal } from "solid-js";
import {
  Button,
  DataTable,
  ActiveFilterBar,
  FilterPopover,
  FileText,
  PageContent,
  PageLoading,
  PageToolbar,
  Plug,
  SearchBar,
  StatusChip,
  matchClauses,
  type FilterField,
} from "@polterware/frontend-design-system";
import { createPageFilters } from "@polterware/feature-shell";
import {
  type WorkspaceCatalogProvider,
  type WorkspaceIntegration,
  type WorkspaceIntegrationResource,
  type WorkspaceIntegrationSyncRun,
} from "@polterware/feature-shell";
import { NodeIcon } from "@polterware/feature-architecture/icons";
import { IntegrationInlineDetail } from "./integration-inline-detail";
import { providerIconKey } from "./provider-icon";
import { ConnectProviderDialog } from "./connect-provider-dialog";
import { matchConnectedIntegrationId } from "./setup-wizard-helpers";
import { IntegrationsCatalogFilterEmpty } from "./integrations-empty";

export { IntegrationInlineDetail } from "./integration-inline-detail";
export { ConnectProviderDialog } from "./connect-provider-dialog";

function toggleValue(current: readonly string[], value: string, checked: boolean): string[] {
  return checked ? [...current.filter((item) => item !== value), value] : current.filter((item) => item !== value);
}

/** One multi-select filter field over the distinct values a table has. */
function valueField(id: string, label: string, values: readonly { id: string; label: string }[]): FilterField {
  const unique = [...new Map(values.filter((value) => value.id).map((value) => [value.id, value])).values()];
  return { id, label, kind: "multi", options: unique.map((value) => ({ id: value.id, label: value.label })) };
}

/** A catalog platform with this project's connection to it, when there is one. */
type PlatformRow = {
  provider: WorkspaceCatalogProvider;
  integration?: WorkspaceIntegration;
  installed: boolean;
  status: "connected" | "signing-in" | "failed" | "not-connected";
};

const STATUS_LABEL: Record<PlatformRow["status"], string> = {
  connected: "Connected",
  "signing-in": "Signing in",
  failed: "Failed",
  "not-connected": "Not connected",
};

const STATUS_TONE: Record<PlatformRow["status"], "healthy" | "syncing" | "danger" | "muted"> = {
  connected: "healthy",
  "signing-in": "syncing",
  failed: "danger",
  "not-connected": "muted",
};

const STATUS_ORDER: PlatformRow["status"][] = ["failed", "connected", "signing-in", "not-connected"];

function platformStatus(provider: WorkspaceCatalogProvider, integration?: WorkspaceIntegration): PlatformRow["status"] {
  if (provider.state === "authenticating") return "signing-in";
  if (provider.state === "failed" || integration?.status === "error") return "failed";
  if (provider.state === "connected" || integration?.status === "connected") return "connected";
  return "not-connected";
}

/**
 * Every platform CLI Polterware knows, in one table: whether its CLI is installed here, whether it
 * is connected to this project, and when its metadata was last refreshed. A row opens the
 * provider's own sign-in flow; a connected one also opens its metadata.
 */
export function IntegrationsView(props: {
  embedded?: boolean;
  projectId?: string;
  providers?: WorkspaceCatalogProvider[];
  integrations?: WorkspaceIntegration[];
  resources?: WorkspaceIntegrationResource[];
  syncRuns?: WorkspaceIntegrationSyncRun[];
  loading?: boolean;
  connectOpen?: boolean;
  connectProviderId?: string;
  onOpenConnect?: (providerId?: string) => void;
  onCloseConnect?: () => void;
  onConnect?: (provider: WorkspaceCatalogProvider) => void;
  onSync?: (integrationId: string) => void;
}) {
  const [selectedId, setSelectedId] = createSignal<string | null>(null);
  const [query, setQuery] = createSignal("");
  const [statusOverrides, setStatusOverrides] = createSignal<Record<string, WorkspaceIntegration["status"]>>({});

  const rows = createMemo<PlatformRow[]>(() => {
    const integrations = (props.integrations ?? []).map((integration) => ({
      ...integration,
      status: statusOverrides()[integration.id] ?? integration.status,
    }));
    const all = (props.providers ?? []).map((provider) => {
      const integration = integrations.find((item) => item.providerId === provider.id);
      return { provider, integration, installed: provider.state !== "cli-unavailable", status: platformStatus(provider, integration) };
    });
    // Connected platforms first, then the installed CLIs, then the rest; the catalog order within each.
    const rank = (row: PlatformRow) => STATUS_ORDER.indexOf(row.status) * 2 + (row.installed ? 0 : 1);
    return all.map((row, index) => ({ row, index }))
      .sort((a, b) => rank(a.row) - rank(b.row) || a.index - b.index)
      .map(({ row }) => row);
  });

  const filters = createPageFilters({
    surface: "platforms",
    fields: () => [
      valueField("category", "Category", rows().map((row) => ({ id: row.provider.category, label: row.provider.category }))),
      valueField("cli", "CLI", rows().map((row) => (row.installed ? { id: "installed", label: "Installed" } : { id: "missing", label: "Not installed" }))),
      valueField("status", "Status", rows().map((row) => ({ id: row.status, label: STATUS_LABEL[row.status] }))),
    ],
  });

  const visible = createMemo(() => {
    const needle = query().trim().toLowerCase();
    return rows().filter((row) => {
      const matches = matchClauses(row, filters.pageClauses(), {
        category: (item) => item.provider.category,
        cli: (item) => (item.installed ? "installed" : "missing"),
        status: (item) => item.status,
      });
      if (!matches) return false;
      if (!needle) return true;
      return [row.provider.name, row.provider.description, row.provider.category, row.provider.binary, row.integration?.environment]
        .some((value) => value?.toLowerCase().includes(needle));
    });
  });

  const selectedIntegration = createMemo(() =>
    (props.integrations ?? []).find((integration) => integration.id === selectedId()),
  );

  // The settings panel is narrower than a page, so the embedded table keeps its own saved widths.
  const widthsKey = () => (props.embedded ? "platforms-settings" : "platforms");

  const actions = () => (
    <>
      <SearchBar placeholder="Search platforms…" aria-label="Search platforms" value={query()} onInput={setQuery} />
      <FilterPopover
        ariaLabel="Platform filters"
        title="Filter platforms"
        shortcut={!props.embedded}
        fields={filters.fields()}
        clauses={filters.clauses()}
        onClausesChange={filters.onClausesChange}
        saved={filters.saved()}
      />
      <Button size="icon" variant="texture" textureTone="neutral" aria-label="Connect platform" onClick={() => props.onOpenConnect?.()}>
        <Plug size={14} aria-hidden />
      </Button>
    </>
  );

  const filterBar = () => (
    <ActiveFilterBar
      fields={filters.fields()}
      clauses={filters.clauses()}
      onClausesChange={filters.onClausesChange}
      saved={filters.saved()}
    />
  );

  const body = () => (
    <Show when={!props.loading} fallback={<PageLoading layout="table" label="Loading platforms…" />}>
      <Show
        when={!selectedIntegration()}
        fallback={
          <IntegrationInlineDetail
            integration={selectedIntegration()!}
            providers={props.providers ?? []}
            resources={props.resources ?? []}
            syncRuns={props.syncRuns ?? []}
            status={statusOverrides()[selectedIntegration()!.id] ?? selectedIntegration()!.status}
            onBack={() => setSelectedId(null)}
            onReconfigure={() => props.onOpenConnect?.(selectedIntegration()!.providerId)}
            onSync={() => props.onSync?.(selectedIntegration()!.id)}
          />
        }
      >
        <Show when={visible().length > 0} fallback={<IntegrationsCatalogFilterEmpty />}>
          <DataTable.Root gutters={false} rows={visible()} getRowId={(row) => row.provider.id} columnWidthsKey={widthsKey()}>
            <DataTable.Table aria-label="Platforms">
              <DataTable.Header>
                <DataTable.ColumnHeader<PlatformRow> id="name" minWidth={160}>Platform</DataTable.ColumnHeader>
                <DataTable.ColumnHeader<PlatformRow> id="cli" width="7.5rem">CLI</DataTable.ColumnHeader>
                <DataTable.ColumnHeader<PlatformRow> id="status" width="8rem">Status</DataTable.ColumnHeader>
                <DataTable.ColumnHeader<PlatformRow> id="lastSync" width="7.5rem">Last refresh</DataTable.ColumnHeader>
                <DataTable.ColumnHeader<PlatformRow> id="actions" width="3rem" sortable={false} resizable={false}><span class="sr-only">Actions</span></DataTable.ColumnHeader>
              </DataTable.Header>
              <DataTable.Body<PlatformRow>>
                {(row) => (
                  <DataTable.Row row={row} rowId={row.provider.id} onActivate={() => props.onOpenConnect?.(row.provider.id)} aria-label={`Open ${row.provider.name} CLI flow`}>
                    <DataTable.PrimaryCell
                      icon={<NodeIcon name={providerIconKey(row.provider)} size={16} brand={false} />}
                      iconTone="integration"
                      iconBrand={[row.provider.name, providerIconKey(row.provider)]}
                      iconSeed={row.provider.id}
                      title={row.provider.name}
                    />
                    <DataTable.Cell>
                      <StatusChip tone={row.installed ? "healthy" : "unknown"} label={row.installed ? "Installed" : "Not installed"} />
                    </DataTable.Cell>
                    <DataTable.Cell><StatusChip tone={STATUS_TONE[row.status]} label={STATUS_LABEL[row.status]} /></DataTable.Cell>
                    <DataTable.Cell class="text-[var(--color-text-muted)]">{row.integration?.lastSync || "—"}</DataTable.Cell>
                    <DataTable.Cell>
                      <Show when={row.integration}>
                        {(integration) => (
                          <Button size="icon" variant="ghost" aria-label={`View ${row.provider.name} metadata`} onClick={(event) => {
                            event.stopPropagation();
                            setSelectedId(integration().id);
                          }}>
                            <FileText size={14} aria-hidden />
                          </Button>
                        )}
                      </Show>
                    </DataTable.Cell>
                  </DataTable.Row>
                )}
              </DataTable.Body>
            </DataTable.Table>
          </DataTable.Root>
        </Show>
      </Show>
    </Show>
  );

  return (
    <>
      <Show when={!selectedIntegration()}>
        <Show
          when={!props.embedded}
          fallback={<><div class="flex items-center justify-end gap-2">{actions()}</div>{filterBar()}</>}
        >
          <PageToolbar.Root>
            <PageToolbar.Actions>{actions()}</PageToolbar.Actions>
          </PageToolbar.Root>
          {filterBar()}
        </Show>
      </Show>
      <Show
        when={!props.embedded}
        fallback={<div class="flex min-w-0 flex-col gap-4">{body()}</div>}
      >
        <PageContent class={`flex min-h-0 flex-1 flex-col gap-4 overflow-auto ${selectedIntegration() ? "p-4" : "p-0"}`}>
          {body()}
        </PageContent>
      </Show>
      <ConnectProviderDialog
        open={props.connectOpen ?? false}
        projectId={props.projectId}
        providerName={props.connectProviderId}
        providers={props.providers ?? []}
        onClose={() => props.onCloseConnect?.()}
        onFinish={(provider) => {
          const integrationId = matchConnectedIntegrationId(provider.name, props.integrations ?? []);
          props.onCloseConnect?.();
          props.onConnect?.(provider);
          if (integrationId) {
            setStatusOverrides((current) => ({ ...current, [integrationId]: "connected" }));
          }
        }}
      />
    </>
  );
}
