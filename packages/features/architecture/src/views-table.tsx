import { DataTable, Eye, PageContent } from "@polterware/frontend-design-system";
import type { ArchViewCard } from "./extra-fixtures";

function updatedAgeInMinutes(value: string): number {
  const amount = Number.parseFloat(value);
  if (!Number.isFinite(amount)) return Number.MAX_SAFE_INTEGER;
  if (value.includes("d")) return amount * 24 * 60;
  if (value.includes("h")) return amount * 60;
  return amount;
}

export function ViewsTable(props: {
  views: ArchViewCard[];
  dataSource?: string;
  activeRowId?: string | null;
  onActivate?: (id: string) => void;
}) {
  return (
    <div class="flex h-full min-h-0 flex-col">
      <PageContent class="p-0">
        <DataTable.Root
          rows={props.views}
          getRowId={(view) => view.id}
          activeRowId={props.activeRowId}
          activateFirstRow
          onActiveRowChange={(id) => {
            if (id) props.onActivate?.(id);
          }}
        >
          <DataTable.Table aria-label="Saved architecture views">
            <DataTable.Header>
              <DataTable.ColumnHeader<ArchViewCard> id="name" sortBy={(view) => view.name}>
                View
              </DataTable.ColumnHeader>
              <DataTable.ColumnHeader<ArchViewCard>
                id="nodeCount"
                sortBy={(view) => view.nodeCount}
              >
                Nodes
              </DataTable.ColumnHeader>
              <DataTable.ColumnHeader<ArchViewCard>
                id="updated"
                sortBy={(view) => updatedAgeInMinutes(view.updated)}
              >
                Updated
              </DataTable.ColumnHeader>
            </DataTable.Header>
            <DataTable.Body<ArchViewCard>>
              {(view) => (
                <DataTable.Row
                  row={view}
                  rowId={view.id}
                  onActivate={() => props.onActivate?.(view.id)}
                  aria-label={`Open ${view.name}`}
                >
                  <DataTable.PrimaryCell icon={<Eye size={16} />} iconTone="architecture" title={view.name} />
                  <DataTable.Cell>{view.nodeCount}</DataTable.Cell>
                  <DataTable.Cell class="text-[var(--color-text-muted)]">{view.updated}</DataTable.Cell>
                </DataTable.Row>
              )}
            </DataTable.Body>
            <DataTable.Empty>No saved views yet.</DataTable.Empty>
          </DataTable.Table>
        </DataTable.Root>
      </PageContent>
    </div>
  );
}
