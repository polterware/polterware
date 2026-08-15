import { Show } from "solid-js";
import {
  DataTable,
  DetailPanel,
  PageError,
  PageLoading,
  StatusChip,
} from "@polterware/frontend-design-system";
import { Activity, FileText, Link2, Network } from "@polterware/frontend-design-system";
import { NodeIcon } from "./domain/node-icon";
import {
  architectureLifecycleLabel,
  architectureLifecycleTone,
  architectureLinkResourceCaption,
  architectureLinkStatusLabel,
  architectureLinkStatusTone,
  connectionEvidenceLabel,
  connectionEvidenceTone,
} from "./node-detail-helpers";
import type { ArchitectureNodeView } from "./types";

export function ArchitectureNodeDetail(props: {
  node: ArchitectureNodeView | null | undefined;
  loading?: boolean;
  error?: string | null;
  notFound?: boolean;
  onOpenRelated?: (id: string) => void;
  onOpenOnCanvas?: (nodeId: string) => void;
  onLinkResource?: (nodeId: string) => void;
}) {
  return (
    <DetailPanel.Content>
      <Show when={props.loading}>
        <DetailPanel.Header title="Loading node" />
        <DetailPanel.Body layout="sections">
          <DetailPanel.Grid>
            <DetailPanel.Section>
              <PageLoading label="Loading node…" />
            </DetailPanel.Section>
          </DetailPanel.Grid>
        </DetailPanel.Body>
      </Show>
      <Show when={!props.loading && (props.error || props.notFound || !props.node)}>
        <DetailPanel.Header
          title={props.notFound || !props.node ? "Node not found" : "Failed to load"}
        />
        <DetailPanel.Body layout="sections">
          <DetailPanel.Grid>
            <DetailPanel.Section>
              <PageError
                title={props.notFound || !props.node ? "Node not found" : "Failed to load"}
                message={props.error ?? "This architecture node is unavailable."}
              />
            </DetailPanel.Section>
          </DetailPanel.Grid>
        </DetailPanel.Body>
      </Show>
      <Show when={!props.loading && props.node}>
        {(n) => {
          const lifecycleLabel = () => architectureLifecycleLabel(n().lifecycle);
          const linkLabel = () => architectureLinkStatusLabel(n().linkStatus);
          const linkCaption = () =>
            architectureLinkResourceCaption(n().linkStatus, n().resourceName);

          return (
            <>
              <DetailPanel.Header
                icon={
                  <NodeIcon
                    type={n().kind}
                    name={n().kind}
                    size={16}
                  />
                }
                title={n().label}
              >
                <Show when={props.onOpenOnCanvas || (n().canLinkResource && props.onLinkResource)}>
                  <DetailPanel.Actions>
                    <Show when={props.onOpenOnCanvas}>
                      <DetailPanel.Action
                        label="Open on canvas"
                        onClick={() => props.onOpenOnCanvas?.(n().id)}
                      >
                        <Network size={14} />
                      </DetailPanel.Action>
                    </Show>
                    <Show when={n().canLinkResource && props.onLinkResource}>
                      <DetailPanel.Action
                        label="Link resource"
                        onClick={() => props.onLinkResource?.(n().id)}
                      >
                        <Link2 size={14} />
                      </DetailPanel.Action>
                    </Show>
                  </DetailPanel.Actions>
                </Show>
              </DetailPanel.Header>
              <DetailPanel.Body layout="sections">
                <DetailPanel.Grid>
                  <DetailPanel.Metric
                    span={3}
                    label="Lifecycle"
                    value={
                      <StatusChip
                        tone={architectureLifecycleTone(n().lifecycle)}
                        label={lifecycleLabel()}
                      />
                    }
                  />
                  <DetailPanel.Metric
                    span={3}
                    label="Link"
                    value={
                      <StatusChip
                        tone={architectureLinkStatusTone(n().linkStatus)}
                        label={linkLabel()}
                      />
                    }
                    description={linkCaption()}
                  />
                  <DetailPanel.Metric span={3} label="Kind" value={n().kind || "—"} />
                  <DetailPanel.Metric
                    span={3}
                    label="Environment"
                    value={n().environment ?? "—"}
                  />

                  <DetailPanel.Section title="Overview" span={4}>
                    <DetailPanel.Metadata>
                      <Show when={linkCaption()}>
                        <DetailPanel.MetadataItem
                          label="Resource"
                          value={linkCaption()!}
                        />
                      </Show>
                      <DetailPanel.MetadataItem label="Role" value={n().role ?? "—"} />
                      <DetailPanel.MetadataItem label="Owner" value={n().owner ?? "—"} />
                    </DetailPanel.Metadata>
                  </DetailPanel.Section>

                  <DetailPanel.Section title="Connected architecture" span={8}>
                    <DataTable.Root
                      rows={n().connected}
                      getRowId={(connection) => connection.nodeId}
                    >
                      <DataTable.Table aria-label="Connected architecture">
                        <DataTable.Header>
                          <DataTable.ColumnHeader
                            id="label"
                            sortBy={(connection) => connection.label}
                          >
                            Node
                          </DataTable.ColumnHeader>
                          <DataTable.ColumnHeader
                            id="relation"
                            sortBy={(connection) => connection.relation}
                          >
                            Relation
                          </DataTable.ColumnHeader>
                          <DataTable.ColumnHeader
                            id="evidence"
                            sortBy={(connection) => connection.evidence}
                          >
                            Evidence
                          </DataTable.ColumnHeader>
                        </DataTable.Header>
                        <DataTable.Body>
                          {(connection) => (
                            <DataTable.Row
                              row={connection}
                              rowId={connection.nodeId}
                              onActivate={() => props.onOpenRelated?.(connection.nodeId)}
                            >
                              <DataTable.PrimaryCell
                                icon={<Network size={16} />}
                                iconTone="architecture"
                                title={connection.label}
                              />
                              <DataTable.Cell>{connection.relation}</DataTable.Cell>
                              <DataTable.Cell>
                                <StatusChip
                                  tone={connectionEvidenceTone(connection.evidence)}
                                  label={connectionEvidenceLabel(connection.evidence)}
                                />
                              </DataTable.Cell>
                            </DataTable.Row>
                          )}
                        </DataTable.Body>
                        <DataTable.Empty>No connections listed.</DataTable.Empty>
                      </DataTable.Table>
                    </DataTable.Root>
                  </DetailPanel.Section>

                  <Show when={n().notes}>
                    <DetailPanel.Section title="Operational notes" span={6}>
                      <p class="m-0 text-[12px] text-[var(--color-text-secondary)]">
                        {n().notes}
                      </p>
                    </DetailPanel.Section>
                  </Show>
                  <Show when={n().docs.length > 0}>
                    <DetailPanel.Section title="Linked docs" span={6}>
                    <DataTable.Root rows={n().docs} getRowId={(doc) => doc.id}>
                      <DataTable.Table aria-label="Linked documents">
                        <DataTable.Header>
                          <DataTable.ColumnHeader id="title" sortBy={(doc) => doc.title}>
                            Document
                          </DataTable.ColumnHeader>
                        </DataTable.Header>
                        <DataTable.Body>
                          {(doc) => (
                            <DataTable.Row row={doc}>
                              <DataTable.PrimaryCell
                                icon={<FileText size={16} />}
                                iconTone="knowledge"
                                title={doc.title}
                              />
                            </DataTable.Row>
                          )}
                        </DataTable.Body>
                        <DataTable.Empty>No docs linked.</DataTable.Empty>
                      </DataTable.Table>
                    </DataTable.Root>
                    </DetailPanel.Section>
                  </Show>

                  <Show when={n().activity.length > 0}>
                    <DetailPanel.Section title="Activity">
                    <DataTable.Root rows={n().activity} getRowId={(event) => event.id}>
                      <DataTable.Table aria-label="Architecture node activity">
                        <DataTable.Header>
                          <DataTable.ColumnHeader
                            id="title"
                            sortBy={(event) => event.title}
                          >
                            Event
                          </DataTable.ColumnHeader>
                          <DataTable.ColumnHeader id="at" sortBy={(event) => event.at}>
                            When
                          </DataTable.ColumnHeader>
                        </DataTable.Header>
                        <DataTable.Body>
                          {(event) => (
                            <DataTable.Row row={event}>
                              <DataTable.PrimaryCell
                                icon={<Activity size={16} />}
                                iconTone="observability"
                                title={event.title}
                              />
                              <DataTable.Cell>{event.at}</DataTable.Cell>
                            </DataTable.Row>
                          )}
                        </DataTable.Body>
                        <DataTable.Empty>No activity.</DataTable.Empty>
                      </DataTable.Table>
                    </DataTable.Root>
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
