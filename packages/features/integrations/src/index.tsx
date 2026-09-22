import { Show, createEffect, createMemo, createSignal, onCleanup } from "solid-js";
import {
  DataTable,
  FilterPopover,
  LibraryBig,
  PageContent,
  PageToolbar,
  ShieldCheck,
  StatusChip,
  Tabs,
  type TabItem,
} from "@polterware/frontend-design-system";
import {
  type WorkspaceCatalogProvider,
  type WorkspaceIntegration,
  type WorkspaceIntegrationResource,
  type WorkspaceIntegrationSyncRun,
} from "@polterware/feature-shell";
import { NodeIcon } from "@polterware/feature-architecture/icons";
import { createDetailPanelSelector, detailPanelStore } from "@polterware/frontend-state";
import { IntegrationDetail } from "./integration-detail";
import { providerIconKey } from "./provider-icon";
import { SetupWizard } from "./setup-wizard";
import { cliStateLabel, cliStateTone, matchConnectedIntegrationId } from "./setup-wizard-helpers";
import {
  IntegrationsCatalogFilterEmpty,
  IntegrationsConnectedEmpty,
} from "./integrations-empty";

export { IntegrationDetail } from "./integration-detail";
export { SetupWizard } from "./setup-wizard";

const TABS: TabItem[] = [
  {
    id: "connected",
    label: "Connected",
    tooltip: {
      title: "Connected providers",
      description:
        "Providers already authenticated via their CLI. Credentials stay in the OS/CLI store, not SQLite.",
      icon: ShieldCheck,
    },
  },
  {
    id: "catalog",
    label: "Catalog",
    tooltip: {
      title: "Provider catalog",
      description:
        "Fixed list: GitHub, GitLab, AWS, Google Cloud, Azure, Vercel, Supabase, Terraform, and Docker. Detect CLI, run auth in a core PTY, probe connection.",
      icon: LibraryBig,
    },
  },
];

function normalizeTab(value?: string): "connected" | "catalog" {
  return value === "catalog" ? "catalog" : "connected";
}

function integrationTone(status: WorkspaceIntegration["status"]): "healthy" | "danger" | "warn" | "muted" {
  if (status === "connected") return "healthy";
  if (status === "error") return "danger";
  if (status === "disabled") return "warn";
  return "muted";
}

