import { DataTable, PageContent, PageLoading, StatusChip } from "@polterware/frontend-design-system";
import { NodeIcon } from "./domain/node-icon";
import {
  linkStatusTone,
  nodeLifecycleTone,
} from "./domain/node-lifecycle";
import type { ArchitectureComponentRow } from "./components-list";

export function ComponentsTable(props: {
  rows: ArchitectureComponentRow[];
  loading?: boolean;
  activeRowId?: string | null;
  onActivate?: (id: string) => void;
}) {
  return (
    <div class="flex h-full min-h-0 flex-col">
      <PageContent class="p-0">
        {props.loading ? (
          <PageLoading label="Loading components…" />
        ) : (
          <DataTable.Root
            rows={props.rows}
            getRowId={(row) => row.id}
            activeRowId={props.activeRowId}
            activateFirstRow
            onActiveRowChange={(id) => {
              if (id) props.onActivate?.(id);
            }}
          >
            <DataTable.Table aria-label="Architecture components">
              <DataTable.Header>
                <DataTable.ColumnHeader<ArchitectureComponentRow>
                  id="name"
                  sortBy={(row) => row.name}
                >
                  Name
                </DataTable.ColumnHeader>
                <DataTable.ColumnHeader<ArchitectureComponentRow>
                  id="type"
                  sortBy={(row) => row.typeLabel}
                >
                  Type
                </DataTable.ColumnHeader>
                <DataTable.ColumnHeader<ArchitectureComponentRow>
                  id="lifecycle"
                  sortBy={(row) => row.lifecycleLabel}
                >
                  Lifecycle
                </DataTable.ColumnHeader>
                <DataTable.ColumnHeader<ArchitectureComponentRow>
                  id="link"
                  sortBy={(row) => row.linkLabel}
                >
                  Link
                </DataTable.ColumnHeader>
              </DataTable.Header>
              <DataTable.Body<ArchitectureComponentRow>>
                {(row) => (
                  <DataTable.Row
                    row={row}
                    rowId={row.id}
                    onActivate={() => props.onActivate?.(row.id)}
                    aria-label={`Open ${row.name}`}
                  >
                    <DataTable.PrimaryCell
                      icon={
                        <NodeIcon
                          name={row.iconName}
                          type={row.type}
                          subtype={row.subtype}
                          size={16}
                        />
                      }
                      iconTone="architecture"
                      title={row.name}
                    />
                    <DataTable.Cell class="text-[var(--color-text-secondary)]">
                      {row.typeLabel}
                    </DataTable.Cell>
                    <DataTable.Cell>
                      <StatusChip
                        tone={nodeLifecycleTone(row.lifecycle)}
                        label={row.lifecycleLabel}
                      />
                    </DataTable.Cell>
                    <DataTable.Cell>
                      <StatusChip
                        tone={linkStatusTone(row.linkStatus)}
                        label={row.linkLabel}
                      />
                    </DataTable.Cell>
                  </DataTable.Row>
                )}
              </DataTable.Body>
              <DataTable.Empty>No architecture components yet.</DataTable.Empty>
            </DataTable.Table>
            <DataTable.Footer>
              <span>
                {props.rows.length} component{props.rows.length === 1 ? "" : "s"}
              </span>
            </DataTable.Footer>
          </DataTable.Root>
        )}
      </PageContent>
    </div>
  );
}
