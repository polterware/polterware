import { For, Show } from "solid-js";
import { DataTable, PageEmpty, StatusChip } from "@polterware/frontend-design-system";
import type { AgentRunDetail, AgentRunRow } from "./agents-types";

/** Minimal AgentRun list + chain detail for Agents → History. */
export function AgentRunHistoryCard(props: {
  runs: AgentRunRow[];
  selectedId: string;
  detail: AgentRunDetail | undefined;
  busy: boolean;
  onSelect: (id: string) => void;
}) {
  return (
    <div class="grid min-h-0 flex-1 grid-cols-1 gap-0 lg:grid-cols-[minmax(0,1.1fr)_minmax(0,1fr)]">
      <div class="min-h-0 overflow-auto border-b border-[var(--color-border)] lg:border-b-0 lg:border-r">
        <DataTable.Root rows={props.runs} getRowId={(run) => run.id}>
          <DataTable.Table aria-label="Agent run history">
            <DataTable.Header>
              <DataTable.ColumnHeader id="when" sortBy={(run) => run.createdAt}>
                When
              </DataTable.ColumnHeader>
              <DataTable.ColumnHeader id="task" sortBy={(run) => run.task}>
                Task
              </DataTable.ColumnHeader>
              <DataTable.ColumnHeader id="status" sortBy={(run) => run.status}>
                Status
              </DataTable.ColumnHeader>
            </DataTable.Header>
            <DataTable.Body>
              {(run) => (
                <DataTable.Row
                  row={run}
                  rowId={run.id}
                  class={
                    props.selectedId === run.id
                      ? "pw-data-table-row--active"
                      : undefined
                  }
                  onActivate={() => props.onSelect(run.id)}
                  aria-label={`Open agent run ${run.task || run.id}`}
                >
                  <DataTable.Cell>
                    <span class="font-mono text-[11px]">{formatWhen(run.createdAt)}</span>
                  </DataTable.Cell>
                  <DataTable.Cell>
                    <div class="min-w-0">
                      <div class="truncate">{run.task || "—"}</div>
                      <Show when={run.agentLabel}>
                        <div class="truncate text-[11px] text-[var(--color-text-muted)]">
                          {run.agentLabel}
                        </div>
                      </Show>
                    </div>
                  </DataTable.Cell>
                  <DataTable.Cell>
                    <StatusChip
                      tone={run.status === "completed" ? "ok" : "warn"}
                      label={run.status || "open"}
                    />
                  </DataTable.Cell>
                </DataTable.Row>
              )}
            </DataTable.Body>
            <DataTable.Empty>
              <Show when={!props.busy}>
                <PageEmpty
                  title="No agent runs yet"
                  description="MCP agents start an AgentRun with agent_run.start, then attribute context, change, evidence, and drift."
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
              title="Select a run"
              description="Pick an AgentRun to reconstruct context → task → change → evidence → drift."
            />
          }
        >
          {(detail) => (
            <div class="space-y-3">
              <div>
                <div class="text-[11px] uppercase tracking-wide text-[var(--color-text-muted)]">
                  Task
                </div>
                <div class="mt-1 font-medium">{detail().run.task}</div>
                <div class="mt-1 flex flex-wrap items-center gap-2 text-[11px] text-[var(--color-text-muted)]">
                  <span class="font-mono">{detail().run.id}</span>
                  <StatusChip
                    tone={detail().run.status === "completed" ? "ok" : "warn"}
                    label={detail().run.status}
                  />
                </div>
              </div>
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
                    {(step) => (
                      <li class="rounded-[var(--radius-md)] border border-[var(--color-border)] p-2">
                        <div class="flex items-center justify-between gap-2">
                          <div class="text-[11px] font-medium uppercase text-[var(--color-text-muted)]">
                            {step.kind}
                          </div>
                          <span class="font-mono text-[11px] text-[var(--color-text-muted)]">
                            {formatWhen(step.createdAt)}
                          </span>
                        </div>
                        <div class="mt-1">{step.summary || "—"}</div>
                      </li>
                    )}
                  </For>
                </ol>
              </div>
            </div>
          )}
        </Show>
      </div>
    </div>
  );
}

function formatWhen(value: string): string {
  if (!value) return "—";
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return value;
  return date.toLocaleString();
}
