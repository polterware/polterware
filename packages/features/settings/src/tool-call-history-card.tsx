import { Show } from "solid-js";
import { Button, Card, DataTable, PageEmpty, StatusChip } from "@polterware/frontend-design-system";
import type { AgentToolCallRow } from "./agents-types";

/** Recent local MCP / desktop tool calls from SQLite. */
export function ToolCallHistoryCard(props: {
  calls: AgentToolCallRow[];
  busy: boolean;
  onRefresh: () => void;
}) {
  return (
    <Card>
      <div class="flex items-start justify-between gap-3">
        <div>
          <p class="m-0 text-[12px] font-medium text-[var(--color-text-primary)]">
            Tool call history
          </p>
          <p class="m-0 mt-1 text-[12px] text-[var(--color-text-muted)]">
            Light local log of experimental MCP and Desktop compile calls. Not a full agent audit.
          </p>
        </div>
        <Button size="sm" variant="ghost" disabled={props.busy} onClick={() => props.onRefresh()}>
          Refresh
        </Button>
      </div>

      <div class="mt-3">
        <DataTable.Root rows={props.calls} getRowId={(call) => call.id}>
          <DataTable.Table aria-label="Agent tool call history">
            <DataTable.Header>
              <DataTable.ColumnHeader id="when" sortBy={(call) => call.createdAt}>
                When
              </DataTable.ColumnHeader>
              <DataTable.ColumnHeader id="tool" sortBy={(call) => call.toolName}>
                Tool
              </DataTable.ColumnHeader>
              <DataTable.ColumnHeader id="source" sortBy={(call) => call.source}>
                Source
              </DataTable.ColumnHeader>
              <DataTable.ColumnHeader id="result" sortBy={(call) => (call.ok ? 1 : 0)}>
                Result
              </DataTable.ColumnHeader>
            </DataTable.Header>
            <DataTable.Body>
              {(call) => (
                <DataTable.Row row={call}>
                  <DataTable.Cell>
                    <span class="font-mono text-[11px]">{formatWhen(call.createdAt)}</span>
                  </DataTable.Cell>
                  <DataTable.Cell>{call.toolName}</DataTable.Cell>
                  <DataTable.Cell>{call.source || "—"}</DataTable.Cell>
                  <DataTable.Cell>
                    <div class="flex flex-col gap-1">
                      <StatusChip
                        tone={call.ok ? "ok" : "danger"}
                        label={call.ok ? "ok" : "failed"}
                      />
                      <span class="text-[11px] text-[var(--color-text-muted)]">
                        {call.resultSummary || "—"}
                      </span>
                    </div>
                  </DataTable.Cell>
                </DataTable.Row>
              )}
            </DataTable.Body>
            <DataTable.Empty>
              <Show when={!props.busy}>
                <PageEmpty
                  title="No tool calls yet"
                  description="Start MCP or run Compile context to record a local entry."
                />
              </Show>
            </DataTable.Empty>
          </DataTable.Table>
        </DataTable.Root>
      </div>
    </Card>
  );
}

function formatWhen(raw: string): string {
  if (!raw) return "—";
  const date = new Date(raw);
  if (Number.isNaN(date.getTime())) return raw;
  return date.toLocaleString();
}
