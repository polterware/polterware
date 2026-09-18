import { Show, createEffect, createMemo, createSignal, onCleanup } from "solid-js";
import {
  A11yDialog,
  Button,
  DataTable,
  DetailPanel,
  Field,
  FilterPopover,
  Input,
  Laptop,
  Package,
  PageContent,
  PageEmpty,
  PageToolbar,
  Play,
  Select,
  SquareTerminal,
  StatusChip,
  Tabs,
} from "@polterware/frontend-design-system";
import { createDetailPanelSelector, detailPanelStore } from "@polterware/frontend-state";
import type { NodeFixture, ProcessFixture, Resource, ResourceStatus } from "./types";

export type * from "./types";

const TABS = [
  { id: "inventory", label: "Inventory" },
  { id: "nodes", label: "Nodes" },
  { id: "processes", label: "Processes" },
] as const;

function normalizeTab(value?: string): "inventory" | "nodes" | "processes" {
  if (value === "nodes" || value === "processes") return value;
  return "inventory";
}

function healthTone(status: ResourceStatus): "healthy" | "warn" | "danger" | "unknown" {
  if (status === "healthy") return "healthy";
  if (status === "degraded") return "warn";
  if (status === "offline") return "danger";
  return "unknown";
}

function processTone(status: ProcessFixture["status"]): "healthy" | "muted" | "danger" {
  if (status === "running") return "healthy";
  if (status === "failed") return "danger";
  return "muted";
}