export function IntegrationsView(props: {
  projectId?: string;
  providers?: WorkspaceCatalogProvider[];
  integrations?: WorkspaceIntegration[];
  resources?: WorkspaceIntegrationResource[];
  syncRuns?: WorkspaceIntegrationSyncRun[];
  tab?: string;
  onTabChange?: (tab: string) => void;
  onOpen?: (id: string) => void;
  setupOpen?: boolean;
  setupProvider?: string;
  onOpenSetup?: (providerId?: string) => void;
  onCloseSetup?: () => void;
  onOpenResource?: (resourceId: string) => void;
  onConnect?: (provider: WorkspaceCatalogProvider) => void;
  onSync?: (integrationId: string) => void;
}) {
  const tab = () => normalizeTab(props.tab);
  const [localSetupProvider, setLocalSetupProvider] = createSignal<string | null>(null);
  const [query, setQuery] = createSignal("");
  const [categories, setCategories] = createSignal<readonly string[]>([]);
  const [statusOverrides, setStatusOverrides] = createSignal<Record<string, WorkspaceIntegration["status"]>>({});
  const scopeId = createMemo(() => `integrations:${tab()}`);
  const activeTarget = createDetailPanelSelector((state) => state.activeTarget);
  const setupOpen = () => props.setupOpen ?? localSetupProvider() !== null;
  const setupProvider = () => props.setupProvider ?? localSetupProvider() ?? undefined;

  const catalog = createMemo(() => {
    const needle = query().trim().toLowerCase();
    return (props.providers ?? []).filter((provider) => {
      if (categories().length > 0 && !categories().includes(provider.category)) return false;
      if (!needle) return true;
      return [provider.name, provider.description, provider.category, provider.binary]
        .some((value) => value.toLowerCase().includes(needle));
    });
  });

  const activeIntegration = createMemo(() => {
    const target = activeTarget();
    return target?.scopeId === "integrations:connected" && target.entityKind === "integration"
      ? (props.integrations ?? []).find((integration) => integration.id === target.entityId)
      : undefined;
  });

  createEffect(() => {
    const scope = scopeId();
    detailPanelStore.getState().registerCollection({
      scopeId: scope,
      status: "ready",
      targets: !setupOpen() && tab() === "connected"
        ? (props.integrations ?? []).map((integration) => ({ scopeId: scope, entityId: integration.id, entityKind: "integration" }))
        : [],
    });
    onCleanup(() => detailPanelStore.getState().unregisterCollection(scope));
  });

  function openSetup(providerId: string) {
    setLocalSetupProvider(providerId);
    props.onOpenSetup?.(providerId);
  }

  function closeSetup() {
    setLocalSetupProvider(null);
    props.onCloseSetup?.();
  }

  function activateIntegration(id: string) {
    detailPanelStore.getState().activateTarget({
      scopeId: "integrations:connected",
      entityId: id,
      entityKind: "integration",
    });
    props.onOpen?.(id);
  }

  return (
    <>
      <Show when={setupOpen()} fallback={
        <div class="flex h-full min-h-0 flex-col">
          <PageToolbar.Root>
            <PageToolbar.Tabs>
              <Tabs items={[...TABS]} value={tab()} onChange={(value) => props.onTabChange?.(value)} aria-label="Integrations" />
            </PageToolbar.Tabs>
            <Show when={tab() === "catalog"}>
              <PageToolbar.Actions>
                <FilterPopover
                  ariaLabel="Integration catalog filters"
                  title="Provider category"
                  search={{ placeholder: "Search providers…", value: query(), onInput: setQuery }}
                  active={categories().length > 0}
                  onClear={() => setCategories([])}
                  groups={[{
                    id: "category",
                    label: "Category",
                    options: [...new Set((props.providers ?? []).map((provider) => provider.category))].map((category) => ({
                      id: category,
                      label: category,
                      checked: categories().includes(category),
                      onCheckedChange: (checked: boolean) => setCategories((current) =>
                        checked ? [...current, category] : current.filter((value) => value !== category),
                      ),
                    })),
                  }]}
                />
              </PageToolbar.Actions>
            </Show>
          </PageToolbar.Root>

          <PageContent class="p-0">
            <Show when={tab() === "connected"}>
              <Show
                when={(props.integrations ?? []).length > 0}
                fallback={
                  <IntegrationsConnectedEmpty
                    onOpenCatalog={() => props.onTabChange?.("catalog")}
                  />
                }
              >
                <DataTable.Root
                  rows={props.integrations ?? []}
                  getRowId={(item) => item.id}
                  activeRowId={activeTarget()?.scopeId === "integrations:connected" ? activeTarget()?.entityId : null}
                  activateFirstRow
                  onActiveRowChange={(id) => id && activateIntegration(id)}
                >
                  <DataTable.Table aria-label="Connected integrations">
                    <DataTable.Header>
                      <DataTable.ColumnHeader<WorkspaceIntegration> id="name">Integration</DataTable.ColumnHeader>
                      <DataTable.ColumnHeader<WorkspaceIntegration> id="status">Status</DataTable.ColumnHeader>
                      <DataTable.ColumnHeader<WorkspaceIntegration> id="environment">Environment</DataTable.ColumnHeader>
                      <DataTable.ColumnHeader<WorkspaceIntegration> id="lastSync">Last refresh</DataTable.ColumnHeader>
                    </DataTable.Header>
                    <DataTable.Body<WorkspaceIntegration>>
                      {(integration) => {
                        const status = () => statusOverrides()[integration.id] ?? integration.status;
                        return (
                          <DataTable.Row row={integration} rowId={integration.id} onActivate={() => activateIntegration(integration.id)}>
                            <DataTable.PrimaryCell icon={<NodeIcon name={providerIconKey(integration)} size={16} />} iconTone="integration" title={integration.name} />
                            <DataTable.Cell><StatusChip tone={integrationTone(status())} label={status()} /></DataTable.Cell>
                            <DataTable.Cell>{integration.environment}</DataTable.Cell>
                            <DataTable.Cell class="text-[var(--color-text-muted)]">{integration.lastSync}</DataTable.Cell>
                          </DataTable.Row>
                        );
                      }}
                    </DataTable.Body>
                  </DataTable.Table>
                </DataTable.Root>
              </Show>
            </Show>

            <Show when={tab() === "catalog"}>
              <Show when={catalog().length > 0} fallback={<IntegrationsCatalogFilterEmpty />}>
                <DataTable.Root rows={catalog()} getRowId={(item) => item.id}>
                  <DataTable.Table aria-label="CLI integration catalog">
                    <DataTable.Header>
                      <DataTable.ColumnHeader<WorkspaceCatalogProvider> id="name">Provider</DataTable.ColumnHeader>
                      <DataTable.ColumnHeader<WorkspaceCatalogProvider> id="category">Category</DataTable.ColumnHeader>
                      <DataTable.ColumnHeader<WorkspaceCatalogProvider> id="binary">CLI</DataTable.ColumnHeader>
                      <DataTable.ColumnHeader<WorkspaceCatalogProvider> id="state">State</DataTable.ColumnHeader>
                    </DataTable.Header>
                    <DataTable.Body<WorkspaceCatalogProvider>>
                      {(provider) => (
                        <DataTable.Row row={provider} rowId={provider.id} onActivate={() => openSetup(provider.id)} aria-label={`Open ${provider.name} CLI flow`}>
                          <DataTable.PrimaryCell icon={<NodeIcon name={providerIconKey(provider)} size={16} />} iconTone="integration" title={provider.name} />
                          <DataTable.Cell>{provider.category}</DataTable.Cell>
                          <DataTable.Cell><code>{provider.authCommands.join(" · ")}</code></DataTable.Cell>
                          <DataTable.Cell><StatusChip tone={cliStateTone(provider.state)} label={cliStateLabel(provider.state)} /></DataTable.Cell>
                        </DataTable.Row>
                      )}
                    </DataTable.Body>
                  </DataTable.Table>
                </DataTable.Root>
              </Show>
            </Show>
          </PageContent>
        </div>
      }>
        <SetupWizard
          projectId={props.projectId}
          providerName={setupProvider()}
          providers={props.providers ?? []}
          onBack={closeSetup}
          onFinish={(provider) => {
            const integrationId = matchConnectedIntegrationId(provider.name, props.integrations ?? []);
            closeSetup();
            props.onConnect?.(provider);
            if (integrationId) {
              setStatusOverrides((current) => ({ ...current, [integrationId]: "connected" }));
              props.onTabChange?.("connected");
              activateIntegration(integrationId);
            }
          }}
        />
      </Show>

      <Show when={!setupOpen() && activeIntegration()}>
        {(integration) => (
          <IntegrationDetail
            integration={integration()}
            providers={props.providers ?? []}
            resources={props.resources ?? []}
            syncRuns={props.syncRuns ?? []}
            status={statusOverrides()[integration().id] ?? integration().status}
            onReconfigure={() => openSetup(integration().providerId)}
            onSync={() => props.onSync?.(integration().id)}
            onOpenResource={props.onOpenResource}
          />
        )}
      </Show>
    </>
  );
}
