import { Show, createEffect, createMemo, createSignal, onCleanup } from "solid-js";
import {
  A11yDialog,
  ActiveFilterBar,
  Button,
  Cloud,
  DataTable,
  DetailPanel,
  Field,
  FilterPopover,
  Input,
  Layers,
  Package,
  PageContent,
  PageToolbar,
  Plus,
  SearchBar,
  Select,
  StatusChip,
  StatusDot,
  ToolbarIconButton,
  type ActiveFilterPill,
  type FilterPopoverGroup,
  type StatusTone,
} from "@polterware/frontend-design-system";
import { createDetailPanelSelector, detailPanelStore } from "@polterware/frontend-state";
import type { Resource, ResourceStatus } from "./types";
import { ResourcesInventoryEmpty, resourcesFilterEmpty } from "./resources-empty";
import { ResourceInventoryTemplateGrid } from "./resources-template-grid";
import {
  EMPTY_RESOURCE_LIST_FILTERS,
  distinctSorted,
  filterResources,
  resourceFiltersFromRoute,
  resourceFiltersToRoute,
  resourceListFiltersActive,
  toggleFilterValue,
  type ResourceListFilters,
} from "./resource-filters";
export type {
  ResourceInventoryDraft,
  ResourceInventoryTemplate,
} from "./create-templates";
export { RESOURCE_INVENTORY_TEMPLATES } from "./create-templates";

export type * from "./types";
export type { ResourceListFilters } from "./resource-filters";

function healthTone(status: ResourceStatus): StatusTone {
  if (status === "healthy") return "healthy";
  if (status === "degraded") return "warn";
  if (status === "offline") return "danger";
  return "unknown";
}

