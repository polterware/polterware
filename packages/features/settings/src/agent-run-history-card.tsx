import { For, Show, createMemo } from "solid-js";
import {
  DataTable,
  FilterChip,
  PageEmpty,
  StatusChip,
} from "@polterware/frontend-design-system";
import type {
  AgentRunDetail,
  ChangeEnvelopeView,
  ProjectChangeRow,
} from "./agents-types";
import {
  driftFindingSeverityTone,
  parseDriftStepPayload,
} from "./drift-step-payload";

export type ChangeActorFilter = "all" | "human" | "agent";

/** Unified Change History feed (B9): human + agent justified mutations. */
export function AgentRunHistoryCard(props: {
  changes: ProjectChangeRow[];
  selectedStepId: string;
  detail: AgentRunDetail | undefined;
  actorFilter: ChangeActorFilter;
  busy: boolean;
  onSelect: (stepId: string, agentRunId: string) => void;
  onActorFilter: (filter: ChangeActorFilter) => void;
}) {
  const envelope = createMemo(() => {
    const steps = props.detail?.steps ?? [];
    const selected = steps.find((step) => step.id === props.selectedStepId)
      ?? steps.find((step) => step.kind === "change");
    return selected ? parseChangeEnvelope(selected.payloadJson) : undefined;
  });

  return (
    <div class="flex min-h-0 flex-1 flex-col">
      <div class="flex flex-wrap items-center gap-2 border-b border-[var(--color-border)] px-3 py-2">
        <FilterChip
          selected={props.actorFilter === "all"}
          onClick={() => props.onActorFilter("all")}
        >
          All
        </FilterChip>
        <FilterChip
          selected={props.actorFilter === "human"}
          onClick={() => props.onActorFilter("human")}
        >
          Human
        </FilterChip>
        <FilterChip
          selected={props.actorFilter === "agent"}
          onClick={() => props.onActorFilter("agent")}
        >
          Agent
        </FilterChip>
      </div>
      <div class="grid min-h-0 flex-1 grid-cols-1 gap-0 lg:grid-cols-[minmax(0,1.1fr)_minmax(0,1fr)]">
        <div class="min-h-0 overflow-auto border-b border-[var(--color-border)] lg:border-b-0 lg:border-r">
          <DataTable.Root rows={props.changes} getRowId={(change) => change.stepId}>
            <DataTable.Table aria-label="Unified change history">
              <DataTable.Header>
                <DataTable.ColumnHeader id="when" sortBy={(change) => change.createdAt}>
                  When
                </DataTable.ColumnHeader>
                <DataTable.ColumnHeader id="actor" sortBy={(change) => change.actorKind}>
                  Actor
                </DataTable.ColumnHeader>
                <DataTable.ColumnHeader id="summary" sortBy={(change) => change.summary}>
                  Summary
                </DataTable.ColumnHeader>
                <DataTable.ColumnHeader id="status" sortBy={(change) => change.runStatus}>
                  Run
                </DataTable.ColumnHeader>
              </DataTable.Header>
              <DataTable.Body>
                {(change) => (
                  <DataTable.Row
                    row={change}
                    rowId={change.stepId}
                    class={
                      props.selectedStepId === change.stepId
                        ? "pw-data-table-row--active"
                        : undefined
                    }
                    onActivate={() => props.onSelect(change.stepId, change.agentRunId)}
                    aria-label={`Open change ${change.summary || change.stepId}`}
                  >
                    <DataTable.Cell>
                      <span class="font-mono text-[11px]">{formatWhen(change.createdAt)}</span>
                    </DataTable.Cell>
                    <DataTable.Cell>
                      <StatusChip
                        tone={change.actorKind === "human" ? "syncing" : "warn"}
                        label={change.actorKind || "agent"}
                      />
                    </DataTable.Cell>
                    <DataTable.Cell>
                      <div class="min-w-0">
                        <div class="truncate">{change.summary || "—"}</div>
                        <Show when={change.agentLabel || change.runTask}>
                          <div class="truncate text-[11px] text-[var(--color-text-muted)]">
                            {change.agentLabel || change.runTask}
                          </div>
                        </Show>
                      </div>
                    </DataTable.Cell>
                    <DataTable.Cell>
                      <StatusChip
                        tone={change.runStatus === "completed" ? "ok" : "warn"}
                        label={change.runStatus || "open"}
                      />
                    </DataTable.Cell>
                  </DataTable.Row>
                )}
              </DataTable.Body>
              <DataTable.Empty>
                <Show when={!props.busy}>
                  <PageEmpty
                    title="No justified changes yet"
                    description="Human Desktop saves and MCP audited writes appear here with summary and rationale."
                  />
                </Show>
              </DataTable.Empty>
            </DataTable.Table>
          </DataTable.Root>
        </div>

        <div class="min-h-0 overflow-auto p-4 text-[12.5px]">
          <Show
            when={props.detail}
            fallback={
              <PageEmpty
                title="Select a change"
                description="Pick a justified mutation to inspect the envelope and audit chain."
              />
            }
          >
            {(detail) => (
              <div class="space-y-3">
                <div>
                  <div class="text-[11px] uppercase tracking-wide text-[var(--color-text-muted)]">
                    Run
                  </div>
                  <div class="mt-1 font-medium">{detail().run.task}</div>
                  <div class="mt-1 flex flex-wrap items-center gap-2 text-[11px] text-[var(--color-text-muted)]">
                    <StatusChip
                      tone={detail().run.actorKind === "human" ? "syncing" : "warn"}
                      label={detail().run.actorKind || "agent"}
                    />
                    <span class="font-mono pw-selectable">{detail().run.id}</span>
                    <StatusChip
                      tone={detail().run.status === "completed" ? "ok" : "warn"}
                      label={detail().run.status}
                    />
                  </div>
                </div>

                <Show when={envelope()}>
                  {(env) => (
                    <div class="rounded-[var(--radius-md)] border border-[var(--color-border)] p-3">
                      <div class="text-[11px] uppercase tracking-wide text-[var(--color-text-muted)]">
                        Change envelope
                      </div>
                      <div class="mt-2">
                        <div class="text-[11px] text-[var(--color-text-muted)]">Summary</div>
                        <div class="mt-0.5 font-medium">{env().summary || "—"}</div>
                      </div>
                      <div class="mt-2">
                        <div class="text-[11px] text-[var(--color-text-muted)]">Rationale</div>
                        <div class="mt-0.5 whitespace-pre-wrap">{env().rationale || "—"}</div>
                      </div>
                      <Show when={env().tool}>
                        <div class="mt-2 text-[11px] text-[var(--color-text-muted)]">
                          Tool: <span class="font-mono pw-selectable text-[var(--color-text-primary)]">{env().tool}</span>
                        </div>
                      </Show>
                      <Show when={env().subjectIds.length > 0}>
                        <div class="mt-2 text-[11px] text-[var(--color-text-muted)]">
                          Subjects:{" "}
                          <span class="font-mono pw-selectable text-[var(--color-text-primary)]">
                            {env().subjectIds.join(", ")}
                          </span>
                        </div>
                      </Show>
                    </div>
                  )}
                </Show>

                <div>
                  <div class="mb-2 text-[11px] uppercase tracking-wide text-[var(--color-text-muted)]">
                    Audit chain
                  </div>
                  <ol class="space-y-2">
                    <li class="rounded-[var(--radius-md)] border border-[var(--color-border)] p-2">
                      <div class="text-[11px] font-medium uppercase text-[var(--color-text-muted)]">
                        task
                      </div>
                      <div class="mt-1">{detail().run.task}</div>
                    </li>
                    <For each={detail().steps}>
                      {(step) => {
                        const drift =
                          step.kind === "drift"
                            ? parseDriftStepPayload(step.payloadJson)
                            : undefined;
                        return (
                        <li
                          class={`rounded-[var(--radius-md)] border p-2 ${
                            step.id === props.selectedStepId
                              ? "border-[var(--color-border)] bg-[var(--color-surface-hover)]"
                              : "border-[var(--color-border)]"
                          }`}
                        >
                          <div class="flex items-center justify-between gap-2">
                            <div class="text-[11px] font-medium uppercase text-[var(--color-text-muted)]">
                              {step.kind}
                            </div>
                            <span class="font-mono text-[11px] text-[var(--color-text-muted)]">
                              {formatWhen(step.createdAt)}
                            </span>
                          </div>
                          <div class="mt-1">{step.summary || "—"}</div>
                          <Show when={drift && drift.findings.length > 0}>
                            <ul class="mt-2 space-y-1.5">
                              <For each={drift!.findings}>
                                {(finding) => (
                                  <li class="rounded-[var(--radius-sm)] border border-[var(--color-border-subtle)] px-2 py-1.5">
                                    <div class="flex flex-wrap items-center gap-1.5">
                                      <StatusChip
                                        tone={driftFindingSeverityTone(finding.severity)}
                                        label={finding.severity || "finding"}
                                      />
                                      <Show when={finding.type}>
                                        <span class="font-mono text-[11px] text-[var(--color-text-muted)]">
                                          {finding.type}
                                        </span>
                                      </Show>
                                    </div>
                                    <div class="mt-1 text-[12px]">
                                      {finding.summary || "—"}
                                    </div>
                                    <Show when={finding.subjectIds.length > 0}>
                                      <div class="mt-1 font-mono text-[11px] text-[var(--color-text-muted)] pw-selectable">
                                        {finding.subjectIds.join(", ")}
                                      </div>
                                    </Show>
                                  </li>
                                )}
                              </For>
                            </ul>
                          </Show>
                        </li>
                        );
                      }}
                    </For>
                  </ol>
                </div>
              </div>
            )}
          </Show>
        </div>
      </div>
    </div>
  );
}

function parseChangeEnvelope(payloadJson: string): ChangeEnvelopeView {
  try {
    const raw = JSON.parse(payloadJson || "{}") as Record<string, unknown>;
    const subjectIds = Array.isArray(raw.subject_ids)
      ? raw.subject_ids.flatMap((value) => typeof value === "string" && value ? [value] : [])
      : [];
    const paths = Array.isArray(raw.paths)
      ? raw.paths.flatMap((value) => typeof value === "string" && value ? [value] : [])
      : [];
    return {
      summary: typeof raw.summary === "string" ? raw.summary : "",
      rationale: typeof raw.rationale === "string" ? raw.rationale : "",
      tool: typeof raw.tool === "string" ? raw.tool : "",
      subjectIds,
      paths,
    };
  } catch {
    return { summary: "", rationale: "", tool: "", subjectIds: [], paths: [] };
  }
}

function formatWhen(value: string): string {
  if (!value) return "—";
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return value;
  return date.toLocaleString();
}
