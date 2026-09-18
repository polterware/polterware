import { For, Show } from "solid-js";
import {
  DataTable,
  DetailPanel,
  Delete,
  Eye,
  Package,
  PageError,
  PageLoading,
  Pencil,
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
        <DetailPanel.Body layout="sections">
          <DetailPanel.Grid>
            <DetailPanel.Section>
              <PageLoading label="Loading view…" />
            </DetailPanel.Section>
          </DetailPanel.Grid>
        </DetailPanel.Body>
      </Show>

      <Show when={!props.loading && (props.error || props.notFound || !props.view)}>
        <DetailPanel.Header
          title={props.notFound || !props.view ? "View not found" : "Failed to load"}
        />
        <DetailPanel.Body layout="sections">
          <DetailPanel.Grid>
            <DetailPanel.Section>
              <PageError
                title={props.notFound || !props.view ? "View not found" : "Failed to load"}
                message={
                  props.error ??
                  "This architecture view is unavailable or was removed from the project."
                }
              />
            </DetailPanel.Section>
          </DetailPanel.Grid>
        </DetailPanel.Body>
      </Show>

      <Show when={!props.loading && props.view}>
        {(v) => {
          const members = () => membershipRows(v());
          const membershipKnown = () => hasMembershipList(v());
          const membershipEmpty = () => membershipEmptyCopy(v());

          return (
            <>
              <DetailPanel.Header icon={<Eye size={16} />} title={v().name}>
                <DetailPanel.Actions>
                  <DetailPanel.Action
                    label="Open"
                    tone="primary"
                    onClick={() => props.onOpen?.(v().id)}
                  >
                    <Eye />
                  </DetailPanel.Action>
                  <DetailPanel.Action label="Edit" onClick={() => props.onEdit?.(v())}>
                    <Pencil />
                  </DetailPanel.Action>
                  <Show when={props.onDelete}>
                    <DetailPanel.Action
                      label="Delete"
                      tone="danger"
                      onClick={() => props.onDelete?.(v())}
                    >
                      <Delete />
                    </DetailPanel.Action>
                  </Show>
                </DetailPanel.Actions>
              </DetailPanel.Header>

              <DetailPanel.Body layout="sections">
                <DetailPanel.Grid>
                  <DetailPanel.Metric span={4} label="Nodes" value={String(v().nodeCount)} />
                  <DetailPanel.Metric span={4} label="Updated" value={v().updated} />
                  <DetailPanel.Metric
                    span={4}
                    label="Owner"
                    value={v().owner?.trim() ? v().owner! : "—"}
                  />

                  <DetailPanel.Section title="Overview" span={8}>
                    <DetailPanel.Metadata>
                      <DetailPanel.MetadataItem
                        label="Description"
                        value={
                          isDescriptionEmpty(v().description)
                            ? emptyDescriptionCopy()
                            : v().description
                        }
                      />
                      <Show when={v().purpose}>
                        <DetailPanel.MetadataItem label="Purpose" value={v().purpose!} />
                      </Show>
                    </DetailPanel.Metadata>
                  </DetailPanel.Section>

                  <DetailPanel.Section title="Environments" span={4}>
                    <Show
                      when={hasEnvironments(v())}
                      fallback={
                        <p class="m-0 text-[12.5px] text-[var(--color-text-muted)]">
                          No environment scope.
                        </p>
                      }
                    >
                      <ul class="m-0 space-y-1.5 p-0">
                        <For each={v().environments}>
                          {(env) => (
                            <li class="list-none text-[12.5px] text-[var(--color-text-secondary)]">
                              {env}
                            </li>
                          )}
                        </For>
                      </ul>
                    </Show>
                    <Show when={hasTags(v())}>
                      <div class="mt-3">
                        <p class="m-0 mb-1.5 text-[11px] font-medium uppercase tracking-wide text-[var(--color-text-muted)]">
                          Tags
                        </p>
                        <ul class="m-0 space-y-1 p-0">
                          <For each={v().tags}>
                            {(tag) => (
                              <li class="list-none text-[12.5px] text-[var(--color-text-secondary)]">
                                {tag}
                              </li>
                            )}
                          </For>
                        </ul>
                      </div>
                    </Show>
                  </DetailPanel.Section>

                  <DetailPanel.Section title="Membership" span={12}>
                    <Show
                      when={membershipKnown() && members().length > 0}
                      fallback={
                        <p class="m-0 text-[12.5px] text-[var(--color-text-muted)]">
                          {membershipEmpty()}
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
                    <DetailPanel.Section title="Filters" span={6}>
                      <p class="m-0 text-[12.5px] text-[var(--color-text-secondary)]">
                        {v().filterSummary}
                      </p>
                    </DetailPanel.Section>
                  </Show>

                  <Show when={v().notes}>
                    <DetailPanel.Section title="Notes" span={v().filterSummary ? 6 : 12}>
                      <p class="m-0 text-[12.5px] text-[var(--color-text-secondary)]">
                        {v().notes}
                      </p>
                    </DetailPanel.Section>
                  </Show>
                </DetailPanel.Grid>
              </DetailPanel.Body>
            </>
          );
        }}
      </Show>
    </DetailPanel.Content>
  );
}
