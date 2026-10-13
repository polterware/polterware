import { ControlButton, DataTable, Delete } from "@polterware/frontend-design-system";
import type { ProcessRow } from "../api";

export function ProcessTable(props: { rows: ProcessRow[]; onKill: (pid: number) => void }) {
  return (
    <DataTable.Root
      rows={props.rows}
      getRowId={(row) => String(row.pid)}
      defaultSort={{ columnId: "cpu", direction: "descending" }}
      virtualized={false}
      estimateRowHeight={40}
    >
      <DataTable.Table aria-label="Processes">
        <DataTable.Header>
          <DataTable.ColumnHeader<ProcessRow> id="pid" sortBy={(row) => row.pid} width="72px">
            PID
          </DataTable.ColumnHeader>
          <DataTable.ColumnHeader<ProcessRow> id="user" sortBy={(row) => row.user}>
            User
          </DataTable.ColumnHeader>
          <DataTable.ColumnHeader<ProcessRow> id="cpu" sortBy={(row) => row.cpuPercent} width="88px">
            CPU %
          </DataTable.ColumnHeader>
          <DataTable.ColumnHeader<ProcessRow> id="mem" sortBy={(row) => row.memPercent} width="88px">
            MEM %
          </DataTable.ColumnHeader>
          <DataTable.ColumnHeader<ProcessRow> id="command" sortBy={(row) => row.command}>
            Command
          </DataTable.ColumnHeader>
          <DataTable.ColumnHeader<ProcessRow> id="actions" width="120px" align="right" sortable={false}>
            <span class="sr-only">Actions</span>
          </DataTable.ColumnHeader>
        </DataTable.Header>
        <DataTable.Body<ProcessRow>>
          {(row) => (
            <DataTable.Row row={row}>
              <DataTable.Cell>{row.pid}</DataTable.Cell>
              <DataTable.Cell>{row.user}</DataTable.Cell>
              <DataTable.Cell>{row.cpuPercent.toFixed(1)}</DataTable.Cell>
              <DataTable.Cell>{row.memPercent.toFixed(1)}</DataTable.Cell>
              <DataTable.Cell>{row.command}</DataTable.Cell>
              <DataTable.Cell align="right">
                <ControlButton tone="danger" onClick={() => props.onKill(row.pid)}>
                  <Delete size={14} />
                  Kill
                </ControlButton>
              </DataTable.Cell>
            </DataTable.Row>
          )}
        </DataTable.Body>
      </DataTable.Table>
    </DataTable.Root>
  );
}
