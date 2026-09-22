import { Show, createEffect, createMemo, createSignal, onCleanup } from "solid-js";
import {
  A11yDialog,
  Button,
  DataTable,
  DetailPanel,
  Field,
  FilterPopover,
  Input,
  PageContent,
  PageToolbar,
  Play,
  Plus,
  Select,
  StatusChip,
  Tabs,
  ToolbarIconButton,
  Workflow,
  CalendarClock,
  SquareTerminal,
} from "@polterware/frontend-design-system";
import { createDetailPanelSelector, detailPanelStore } from "@polterware/frontend-state";
import { CreateScheduleDialog, type CreateScheduleDraft } from "./create-schedule-dialog";
import { CreateWorkflowDialog, type CreateWorkflowDraft } from "./create-workflow-dialog";
import { scheduleDraftFromTemplate } from "./create-templates";
import type { OperationRun, OperationSchedule, OperationWorkflow, RunStatus } from "./types";
import {
  OperationsRunsEmpty,
  OperationsSchedulesEmpty,
  OperationsWorkflowsEmpty,
  operationsFilterEmpty,
} from "./operations-empty";
import {
  OperationsRunTemplateGrid,
  OperationsScheduleTemplateGrid,
  OperationsWorkflowTemplateGrid,
} from "./operations-template-grid";

export type * from "./types";
export type { CreateScheduleDraft } from "./create-schedule-dialog";
export type { CreateWorkflowDraft } from "./create-workflow-dialog";
export { CreateScheduleDialog } from "./create-schedule-dialog";
export { CreateWorkflowDialog } from "./create-workflow-dialog";
export {
  OPERATIONS_RUN_TEMPLATES,
  OPERATIONS_SCHEDULE_TEMPLATES,
  OPERATIONS_WORKFLOW_TEMPLATES,
} from "./create-templates";

const TABS = [
  { id: "runs", label: "Runs" },
  { id: "workflows", label: "Workflows" },
  { id: "schedules", label: "Schedules" },
] as const;

function normalizeTab(value?: string): "runs" | "workflows" | "schedules" {
  if (value === "workflows" || value === "schedules") return value;
  return "runs";
}

function runTone(status: RunStatus): "muted" | "syncing" | "healthy" | "danger" {
  if (status === "success") return "healthy";
  if (status === "failed") return "danger";
  if (status === "running") return "syncing";
  return "muted";
}

function workflowTone(status: OperationWorkflow["status"]): "healthy" | "warn" | "muted" {
  if (status === "ready") return "healthy";
  if (status === "draft") return "warn";
  return "muted";
}

function scheduleTone(status: OperationSchedule["status"]): "healthy" | "warn" | "muted" {
  if (status === "active") return "healthy";
  if (status === "paused") return "warn";
  return "muted";
}

