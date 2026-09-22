import { Show } from "solid-js";
import { DataTable, PageEmpty, StatusChip } from "@polterware/frontend-design-system";
import type { AgentToolCallRow } from "./agents-types";

/** Recent local MCP / desktop tool calls from SQLite. */
export function ToolCallHistoryCard(props: {
  calls: AgentToolCallRow[];
  busy: boolean;
}) {
  return (
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
                <div class="flex min-w-0 items-center gap-2">
                  <StatusChip
                    tone={call.ok ? "ok" : "danger"}
                    label={call.ok ? "ok" : "failed"}
                  />
                  <Show when={call.resultSummary}>
                    <span class="truncate text-[11px] text-[var(--color-text-muted)]" title={call.resultSummary}>
                      {call.resultSummary}
                    </span>
                  </Show>
                </div>
              </DataTable.Cell>
            </DataTable.Row>
          )}
        </DataTable.Body>
        <DataTable.Empty>
          <Show when={!props.busy}>
            <PageEmpty
              title="No tool calls yet"
              description="Start MCP or run Compile to record a local entry."
            />
          </Show>
        </DataTable.Empty>
      </DataTable.Table>
    </DataTable.Root>
  );
}

function formatWhen(raw: string): string {
  if (!raw) return "—";
  const date = new Date(raw);
  if (Number.isNaN(date.getTime())) return raw;
  return date.toLocaleString();
}
