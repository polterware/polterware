import { DataTable, FileText } from "@polterware/frontend-design-system";
import type { WorkspaceSource } from "@polterware/feature-shell";

export function SourcesTable(props: {
  rows: WorkspaceSource[];
  activeRowId?: string | null;
  onActivate: (id: string) => void;
}) {
  return (
    <DataTable.Root
      rows={props.rows}
      getRowId={(item) => item.id}
      activeRowId={props.activeRowId ?? null}
      activateFirstRow
      onActiveRowChange={(id) => id && props.onActivate(id)}
    >
      <DataTable.Table aria-label="Sources">
        <DataTable.Header>
          <DataTable.ColumnHeader<WorkspaceSource> id="title">Source</DataTable.ColumnHeader>
          <DataTable.ColumnHeader<WorkspaceSource> id="path">Path</DataTable.ColumnHeader>
          <DataTable.ColumnHeader<WorkspaceSource> id="updatedAt">Updated</DataTable.ColumnHeader>
        </DataTable.Header>
        <DataTable.Body<WorkspaceSource>>
          {(item) => (
            <DataTable.Row row={item} rowId={item.id} onActivate={() => props.onActivate(item.id)}>
              <DataTable.PrimaryCell
                icon={<FileText size={16} />}
                iconTone="knowledge"
                title={item.title}
              />
              <DataTable.Cell class="font-mono text-[12px] text-[var(--color-text-muted)]">
                {item.path || "—"}
              </DataTable.Cell>
              <DataTable.Cell class="text-[var(--color-text-muted)]">{item.updatedAt}</DataTable.Cell>
            </DataTable.Row>
          )}
        </DataTable.Body>
      </DataTable.Table>
    </DataTable.Root>
  );
}