export function OperationsView(props: {
  runs?: OperationRun[];
  workflows?: OperationWorkflow[];
  schedules?: OperationSchedule[];
  tab?: string;
  search?: string;
  filters?: string[];
  runCommandOpen?: boolean;
  onTabChange?: (tab: string) => void;
  onSearch?: (search: string) => void;
  onFiltersChange?: (filters: string[]) => void;
  onOpenRun?: (id: string) => void;
  onOpenWorkflow?: (id: string) => void;
  onOpenSchedule?: (id: string) => void;
  onCreateWorkflow?: (draft: CreateWorkflowDraft) => void;
  onCreateSchedule?: (draft: CreateScheduleDraft) => void;
  onRunWorkflow?: (workflowId: string) => void;
  onRunCommand?: () => void;
  onCloseRunCommand?: () => void;
  onSubmitRunCommand?: (input: { target: string; command: string; cwd: string; environment: string }) => void;
}) {
  const tab = () => normalizeTab(props.tab);
  const [query, setQuery] = createSignal(props.search ?? "");
  const [statusFilters, setStatusFilters] = createSignal<string[]>(props.filters ?? []);
  const [target, setTarget] = createSignal("local");
  const [command, setCommand] = createSignal("");
  const [cwd, setCwd] = createSignal("~/project");
  const [environment, setEnvironment] = createSignal("Development");
  const [createWorkflowOpen, setCreateWorkflowOpen] = createSignal(false);
  const [createScheduleOpen, setCreateScheduleOpen] = createSignal(false);
  const scopeId = createMemo(() => `operations:${tab()}`);
  const activeTarget = createDetailPanelSelector((state) => state.activeTarget);

  const runRows = () => props.runs ?? [];
  const workflowRows = () => props.workflows ?? [];
  const scheduleRows = () => props.schedules ?? [];
  const runs = createMemo(() => runRows().filter((run) => {
    const needle = query().trim().toLowerCase();
    if (statusFilters().length > 0 && !statusFilters().includes(run.status)) return false;
    return !needle || [run.title, run.target, run.environment].some((value) => value.toLowerCase().includes(needle));
  }));
  const workflows = createMemo(() => workflowRows().filter((workflow) => {
    const needle = query().trim().toLowerCase();
    return !needle || [workflow.name, workflow.description].some((value) => value.toLowerCase().includes(needle));
  }));
  const schedules = createMemo(() => scheduleRows().filter((schedule) => {
    const needle = query().trim().toLowerCase();
    return !needle || [schedule.name, schedule.workflowName, schedule.environment].some((value) => value.toLowerCase().includes(needle));
  }));

  createEffect(() => {
    const scope = scopeId();
    const targets = tab() === "workflows"
      ? workflows().map((item) => ({ scopeId: scope, entityId: item.id, entityKind: "workflow" }))
      : tab() === "schedules"
        ? schedules().map((item) => ({ scopeId: scope, entityId: item.id, entityKind: "schedule" }))
        : runs().map((item) => ({ scopeId: scope, entityId: item.id, entityKind: "run" }));
    detailPanelStore.getState().registerCollection({ scopeId: scope, status: "ready", targets });
    onCleanup(() => detailPanelStore.getState().unregisterCollection(scope));
  });

  const activeRun = createMemo(() => activeTarget()?.scopeId === "operations:runs" ? runRows().find((item) => item.id === activeTarget()?.entityId) : undefined);
  const activeWorkflow = createMemo(() => activeTarget()?.scopeId === "operations:workflows" ? workflowRows().find((item) => item.id === activeTarget()?.entityId) : undefined);
  const activeSchedule = createMemo(() => activeTarget()?.scopeId === "operations:schedules" ? scheduleRows().find((item) => item.id === activeTarget()?.entityId) : undefined);

  function activate(id: string, kind: "run" | "workflow" | "schedule") {
    detailPanelStore.getState().activateTarget({ scopeId: scopeId(), entityId: id, entityKind: kind });
    if (kind === "run") props.onOpenRun?.(id);
    else if (kind === "workflow") props.onOpenWorkflow?.(id);
    else props.onOpenSchedule?.(id);
  }

  return (
    <>
      <div class="flex h-full min-h-0 flex-col">
        <PageToolbar.Root>
          <PageToolbar.Tabs>
            <Tabs items={[...TABS]} value={tab()} onChange={(value) => {
              setQuery(""); setStatusFilters([]); props.onTabChange?.(value);
            }} aria-label="Operations" />
          </PageToolbar.Tabs>
          <PageToolbar.Actions>
            <FilterPopover
              ariaLabel="Operations filters"
              title="Filter operations"
              search={{ placeholder: `Search ${tab()}…`, value: query(), onInput: (value) => { setQuery(value); props.onSearch?.(value); } }}
              active={statusFilters().length > 0}
              onClear={() => { setStatusFilters([]); props.onFiltersChange?.([]); }}
              groups={tab() === "runs" ? [{
                id: "status", label: "Status", options: ["queued", "running", "success", "failed", "cancelled"].map((status) => ({
                  id: status, label: status, checked: statusFilters().includes(status), onCheckedChange: (checked: boolean) => {
                    const next = checked ? [...statusFilters(), status] : statusFilters().filter((value) => value !== status);
                    setStatusFilters(next); props.onFiltersChange?.(next);
                  },
                })),
              }] : []}
            />
            <Show when={tab() === "runs"}>
              <ToolbarIconButton label="Preview command" onClick={() => props.onRunCommand?.()}>
                <Play size={14} />
              </ToolbarIconButton>
            </Show>
            <Show when={tab() === "workflows"}>
              <ToolbarIconButton label="Add workflow" onClick={() => setCreateWorkflowOpen(true)}>
                <Plus size={14} />
              </ToolbarIconButton>
            </Show>
            <Show when={tab() === "schedules"}>
              <ToolbarIconButton label="Add schedule" onClick={() => setCreateScheduleOpen(true)}>
                <Plus size={14} />
              </ToolbarIconButton>
            </Show>
          </PageToolbar.Actions>
        </PageToolbar.Root>
        <PageContent class="p-0">
          <Show when={tab() === "runs"}>
            <Show
              when={runRows().length > 0}
              fallback={
                <OperationsRunsEmpty
                  onRunCommand={() => props.onRunCommand?.()}
                  onCreateFromTemplate={
                    props.onSubmitRunCommand
                      ? (template) => props.onSubmitRunCommand?.(template.draft)
                      : undefined
                  }
                />
              }
            >
              <Show when={runs().length > 0} fallback={operationsFilterEmpty("runs")}>
                <DataTable.Root rows={runs()} getRowId={(item) => item.id} activeRowId={activeTarget()?.scopeId === "operations:runs" ? activeTarget()?.entityId : null} activateFirstRow onActiveRowChange={(id) => id && activate(id, "run")}>
                  <DataTable.Table aria-label="Runs"><DataTable.Header>
                    <DataTable.ColumnHeader<OperationRun> id="title">Run</DataTable.ColumnHeader><DataTable.ColumnHeader<OperationRun> id="status">Status</DataTable.ColumnHeader><DataTable.ColumnHeader<OperationRun> id="target">Target</DataTable.ColumnHeader><DataTable.ColumnHeader<OperationRun> id="startedAt">Started</DataTable.ColumnHeader>
                  </DataTable.Header><DataTable.Body<OperationRun>>{(item) => <DataTable.Row row={item} rowId={item.id} onActivate={() => activate(item.id, "run")}><DataTable.PrimaryCell icon={<SquareTerminal size={16} />} iconTone="execution" title={item.title} /><DataTable.Cell><StatusChip tone={runTone(item.status)} label={item.status} /></DataTable.Cell><DataTable.Cell>{item.target}</DataTable.Cell><DataTable.Cell>{item.startedAt}</DataTable.Cell></DataTable.Row>}</DataTable.Body></DataTable.Table>
                </DataTable.Root>
              </Show>
            </Show>
          </Show>
          <Show when={tab() === "workflows"}>
            <Show
              when={workflowRows().length > 0}
              fallback={
                <OperationsWorkflowsEmpty
                  onCreateWorkflow={() => setCreateWorkflowOpen(true)}
                  onCreateFromTemplate={
                    props.onCreateWorkflow
                      ? (template) => props.onCreateWorkflow?.(template.draft)
                      : undefined
                  }
                />
              }
            >
              <Show when={workflows().length > 0} fallback={operationsFilterEmpty("workflows")}>
                <DataTable.Root rows={workflows()} getRowId={(item) => item.id} activeRowId={activeTarget()?.scopeId === "operations:workflows" ? activeTarget()?.entityId : null} activateFirstRow onActiveRowChange={(id) => id && activate(id, "workflow")}>
                  <DataTable.Table aria-label="Workflows"><DataTable.Header><DataTable.ColumnHeader<OperationWorkflow> id="name">Workflow</DataTable.ColumnHeader><DataTable.ColumnHeader<OperationWorkflow> id="status">Status</DataTable.ColumnHeader><DataTable.ColumnHeader<OperationWorkflow> id="trigger">Trigger</DataTable.ColumnHeader><DataTable.ColumnHeader<OperationWorkflow> id="lastRun">Last run</DataTable.ColumnHeader></DataTable.Header><DataTable.Body<OperationWorkflow>>{(item) => <DataTable.Row row={item} rowId={item.id} onActivate={() => activate(item.id, "workflow")}><DataTable.PrimaryCell icon={<Workflow size={16} />} iconTone="execution" title={item.name} /><DataTable.Cell><StatusChip tone={workflowTone(item.status)} label={item.status} /></DataTable.Cell><DataTable.Cell>{item.trigger}</DataTable.Cell><DataTable.Cell>{item.lastRun}</DataTable.Cell></DataTable.Row>}</DataTable.Body></DataTable.Table>
                </DataTable.Root>
              </Show>
            </Show>
          </Show>
          <Show when={tab() === "schedules"}>
            <Show
              when={scheduleRows().length > 0}
              fallback={
                <OperationsSchedulesEmpty
                  hasWorkflows={workflowRows().length > 0}
                  onCreateSchedule={() => setCreateScheduleOpen(true)}
                  onCreateFromTemplate={
                    props.onCreateSchedule && workflowRows().length > 0
                      ? (template) => {
                          const workflowId = workflowRows()[0]?.id;
                          if (!workflowId) return;
                          props.onCreateSchedule?.(scheduleDraftFromTemplate(template, workflowId));
                        }
                      : undefined
                  }
                />
              }
            >
              <Show when={schedules().length > 0} fallback={operationsFilterEmpty("schedules")}>
                <DataTable.Root rows={schedules()} getRowId={(item) => item.id} activeRowId={activeTarget()?.scopeId === "operations:schedules" ? activeTarget()?.entityId : null} activateFirstRow onActiveRowChange={(id) => id && activate(id, "schedule")}>
                  <DataTable.Table aria-label="Schedules"><DataTable.Header><DataTable.ColumnHeader<OperationSchedule> id="name">Schedule</DataTable.ColumnHeader><DataTable.ColumnHeader<OperationSchedule> id="status">Status</DataTable.ColumnHeader><DataTable.ColumnHeader<OperationSchedule> id="expression">Expression</DataTable.ColumnHeader><DataTable.ColumnHeader<OperationSchedule> id="nextRun">Next run</DataTable.ColumnHeader></DataTable.Header><DataTable.Body<OperationSchedule>>{(item) => <DataTable.Row row={item} rowId={item.id} onActivate={() => activate(item.id, "schedule")}><DataTable.PrimaryCell icon={<CalendarClock size={16} />} iconTone="execution" title={item.name} /><DataTable.Cell><StatusChip tone={scheduleTone(item.status)} label={item.status} /></DataTable.Cell><DataTable.Cell><code>{item.expression}</code></DataTable.Cell><DataTable.Cell>{item.nextRun}</DataTable.Cell></DataTable.Row>}</DataTable.Body></DataTable.Table>
                </DataTable.Root>
              </Show>
            </Show>
          </Show>
        </PageContent>
      </div>

      <Show when={activeRun()}>{(item) => <DetailPanel.Content><DetailPanel.Header icon={<SquareTerminal size={16} />} title={item().title} /><DetailPanel.Body layout="sections"><DetailPanel.Grid><DetailPanel.Section title="Run"><DetailPanel.Metadata><DetailPanel.MetadataItem label="Status" value={<StatusChip tone={runTone(item().status)} label={item().status} />} /><DetailPanel.MetadataItem label="Target" value={item().target} /><DetailPanel.MetadataItem label="Environment" value={item().environment} /><DetailPanel.MetadataItem label="Command" value={item().command} /></DetailPanel.Metadata></DetailPanel.Section><DetailPanel.Section title="Steps"><div class="space-y-2">{item().steps.map((step) => <div class="flex justify-between gap-3"><span>{step.name}</span><StatusChip tone={runTone(step.status === "skipped" ? "cancelled" : step.status)} label={step.status} /></div>)}</div></DetailPanel.Section><DetailPanel.Section title="Output"><pre class="m-0 whitespace-pre-wrap font-mono text-[12px]">{item().outputLines.join("\n")}</pre><p class="mb-0 mt-3 text-[12px] text-[var(--color-text-muted)]">{item().result}</p></DetailPanel.Section></DetailPanel.Grid></DetailPanel.Body></DetailPanel.Content>}</Show>
      <Show when={activeWorkflow()}>{(item) => <DetailPanel.Content><DetailPanel.Header icon={<Workflow size={16} />} title={item().name}><Show when={props.onRunWorkflow}><DetailPanel.Actions><DetailPanel.Action label="Run workflow" tone="primary" onClick={() => props.onRunWorkflow?.(item().id)}><Play /></DetailPanel.Action></DetailPanel.Actions></Show></DetailPanel.Header><DetailPanel.Body layout="sections"><DetailPanel.Grid><DetailPanel.Section title="Workflow"><p class="m-0 text-[13px]">{item().description}</p><p class="mb-0 mt-2 text-[12px] text-[var(--color-text-muted)]">Runs locally through polterware-core.</p></DetailPanel.Section><DetailPanel.Section title="Direct steps"><div class="space-y-2">{item().steps.map((step, index) => <div class="flex gap-3"><span class="text-[var(--color-text-muted)]">{index + 1}</span><div><p class="m-0 text-[13px]">{step.label}</p><p class="m-0 text-[11px] text-[var(--color-text-muted)]">{step.type}{step.command ? ` · ${step.command}` : ""}</p></div></div>)}</div></DetailPanel.Section></DetailPanel.Grid></DetailPanel.Body></DetailPanel.Content>}</Show>
      <Show when={activeSchedule()}>{(item) => <DetailPanel.Content><DetailPanel.Header icon={<CalendarClock size={16} />} title={item().name} /><DetailPanel.Body layout="sections"><DetailPanel.Grid><DetailPanel.Section title="Schedule"><DetailPanel.Metadata><DetailPanel.MetadataItem label="Status" value={<StatusChip tone={scheduleTone(item().status)} label={item().status} />} /><DetailPanel.MetadataItem label="Workflow" value={item().workflowName} /><DetailPanel.MetadataItem label="Expression" value={item().expression} /><DetailPanel.MetadataItem label="Timezone" value={item().timezone} /><DetailPanel.MetadataItem label="Next run" value={item().nextRun} /><DetailPanel.MetadataItem label="Environment" value={item().environment} /></DetailPanel.Metadata><p class="mb-0 mt-3 text-[12px] text-[var(--color-text-muted)]">Runs locally through polterware-core when due. Missed times while the core was offline are skipped.</p></DetailPanel.Section></DetailPanel.Grid></DetailPanel.Body></DetailPanel.Content>}</Show>

      <CreateWorkflowDialog
        open={createWorkflowOpen()}
        onClose={() => setCreateWorkflowOpen(false)}
        onCreateFromTemplate={(template) => {
          setCreateWorkflowOpen(false);
          props.onCreateWorkflow?.(template.draft);
        }}
        onSubmit={(draft) => {
          setCreateWorkflowOpen(false);
          props.onCreateWorkflow?.(draft);
        }}
      />

      <CreateScheduleDialog
        open={createScheduleOpen()}
        workflows={workflowRows().map((workflow) => ({ id: workflow.id, name: workflow.name }))}
        onClose={() => setCreateScheduleOpen(false)}
        onCreateFromTemplate={(draft) => {
          setCreateScheduleOpen(false);
          props.onCreateSchedule?.(draft);
        }}
        onSubmit={(draft) => {
          setCreateScheduleOpen(false);
          props.onCreateSchedule?.(draft);
        }}
      />

      <A11yDialog open={Boolean(props.runCommandOpen)} title="Run command" size="xl" onClose={() => props.onCloseRunCommand?.()} footer={<><Button variant="ghost" size="sm" onClick={() => props.onCloseRunCommand?.()}>Cancel</Button><Button variant="primary" size="sm" disabled={!command().trim()} onClick={() => props.onSubmitRunCommand?.({ target: target(), command: command(), cwd: cwd(), environment: environment() })}>Run blank</Button></>}>
        <div class="grid gap-4">
          <OperationsRunTemplateGrid
            onSelect={(template) => props.onSubmitRunCommand?.(template.draft)}
          />
          <p class="m-0 text-[12px] text-[var(--color-text-muted)]">The command runs locally through polterware-core and appends logs to the operation.</p>
          <Field label="Target"><Input size="sm" value={target()} onInput={(event) => setTarget(event.currentTarget.value)} /></Field>
          <Field label="Command"><Input size="sm" class="font-mono" value={command()} onInput={(event) => setCommand(event.currentTarget.value)} placeholder="systemctl status example-api" /></Field>
          <Field label="Working directory"><Input size="sm" value={cwd()} onInput={(event) => setCwd(event.currentTarget.value)} /></Field>
          <Field label="Environment"><Select size="sm" value={environment()} options={[{ value: "Development", label: "Development" }, { value: "Staging", label: "Staging" }, { value: "Production", label: "Production" }]} onChange={setEnvironment} /></Field>
        </div>
      </A11yDialog>
    </>
  );
}
