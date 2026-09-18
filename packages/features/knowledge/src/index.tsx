import { Show, createEffect, createMemo, createSignal, onCleanup } from "solid-js";
import {
  Book03Icon,
  BookOpen02Icon,
  Button,
  DataTable,
  FilterPopover,
  PageContent,
  PageEmpty,
  PageToolbar,
  Plus,
  StatusChip,
  Tabs,
} from "@polterware/frontend-design-system";
import {
  type WorkspaceDocument,
  type WorkspaceRunbook,
} from "@polterware/feature-shell";
import { createDetailPanelSelector, detailPanelStore } from "@polterware/frontend-state";
import { CreateKnowledgeDialog } from "./create-knowledge-dialog";
import { DocumentDetail } from "./document-detail";
import { RunbookDetail } from "./runbook-detail";
import { documentStatusTone, runbookSeverityTone } from "./knowledge-document-helpers";

const TABS = [
  { id: "docs", label: "Docs" },
  { id: "runbooks", label: "Runbooks" },
] as const;

function normalizeTab(raw?: string): "docs" | "runbooks" {
  return raw === "runbooks" ? "runbooks" : "docs";
}

export function KnowledgeView(props: {
  documents?: WorkspaceDocument[];
  runbooks?: WorkspaceRunbook[];
  tab?: string;
  error?: string;
  onTabChange?: (tab: string) => void;
  onOpenDoc?: (id: string) => void;
  onOpenRunbook?: (id: string) => void;
  onOpenComponent?: (componentId: string) => void;
  onOpenTerminal?: () => void;
  onOpenOperation?: (id?: string) => void;
  onSaveBody?: (id: string, body: string) => void;
  onCreate?: (kind: "doc" | "runbook", title: string) => void;
  onArchive?: (id: string) => void;
  onDuplicate?: (id: string) => void;
}) {
  const tab = () => normalizeTab(props.tab);
  const [query, setQuery] = createSignal("");
  const [filters, setFilters] = createSignal<readonly string[]>([]);
  const [bodyOverrides, setBodyOverrides] = createSignal<Record<string, string>>({});
  const [createOpen, setCreateOpen] = createSignal(false);
  const scopeId = createMemo(() => `knowledge:${tab()}`);
  const activeTarget = createDetailPanelSelector((state) => state.activeTarget);

  const docs = createMemo(() => {
    const needle = query().trim().toLowerCase();
    return (props.documents ?? []).filter((document) => {
      if (document.archived) return false;
      if (filters().length > 0 && !filters().includes(document.status)) return false;
      if (!needle) return true;
      return [document.title, document.summary, ...document.tags, ...document.componentNames]
        .some((value) => value.toLowerCase().includes(needle));
    });
  });

  const runbooks = createMemo(() => {
    const needle = query().trim().toLowerCase();
    return (props.runbooks ?? []).filter((runbook) => {
      if (filters().length > 0 && !filters().includes(runbook.severity)) return false;
      if (!needle) return true;
      return [runbook.title, runbook.service, runbook.owner]
        .some((value) => value.toLowerCase().includes(needle));
    });
  });

  createEffect(() => {
    const scope = scopeId();
    const targets = tab() === "runbooks"
      ? runbooks().map((runbook) => ({ scopeId: scope, entityId: runbook.id, entityKind: "runbook" }))
      : docs().map((document) => ({ scopeId: scope, entityId: document.id, entityKind: "document" }));
    detailPanelStore.getState().registerCollection({
      scopeId: scope,
      status: props.error ? "error" : "ready",
      targets,
    });
    onCleanup(() => detailPanelStore.getState().unregisterCollection(scope));
  });

  const document = createMemo((): WorkspaceDocument | undefined => {
    const target = activeTarget();
    return target?.scopeId === "knowledge:docs" && target.entityKind === "document"
      ? (props.documents ?? []).find((item) => item.id === target.entityId)
      : undefined;
  });

  const runbook = createMemo((): WorkspaceRunbook | undefined => {
    const target = activeTarget();
    return target?.scopeId === "knowledge:runbooks" && target.entityKind === "runbook"
      ? (props.runbooks ?? []).find((item) => item.id === target.entityId)
      : undefined;
  });

  function activate(entityId: string, entityKind: "document" | "runbook") {
    detailPanelStore.getState().activateTarget({ scopeId: scopeId(), entityId, entityKind });
    if (entityKind === "document") props.onOpenDoc?.(entityId);
    else props.onOpenRunbook?.(entityId);
  }

  const filterOptions = createMemo(() =>
    (tab() === "docs"
      ? [{ id: "published", label: "Published" }, { id: "draft", label: "Draft" }]
      : [
          { id: "critical", label: "Critical" },
          { id: "high", label: "High" },
          { id: "medium", label: "Medium" },
          { id: "low", label: "Low" },
        ]).map((option) => ({
          ...option,
          checked: filters().includes(option.id),
          onCheckedChange: (checked: boolean) => setFilters((current) =>
            checked ? [...current, option.id] : current.filter((value) => value !== option.id),
          ),
        })),
  );

  return (
    <>
      <div class="flex h-full min-h-0 flex-col">
        <PageToolbar.Root>
          <PageToolbar.Tabs>
            <Tabs
              items={[...TABS]}
              value={tab()}
              onChange={(next) => {
                setQuery("");
                setFilters([]);
                setCreateOpen(false);
                props.onTabChange?.(next);
              }}
              aria-label="Knowledge"
            />
          </PageToolbar.Tabs>
          <PageToolbar.Actions>
            <FilterPopover
              ariaLabel={tab() === "docs" ? "Document filters" : "Runbook filters"}
              title={tab() === "docs" ? "Document status" : "Runbook severity"}
              search={{
                placeholder: tab() === "docs" ? "Search docs…" : "Search runbooks…",
                value: query(),
                onInput: setQuery,
              }}
              active={filters().length > 0}
              onClear={() => setFilters([])}
              groups={[{
                id: "status",
                label: tab() === "docs" ? "Status" : "Severity",
                options: filterOptions(),
              }]}
            />
            <Show when={props.onCreate}>
              <Button
                size="icon"
                variant="primary"
                class="!rounded-full"
                aria-label={tab() === "runbooks" ? "New runbook" : "New document"}
                title={tab() === "runbooks" ? "New runbook" : "New document"}
                onClick={() => setCreateOpen(true)}
              >
                <Plus size={14} />
              </Button>
            </Show>
          </PageToolbar.Actions>
        </PageToolbar.Root>

        <Show when={tab() === "docs"}>
          <PageContent class="p-0">
            <Show when={docs().length > 0} fallback={<PageEmpty title="No documents match" description="Try another search or status filter." />}>
              <DataTable.Root
                rows={docs()}
                getRowId={(item) => item.id}
                activeRowId={activeTarget()?.scopeId === "knowledge:docs" ? activeTarget()?.entityId : null}
                activateFirstRow
                onActiveRowChange={(id) => id && activate(id, "document")}
              >
                <DataTable.Table aria-label="Documents">
                  <DataTable.Header>
                    <DataTable.ColumnHeader<WorkspaceDocument> id="title">Document</DataTable.ColumnHeader>
                    <DataTable.ColumnHeader<WorkspaceDocument> id="status">Status</DataTable.ColumnHeader>
                    <DataTable.ColumnHeader<WorkspaceDocument> id="author">Author</DataTable.ColumnHeader>
                    <DataTable.ColumnHeader<WorkspaceDocument> id="updatedAt">Updated</DataTable.ColumnHeader>
                  </DataTable.Header>
                  <DataTable.Body<WorkspaceDocument>>
                    {(item) => (
                      <DataTable.Row row={item} rowId={item.id} onActivate={() => activate(item.id, "document")}>
                        <DataTable.PrimaryCell icon={<Book03Icon size={16} />} iconTone="knowledge" title={item.title} />
                        <DataTable.Cell><StatusChip tone={documentStatusTone(item.status)} label={item.status} /></DataTable.Cell>
                        <DataTable.Cell>{item.author}</DataTable.Cell>
                        <DataTable.Cell class="text-[var(--color-text-muted)]">{item.updatedAt}</DataTable.Cell>
                      </DataTable.Row>
                    )}
                  </DataTable.Body>
                </DataTable.Table>
              </DataTable.Root>
            </Show>
          </PageContent>
        </Show>

        <Show when={tab() === "runbooks"}>
          <PageContent class="p-0">
            <Show when={runbooks().length > 0} fallback={<PageEmpty title="No runbooks match" description="Try another search or severity filter." />}>
              <DataTable.Root
                rows={runbooks()}
                getRowId={(item) => item.id}
                activeRowId={activeTarget()?.scopeId === "knowledge:runbooks" ? activeTarget()?.entityId : null}
                activateFirstRow
                onActiveRowChange={(id) => id && activate(id, "runbook")}
              >
                <DataTable.Table aria-label="Runbooks">
                  <DataTable.Header>
                    <DataTable.ColumnHeader<WorkspaceRunbook> id="title">Runbook</DataTable.ColumnHeader>
                    <DataTable.ColumnHeader<WorkspaceRunbook> id="service">Service</DataTable.ColumnHeader>
                    <DataTable.ColumnHeader<WorkspaceRunbook> id="severity">Severity</DataTable.ColumnHeader>
                    <DataTable.ColumnHeader<WorkspaceRunbook> id="lastValidated">Last validated</DataTable.ColumnHeader>
                  </DataTable.Header>
                  <DataTable.Body<WorkspaceRunbook>>
                    {(item) => (
                      <DataTable.Row row={item} rowId={item.id} onActivate={() => activate(item.id, "runbook")}>
                        <DataTable.PrimaryCell icon={<BookOpen02Icon size={16} />} iconTone="knowledge" title={item.title} />
                        <DataTable.Cell>{item.service}</DataTable.Cell>
                        <DataTable.Cell><StatusChip tone={runbookSeverityTone(item.severity)} label={item.severity} /></DataTable.Cell>
                        <DataTable.Cell class="text-[var(--color-text-muted)]">{item.lastValidated}</DataTable.Cell>
                      </DataTable.Row>
                    )}
                  </DataTable.Body>
                </DataTable.Table>
              </DataTable.Root>
            </Show>
          </PageContent>
        </Show>
      </div>

      <Show when={document()}>
        {(item) => (
          <DocumentDetail
            document={item()}
            bodyMarkdown={bodyOverrides()[item().id] ?? item().bodyMarkdown}
            onBodyChange={(markdown) => {
              setBodyOverrides((current) => ({ ...current, [item().id]: markdown }));
              props.onSaveBody?.(item().id, markdown);
            }}
            onOpenComponent={props.onOpenComponent}
            onArchive={props.onArchive ? () => props.onArchive?.(item().id) : undefined}
          />
        )}
      </Show>
      <Show when={runbook()}>
        {(item) => (
          <RunbookDetail
            runbook={item()}
            bodyMarkdown={bodyOverrides()[item().id] ?? item().bodyMarkdown}
            onBodyChange={(markdown) => {
              setBodyOverrides((current) => ({ ...current, [item().id]: markdown }));
              props.onSaveBody?.(item().id, markdown);
            }}
            onOpenTerminal={props.onOpenTerminal}
            onOpenComponent={props.onOpenComponent}
            onOpenOperation={props.onOpenOperation}
            onArchive={props.onArchive ? () => props.onArchive?.(item().id) : undefined}
            onDuplicate={props.onDuplicate ? () => props.onDuplicate?.(item().id) : undefined}
          />
        )}
      </Show>
      <CreateKnowledgeDialog
        open={createOpen()}
        kind={tab() === "runbooks" ? "runbook" : "doc"}
        onClose={() => setCreateOpen(false)}
        onSubmit={(title) => {
          props.onCreate?.(tab() === "runbooks" ? "runbook" : "doc", title);
          setCreateOpen(false);
        }}
      />
    </>
  );
}
