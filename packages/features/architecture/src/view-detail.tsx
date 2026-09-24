import { Show } from "solid-js";
import {
  DataTable,
  DetailPanel,
  Eye,
  Package,
  PageError,
  PageLoading,
} from "@polterware/frontend-design-system";
import type { ArchViewCard } from "./extra-fixtures";
import {
  emptyDescriptionCopy,
  hasEnvironments,
  hasMembershipList,
  hasTags,
  isDescriptionEmpty,
  membershipEmptyCopy,
  membershipRows,
} from "./view-detail-helpers";

export {
  emptyDescriptionCopy,
  hasEnvironments,
  hasMembershipList,
  hasTags,
  isDescriptionEmpty,
  isViewEnriched,
  membershipEmptyCopy,
  membershipRows,
} from "./view-detail-helpers";

function chips(values: readonly string[] | undefined) {
  return (values ?? []).map((value) => ({ id: value, title: value }));
}

export function ArchitectureViewDetail(props: {
  view: ArchViewCard | null | undefined;
  loading?: boolean;
  error?: string | null;
  notFound?: boolean;
  onOpen?: (viewId: string) => void;
  onEdit?: (view: ArchViewCard) => void;
  onDelete?: (view: ArchViewCard) => void;
  onOpenMember?: (memberId: string) => void;
}) {
  return (
    <DetailPanel.Content>
      <Show when={props.loading}>
        <DetailPanel.Header title="Loading view" />
        <DetailPanel.Body>
          <DetailPanel.Main>
            <PageLoading label="Loading view…" />
          </DetailPanel.Main>
        </DetailPanel.Body>
      </Show>

      <Show when={!props.loading && (props.error || props.notFound || !props.view)}>
        <DetailPanel.Header
          title={props.notFound || !props.view ? "View not found" : "Failed to load"}
        />
        <DetailPanel.Body>
          <DetailPanel.Main>
            <PageError
              title={props.notFound || !props.view ? "View not found" : "Failed to load"}
              message={
                props.error ??
                "This architecture view is unavailable or was removed from the project."
              }
            />
          </DetailPanel.Main>
        </DetailPanel.Body>
      </Show>

      <Show when={!props.loading && props.view}>
        {(v) => {
          const members = () => membershipRows(v());
          const membershipKnown = () => hasMembershipList(v());
          const nodeLabel = () => `${v().nodeCount} ${v().nodeCount === 1 ? "node" : "nodes"}`;

          return (
            <>
              <DetailPanel.Header
                icon={<Eye size={16} />}
                title={v().name}
                subtitle={[nodeLabel(), v().updated ? `Updated ${v().updated}` : ""]
                  .filter(Boolean)
                  .join(" · ")}
              >
                <DetailPanel.Actions>
                  <DetailPanel.PrimaryAction
                    label="Open"
                    icon={<Eye size={14} aria-hidden />}
                    onClick={() => props.onOpen?.(v().id)}
                  />
                  <Show when={props.onEdit || props.onDelete}>
                    <DetailPanel.Overflow>
                      <Show when={props.onEdit}>
                        <DetailPanel.OverflowItem value="edit" onSelect={() => props.onEdit?.(v())}>
                          Edit view
                        </DetailPanel.OverflowItem>
                      </Show>
                      <Show when={props.onDelete}>
                        <DetailPanel.OverflowItem
                          value="delete"
                          danger
                          onSelect={() => props.onDelete?.(v())}
                        >
                          Delete view
                        </DetailPanel.OverflowItem>
                      </Show>
                    </DetailPanel.Overflow>
                  </Show>
                </DetailPanel.Actions>
              </DetailPanel.Header>

              <DetailPanel.Body>
                <DetailPanel.Main>
                  <DetailPanel.Section title="Overview">
                    <Show
                      when={!isDescriptionEmpty(v().description)}
                      fallback={
                        <p class="pw-detail-panel__text" data-empty>
                          {emptyDescriptionCopy()}
                        </p>
                      }
                    >
                      <p class="pw-detail-panel__text">{v().description}</p>
                    </Show>
                    <Show when={v().purpose}>
                      <p class="pw-detail-panel__text">{v().purpose}</p>
                    </Show>
                  </DetailPanel.Section>

                  <DetailPanel.Section title="Membership">
                    <Show
                      when={membershipKnown() && members().length > 0}
                      fallback={
                        <p class="pw-detail-panel__text" data-empty>
                          {membershipEmptyCopy(v())}
                        </p>
                      }
                    >
                      <DataTable.Root rows={members()} getRowId={(member) => member.id}>
                        <DataTable.Table aria-label="View members">
                          <DataTable.Header>
                            <DataTable.ColumnHeader
                              id="name"
                              sortBy={(member) => member.name}
                            >
                              Component
                            </DataTable.ColumnHeader>
                            <DataTable.ColumnHeader
                              id="type"
                              sortBy={(member) => member.type ?? ""}
                            >
                              Type
                            </DataTable.ColumnHeader>
                            <DataTable.ColumnHeader
                              id="layer"
                              sortBy={(member) => member.layer ?? ""}
                            >
                              Layer
                            </DataTable.ColumnHeader>
                          </DataTable.Header>
                          <DataTable.Body>
                            {(member) => (
                              <DataTable.Row
                                row={member}
                                rowId={member.id}
                                onActivate={() => props.onOpenMember?.(member.id)}
                                aria-label={`Open ${member.name}`}
                              >
                                <DataTable.PrimaryCell
                                  icon={<Package size={16} />}
                                  iconTone="architecture"
                                  title={member.name}
                                />
                                <DataTable.Cell>
                                  {member.type ?? "—"}
                                </DataTable.Cell>
                                <DataTable.Cell class="text-[var(--color-text-muted)]">
                                  {member.layer ?? "—"}
                                </DataTable.Cell>
                              </DataTable.Row>
                            )}
                          </DataTable.Body>
                          <DataTable.Empty>No members in this view.</DataTable.Empty>
                        </DataTable.Table>
                      </DataTable.Root>
                    </Show>
                  </DetailPanel.Section>

                  <Show when={v().filterSummary}>
                    <DetailPanel.Section title="Filters">
                      <p class="pw-detail-panel__text">{v().filterSummary}</p>
                    </DetailPanel.Section>
                  </Show>

                  <Show when={v().notes}>
                    <DetailPanel.Section title="Notes">
                      <p class="pw-detail-panel__text">{v().notes}</p>
                    </DetailPanel.Section>
                  </Show>
                </DetailPanel.Main>

                <DetailPanel.Rail>
                  <DetailPanel.Section title="Properties">
                    <DetailPanel.Properties>
                      <DetailPanel.Property label="Nodes" value={String(v().nodeCount)} />
                      <DetailPanel.Property label="Owner" value={v().owner?.trim()} />
                      <DetailPanel.Property label="Updated" value={v().updated} />
                    </DetailPanel.Properties>
                  </DetailPanel.Section>
                  <Show when={hasEnvironments(v())}>
                    <DetailPanel.Section title="Environments">
                      <DetailPanel.LinkedEntities items={chips(v().environments)} />
                    </DetailPanel.Section>
                  </Show>
                  <Show when={hasTags(v())}>
                    <DetailPanel.Section title="Tags">
                      <DetailPanel.LinkedEntities items={chips(v().tags)} />
                    </DetailPanel.Section>
                  </Show>
                </DetailPanel.Rail>
              </DetailPanel.Body>
            </>
          );
        }}
      </Show>
    </DetailPanel.Content>
  );
}