export function ResourcesView(props: {
  catalog?: Resource[];
  dataSource?: "fixture" | "empty";
  search?: string;
  filters?: Readonly<Record<string, readonly string[]>>;
  environmentId?: string;
  dialogOpen?: boolean;
  onSearch?: (search: string) => void;
  onFiltersChange?: (filters: Record<string, readonly string[]>) => void;
  onOpen?: (id: string) => void;
  onOpenArchitecture?: (id: string) => void;
  onCreate?: () => void;
  onCloseDialog?: () => void;
  onSubmitDialog?: (draft: { name: string; type: string }) => void;
}) {
  const catalog = () => props.catalog ?? [];
  const [filters, setFilters] = createSignal<ResourceListFilters>(
    resourceFiltersFromRoute(props.filters, props.search ?? ""),
  );
  const [name, setName] = createSignal("");
  const [type, setType] = createSignal("Service");
  const scopeId = createMemo(() => "resources:inventory");
  const activeTarget = createDetailPanelSelector((state) => state.activeTarget);

  createEffect(() => {
    setFilters(resourceFiltersFromRoute(props.filters, props.search ?? ""));
  });

  createEffect(() => {
    if (props.dialogOpen) return;
    setName("");
    setType("Service");
  });

  const resources = createMemo(() => filterResources(catalog(), filters()));

  createEffect(() => {
    const scope = scopeId();
    const targets = resources().map((item) => ({
      scopeId: scope,
      entityId: item.id,
      entityKind: "resource",
    }));
    detailPanelStore.getState().registerCollection({ scopeId: scope, status: "ready", targets });
    onCleanup(() => detailPanelStore.getState().unregisterCollection(scope));
  });

  const activeResource = createMemo(() =>
    activeTarget()?.scopeId === "resources:inventory"
      ? catalog().find((item) => item.id === activeTarget()?.entityId)
      : undefined,
  );

  function activate(id: string) {
    detailPanelStore.getState().activateTarget({
      scopeId: scopeId(),
      entityId: id,
      entityKind: "resource",
    });
    props.onOpen?.(id);
  }

  function commitFilters(next: ResourceListFilters) {
    setFilters(next);
    props.onSearch?.(next.query);
    props.onFiltersChange?.(resourceFiltersToRoute(next));
  }

  const filterGroups = createMemo((): FilterPopoverGroup[] => {
    const current = filters();
    const types = distinctSorted(catalog().map((item) => item.type));
    const providers = distinctSorted(catalog().map((item) => item.provider));
    const environments = distinctSorted(catalog().map((item) => item.environment));
    const icon = (node: typeof Package) => {
      const Icon = node;
      return <Icon size={14} aria-hidden />;
    };
    return [
      {
        id: "status",
        label: "Status",
        leading: <StatusDot tone="healthy" showLabel={false} />,
        options: (["healthy", "degraded", "offline", "unknown"] as const).map((status) => ({
          id: status,
          label: status,
          leading: <StatusDot tone={healthTone(status)} showLabel={false} />,
          checked: current.statuses.includes(status),
          onCheckedChange: (checked: boolean) =>
            commitFilters({
              ...current,
              statuses: toggleFilterValue(current.statuses, status, checked),
            }),
        })),
      },
      {
        id: "type",
        label: "Type",
        leading: icon(Package),
        options: types.map((value) => ({
          id: value,
          label: value,
          checked: current.types.includes(value),
          onCheckedChange: (checked: boolean) =>
            commitFilters({
              ...current,
              types: toggleFilterValue(current.types, value, checked),
            }),
        })),
      },
      {
        id: "provider",
        label: "Provider",
        leading: icon(Cloud),
        options: providers.map((value) => ({
          id: value,
          label: value,
          checked: current.providers.includes(value),
          onCheckedChange: (checked: boolean) =>
            commitFilters({
              ...current,
              providers: toggleFilterValue(current.providers, value, checked),
            }),
        })),
      },
      {
        id: "environment",
        label: "Environment",
        leading: icon(Layers),
        options: environments.map((value) => ({
          id: value,
          label: value,
          checked: current.environments.includes(value),
          onCheckedChange: (checked: boolean) =>
            commitFilters({
              ...current,
              environments: toggleFilterValue(current.environments, value, checked),
            }),
        })),
      },
    ].filter((group) => group.id === "status" || group.options.length > 0);
  });

  const activePills = createMemo((): ActiveFilterPill[] => {
    const current = filters();
    const pills: ActiveFilterPill[] = [];
    for (const status of current.statuses) {
      pills.push({
        id: `status:${status}`,
        label: `Status: ${status}`,
        onRemove: () =>
          commitFilters({
            ...current,
            statuses: current.statuses.filter((value) => value !== status),
          }),
      });
    }
    for (const value of current.types) {
      pills.push({
        id: `type:${value}`,
        label: `Type: ${value}`,
        onRemove: () =>
          commitFilters({
            ...current,
            types: current.types.filter((item) => item !== value),
          }),
      });
    }
    for (const value of current.providers) {
      pills.push({
        id: `provider:${value}`,
        label: `Provider: ${value}`,
        onRemove: () =>
          commitFilters({
            ...current,
            providers: current.providers.filter((item) => item !== value),
          }),
      });
    }
    for (const value of current.environments) {
      pills.push({
        id: `environment:${value}`,
        label: `Environment: ${value}`,
        onRemove: () =>
          commitFilters({
            ...current,
            environments: current.environments.filter((item) => item !== value),
          }),
      });
    }
    return pills;
  });

  return (
    <>
      <div class="flex h-full min-h-0 flex-col">
        <PageToolbar.Root>
          <PageToolbar.Leading>
            <span class="sr-only">Resource inventory</span>
          </PageToolbar.Leading>
          <PageToolbar.Actions>
            <SearchBar
              placeholder="Search"
              value={filters().query}
              onInput={(value) => commitFilters({ ...filters(), query: value })}
              aria-label="Search"
            />
            <FilterPopover
              ariaLabel="Resource filters"
              title="Filter resources"
              active={resourceListFiltersActive(filters())}
              onClear={() =>
                commitFilters({
                  ...EMPTY_RESOURCE_LIST_FILTERS,
                  query: filters().query,
                })
              }
              groups={filterGroups()}
            />
            <ToolbarIconButton label="Add resource" onClick={() => props.onCreate?.()}>
              <Plus size={14} />
            </ToolbarIconButton>
          </PageToolbar.Actions>
        </PageToolbar.Root>
        <ActiveFilterBar
          items={activePills()}
          onClear={() =>
            commitFilters({
              ...EMPTY_RESOURCE_LIST_FILTERS,
              query: filters().query,
            })
          }
        />
        <PageContent class="p-0">
          <Show
            when={catalog().length > 0}
            fallback={
              <ResourcesInventoryEmpty
                onCreate={() => props.onCreate?.()}
                onCreateFromTemplate={(template) => props.onSubmitDialog?.(template.draft)}
              />
            }
          >
            <Show when={resources().length > 0} fallback={resourcesFilterEmpty()}>
              <DataTable.Root
                rows={resources()}
                getRowId={(item) => item.id}
                activeRowId={
                  activeTarget()?.scopeId === "resources:inventory"
                    ? activeTarget()?.entityId
                    : null
                }
                activateFirstRow
                onActiveRowChange={(id) => id && activate(id)}
              >
                <DataTable.Table aria-label="Resource inventory">
                  <DataTable.Header>
                    <DataTable.ColumnHeader<Resource> id="name">Resource</DataTable.ColumnHeader>
                    <DataTable.ColumnHeader<Resource> id="type">Type</DataTable.ColumnHeader>
                    <DataTable.ColumnHeader<Resource> id="provider">Provider</DataTable.ColumnHeader>
                    <DataTable.ColumnHeader<Resource> id="status">Status</DataTable.ColumnHeader>
                  </DataTable.Header>
                  <DataTable.Body<Resource>>
                    {(item) => (
                      <DataTable.Row
                        row={item}
                        rowId={item.id}
                        onActivate={() => activate(item.id)}
                      >
                        <DataTable.PrimaryCell
                          icon={<Package size={16} />}
                          iconTone="data"
                          title={item.name}
                        />
                        <DataTable.Cell>{item.type}</DataTable.Cell>
                        <DataTable.Cell>{item.provider}</DataTable.Cell>
                        <DataTable.Cell>
                          <StatusChip tone={healthTone(item.status)} label={item.status} />
                        </DataTable.Cell>
                      </DataTable.Row>
                    )}
                  </DataTable.Body>
                </DataTable.Table>
              </DataTable.Root>
            </Show>
          </Show>
        </PageContent>
      </div>

      <Show when={activeResource()}>
        {(item) => (
          <DetailPanel.Content>
            <DetailPanel.Header icon={<Package size={16} />} title={item().name}>
              <DetailPanel.Actions>
                <DetailPanel.Action
                  label="Open architecture"
                  onClick={() => props.onOpenArchitecture?.(item().id)}
                >
                  <Package />
                </DetailPanel.Action>
              </DetailPanel.Actions>
            </DetailPanel.Header>
            <DetailPanel.Body layout="sections">
              <DetailPanel.Grid>
                <DetailPanel.Section title="Resource">
                  <DetailPanel.Metadata>
                    <DetailPanel.MetadataItem
                      label="Status"
                      value={<StatusChip tone={healthTone(item().status)} label={item().status} />}
                    />
                    <DetailPanel.MetadataItem label="Type" value={item().type} />
                    <DetailPanel.MetadataItem label="Provider" value={item().provider} />
                    <DetailPanel.MetadataItem label="Environment" value={item().environment} />
                    <DetailPanel.MetadataItem label="Region" value={item().region} />
                    <DetailPanel.MetadataItem label="Source" value="Local store" />
                  </DetailPanel.Metadata>
                  <p class="mb-0 mt-3 text-[12px] text-[var(--color-text-secondary)]">
                    {item().description}
                  </p>
                </DetailPanel.Section>
              </DetailPanel.Grid>
            </DetailPanel.Body>
          </DetailPanel.Content>
        )}
      </Show>

      <A11yDialog
        open={Boolean(props.dialogOpen)}
        title="Add resource"
        size="xl"
        onClose={() => props.onCloseDialog?.()}
        footer={(
          <>
            <Button variant="ghost" size="sm" onClick={() => props.onCloseDialog?.()}>
              Cancel
            </Button>
            <Button
              variant="primary"
              size="sm"
              disabled={!name().trim()}
              onClick={() => props.onSubmitDialog?.({ name: name().trim(), type: type() })}
            >
              Save blank
            </Button>
          </>
        )}
      >
        <div class="grid gap-4">
          <ResourceInventoryTemplateGrid
            onSelect={(template) => props.onSubmitDialog?.(template.draft)}
          />
          <p class="m-0 text-[12px] text-[var(--color-text-muted)]">
            This resource is stored in the local SQLite database.
          </p>
          <Field label="Name">
            <Input
              size="sm"
              value={name()}
              onInput={(event) => setName(event.currentTarget.value)}
            />
          </Field>
          <Field label="Type">
            <Select
              size="sm"
              value={type()}
              options={[
                { value: "Service", label: "Service" },
                { value: "Database", label: "Database" },
                { value: "Cache", label: "Cache" },
              ]}
              onChange={setType}
            />
          </Field>
          <p class="m-0 text-[12px] text-[var(--color-text-muted)]">
            Created in the currently selected environment from the header switcher.
          </p>
        </div>
      </A11yDialog>
    </>
  );
}