export function ResourcesView(props: {
  catalog?: Resource[];
  nodes?: NodeFixture[];
  processes?: ProcessFixture[];
  dataSource?: "fixture" | "empty";
  tab?: string;
  search?: string;
  filters?: string[];
  environmentId?: string;
  dialogOpen?: boolean;
  onSearch?: (search: string) => void;
  onFiltersChange?: (filters: string[]) => void;
  onTabChange?: (tab: string) => void;
  onOpen?: (id: string) => void;
  onOpenNode?: (id: string) => void;
  onOpenProcess?: (id: string) => void;
  onOpenArchitecture?: (id: string) => void;
  onCreate?: () => void;
  onCloseDialog?: () => void;
  onSubmitDialog?: () => void;
  onTerminal?: () => void;
  onRunCommand?: () => void;
}) {
  const tab = () => normalizeTab(props.tab);
  const catalog = () => props.catalog ?? [];
  const nodeRows = () => props.nodes ?? [];
  const processRows = () => props.processes ?? [];
  const [query, setQuery] = createSignal(props.search ?? "");
  const [statuses, setStatuses] = createSignal<string[]>(props.filters ?? []);
  const [name, setName] = createSignal("");
  const [type, setType] = createSignal("Service");
  const [environment, setEnvironment] = createSignal("Development");
  const scopeId = createMemo(() => `resources:${tab()}`);
  const activeTarget = createDetailPanelSelector((state) => state.activeTarget);

  const resources = createMemo(() => catalog().filter((resource) => {
    const needle = query().trim().toLowerCase();
    if (statuses().length > 0 && !statuses().includes(resource.status)) return false;
    return !needle || [resource.name, resource.type, resource.provider, ...resource.tags].some((value) => value.toLowerCase().includes(needle));
  }));
  const nodes = createMemo(() => nodeRows().filter((node) => {
    const needle = query().trim().toLowerCase();
    if (statuses().length > 0 && !statuses().includes(node.status)) return false;
    return !needle || [node.name, node.platform, node.environment].some((value) => value.toLowerCase().includes(needle));
  }));
  const processes = createMemo(() => processRows().filter((process) => {
    const needle = query().trim().toLowerCase();
    if (statuses().length > 0 && !statuses().includes(process.status)) return false;
    return !needle || [process.name, process.nodeName, process.command].some((value) => value.toLowerCase().includes(needle));
  }));

  createEffect(() => {
    const scope = scopeId();
    const targets = tab() === "nodes"
      ? nodes().map((item) => ({ scopeId: scope, entityId: item.id, entityKind: "node" }))
      : tab() === "processes"
        ? processes().map((item) => ({ scopeId: scope, entityId: item.id, entityKind: "process" }))
        : resources().map((item) => ({ scopeId: scope, entityId: item.id, entityKind: "resource" }));
    detailPanelStore.getState().registerCollection({ scopeId: scope, status: "ready", targets });
    onCleanup(() => detailPanelStore.getState().unregisterCollection(scope));
  });

  const activeResource = createMemo(() => activeTarget()?.scopeId === "resources:inventory" ? catalog().find((item) => item.id === activeTarget()?.entityId) : undefined);
  const activeNode = createMemo(() => activeTarget()?.scopeId === "resources:nodes" ? nodeRows().find((item) => item.id === activeTarget()?.entityId) : undefined);
  const activeProcess = createMemo(() => activeTarget()?.scopeId === "resources:processes" ? processRows().find((item) => item.id === activeTarget()?.entityId) : undefined);

  function activate(id: string, kind: "resource" | "node" | "process") {
    detailPanelStore.getState().activateTarget({ scopeId: scopeId(), entityId: id, entityKind: kind });
    if (kind === "resource") props.onOpen?.(id);
    else if (kind === "node") props.onOpenNode?.(id);
    else props.onOpenProcess?.(id);
  }

  return (
    <>
      <div class="flex h-full min-h-0 flex-col">
        <PageToolbar.Root>
          <PageToolbar.Tabs><Tabs items={[...TABS]} value={tab()} onChange={(value) => { setQuery(""); setStatuses([]); props.onTabChange?.(value); }} aria-label="Resources" /></PageToolbar.Tabs>
          <PageToolbar.Actions>
            <FilterPopover ariaLabel="Resource filters" title="Filter resources" search={{ placeholder: `Search ${tab()}…`, value: query(), onInput: (value) => { setQuery(value); props.onSearch?.(value); } }} active={statuses().length > 0} onClear={() => { setStatuses([]); props.onFiltersChange?.([]); }} groups={[{ id: "status", label: "Status", options: (tab() === "processes" ? ["running", "stopped", "failed"] : ["healthy", "degraded", "offline", "unknown"]).map((status) => ({ id: status, label: status, checked: statuses().includes(status), onCheckedChange: (checked: boolean) => { const next = checked ? [...statuses(), status] : statuses().filter((value) => value !== status); setStatuses(next); props.onFiltersChange?.(next); } })) }]} />
            <Button size="sm" variant="primary" onClick={() => props.onCreate?.()}>Add resource</Button>
          </PageToolbar.Actions>
        </PageToolbar.Root>
        <PageContent class="p-0">
          <Show when={tab() === "inventory"}>
            <Show when={resources().length > 0} fallback={<PageEmpty title="No resources match" description="Try another search or status filter." />}>
              <DataTable.Root rows={resources()} getRowId={(item) => item.id} activeRowId={activeTarget()?.scopeId === "resources:inventory" ? activeTarget()?.entityId : null} activateFirstRow onActiveRowChange={(id) => id && activate(id, "resource")}>
                <DataTable.Table aria-label="Resource inventory"><DataTable.Header><DataTable.ColumnHeader<Resource> id="name">Resource</DataTable.ColumnHeader><DataTable.ColumnHeader<Resource> id="type">Type</DataTable.ColumnHeader><DataTable.ColumnHeader<Resource> id="provider">Provider</DataTable.ColumnHeader><DataTable.ColumnHeader<Resource> id="status">Status</DataTable.ColumnHeader></DataTable.Header><DataTable.Body<Resource>>{(item) => <DataTable.Row row={item} rowId={item.id} onActivate={() => activate(item.id, "resource")}><DataTable.PrimaryCell icon={<Package size={16} />} iconTone="data" title={item.name} /><DataTable.Cell>{item.type}</DataTable.Cell><DataTable.Cell>{item.provider}</DataTable.Cell><DataTable.Cell><StatusChip tone={healthTone(item.status)} label={item.status} /></DataTable.Cell></DataTable.Row>}</DataTable.Body></DataTable.Table>
              </DataTable.Root>
            </Show>
          </Show>
          <Show when={tab() === "nodes"}>
            <DataTable.Root rows={nodes()} getRowId={(item) => item.id} activeRowId={activeTarget()?.scopeId === "resources:nodes" ? activeTarget()?.entityId : null} activateFirstRow onActiveRowChange={(id) => id && activate(id, "node")}>
              <DataTable.Table aria-label="Nodes"><DataTable.Header><DataTable.ColumnHeader<NodeFixture> id="name">Node</DataTable.ColumnHeader><DataTable.ColumnHeader<NodeFixture> id="status">Status</DataTable.ColumnHeader><DataTable.ColumnHeader<NodeFixture> id="platform">Platform</DataTable.ColumnHeader><DataTable.ColumnHeader<NodeFixture> id="lastSeen">Last seen</DataTable.ColumnHeader></DataTable.Header><DataTable.Body<NodeFixture>>{(item) => <DataTable.Row row={item} rowId={item.id} onActivate={() => activate(item.id, "node")}><DataTable.PrimaryCell icon={<Laptop size={16} />} iconTone="infrastructure" title={item.name} /><DataTable.Cell><StatusChip tone={healthTone(item.status)} label={item.status} /></DataTable.Cell><DataTable.Cell>{item.platform}</DataTable.Cell><DataTable.Cell>{item.lastSeen}</DataTable.Cell></DataTable.Row>}</DataTable.Body></DataTable.Table>
            </DataTable.Root>
          </Show>
          <Show when={tab() === "processes"}>
            <DataTable.Root rows={processes()} getRowId={(item) => item.id} activeRowId={activeTarget()?.scopeId === "resources:processes" ? activeTarget()?.entityId : null} activateFirstRow onActiveRowChange={(id) => id && activate(id, "process")}>
              <DataTable.Table aria-label="Processes"><DataTable.Header><DataTable.ColumnHeader<ProcessFixture> id="name">Process</DataTable.ColumnHeader><DataTable.ColumnHeader<ProcessFixture> id="status">Status</DataTable.ColumnHeader><DataTable.ColumnHeader<ProcessFixture> id="nodeName">Node</DataTable.ColumnHeader><DataTable.ColumnHeader<ProcessFixture> id="startedAt">Started</DataTable.ColumnHeader></DataTable.Header><DataTable.Body<ProcessFixture>>{(item) => <DataTable.Row row={item} rowId={item.id} onActivate={() => activate(item.id, "process")}><DataTable.PrimaryCell icon={<SquareTerminal size={16} />} iconTone="execution" title={item.name} /><DataTable.Cell><StatusChip tone={processTone(item.status)} label={item.status} /></DataTable.Cell><DataTable.Cell>{item.nodeName}</DataTable.Cell><DataTable.Cell>{item.startedAt}</DataTable.Cell></DataTable.Row>}</DataTable.Body></DataTable.Table>
            </DataTable.Root>
          </Show>
        </PageContent>
      </div>

      <Show when={activeResource()}>{(item) => <DetailPanel.Content><DetailPanel.Header icon={<Package size={16} />} title={item().name}><DetailPanel.Actions><DetailPanel.Action label="Open architecture" onClick={() => props.onOpenArchitecture?.(item().id)}><Package /></DetailPanel.Action></DetailPanel.Actions></DetailPanel.Header><DetailPanel.Body layout="sections"><DetailPanel.Grid><DetailPanel.Section title="Resource"><DetailPanel.Metadata><DetailPanel.MetadataItem label="Status" value={<StatusChip tone={healthTone(item().status)} label={item().status} />} /><DetailPanel.MetadataItem label="Type" value={item().type} /><DetailPanel.MetadataItem label="Provider" value={item().provider} /><DetailPanel.MetadataItem label="Environment" value={item().environment} /><DetailPanel.MetadataItem label="Region" value={item().region} /><DetailPanel.MetadataItem label="Source" value="Local store" /></DetailPanel.Metadata><p class="mb-0 mt-3 text-[12px] text-[var(--color-text-secondary)]">{item().description}</p></DetailPanel.Section></DetailPanel.Grid></DetailPanel.Body></DetailPanel.Content>}</Show>
      <Show when={activeNode()}>{(item) => <DetailPanel.Content><DetailPanel.Header icon={<Laptop size={16} />} title={item().name}><DetailPanel.Actions><DetailPanel.Action label="Open visual terminal" onClick={() => props.onTerminal?.()}><SquareTerminal /></DetailPanel.Action></DetailPanel.Actions></DetailPanel.Header><DetailPanel.Body layout="sections"><DetailPanel.Grid><DetailPanel.Section title="Node"><DetailPanel.Metadata><DetailPanel.MetadataItem label="Status" value={<StatusChip tone={healthTone(item().status)} label={item().status} />} /><DetailPanel.MetadataItem label="Platform" value={item().platform} /><DetailPanel.MetadataItem label="Environment" value={item().environment} /><DetailPanel.MetadataItem label="Address" value={item().address} /><DetailPanel.MetadataItem label="CPU" value={item().cpu} /><DetailPanel.MetadataItem label="Memory" value={item().memory} /></DetailPanel.Metadata></DetailPanel.Section></DetailPanel.Grid></DetailPanel.Body></DetailPanel.Content>}</Show>
      <Show when={activeProcess()}>{(item) => <DetailPanel.Content><DetailPanel.Header icon={<SquareTerminal size={16} />} title={item().name}><DetailPanel.Actions><DetailPanel.Action label="Preview command" onClick={() => props.onRunCommand?.()}><Play /></DetailPanel.Action><DetailPanel.Action label="Open visual terminal" onClick={() => props.onTerminal?.()}><SquareTerminal /></DetailPanel.Action></DetailPanel.Actions></DetailPanel.Header><DetailPanel.Body layout="sections"><DetailPanel.Grid><DetailPanel.Section title="Process"><DetailPanel.Metadata><DetailPanel.MetadataItem label="Status" value={<StatusChip tone={processTone(item().status)} label={item().status} />} /><DetailPanel.MetadataItem label="Node" value={item().nodeName} /><DetailPanel.MetadataItem label="PID" value={item().pid?.toString() ?? "—"} /><DetailPanel.MetadataItem label="Started" value={item().startedAt} /><DetailPanel.MetadataItem label="Command" value={item().command} /><DetailPanel.MetadataItem label="Source" value="Local store" /></DetailPanel.Metadata></DetailPanel.Section></DetailPanel.Grid></DetailPanel.Body></DetailPanel.Content>}</Show>

      <A11yDialog open={Boolean(props.dialogOpen)} title="Add resource" size="xl" onClose={() => props.onCloseDialog?.()} footer={<><Button variant="ghost" size="sm" onClick={() => props.onCloseDialog?.()}>Cancel</Button><Button variant="primary" size="sm" disabled={!name().trim()} onClick={() => props.onSubmitDialog?.()}>Save</Button></>}><div class="grid gap-3"><p class="m-0 text-[12px] text-[var(--color-text-muted)]">This resource is stored in the local SQLite database.</p><Field label="Name"><Input size="sm" value={name()} onInput={(event) => setName(event.currentTarget.value)} /></Field><Field label="Type"><Select size="sm" value={type()} options={[{ value: "Service", label: "Service" }, { value: "Database", label: "Database" }, { value: "Cache", label: "Cache" }]} onChange={setType} /></Field><Field label="Environment"><Select size="sm" value={environment()} options={[{ value: "Development", label: "Development" }, { value: "Staging", label: "Staging" }, { value: "Production", label: "Production" }]} onChange={setEnvironment} /></Field></div></A11yDialog>
    </>
  );
}
