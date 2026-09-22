import { Show, createEffect, createMemo, createSignal, onCleanup } from "solid-js";
import {
  Book03Icon,
  BookOpen02Icon,
  DataTable,
  FileText,
  FilterPopover,
  PageContent,
  PageToolbar,
  Plus,
  StatusChip,
  Tabs,
  ToolbarIconButton,
} from "@polterware/frontend-design-system";
import {
  type WorkspaceAdr,
  type WorkspaceConstitution,
  type WorkspaceDocument,
  type WorkspacePpmEntity,
  type WorkspaceRunbook,
  type WorkspaceSpecArtifactKind,
  type WorkspaceSpecFeature,
} from "@polterware/feature-shell";
import { createDetailPanelSelector, detailPanelStore } from "@polterware/frontend-state";
import { ConstitutionDetail } from "./constitution-detail";
import { ConstitutionRow } from "./constitution-row";
import { CreateKnowledgeDialog, type CreateKnowledgeKind } from "./create-knowledge-dialog";
import type { KnowledgeCreateTemplate } from "./create-templates";
import { DocumentDetail } from "./document-detail";
import { EntityLinksSection } from "./entity-links-section";
import { RunbookDetail } from "./runbook-detail";
import { SpecFeatureDetail } from "./spec-feature-detail";
import {
  documentStatusTone,
  runbookSeverityTone,
  specStatusTone,
} from "./knowledge-document-helpers";
import {
  KnowledgeAdrsEmpty,
  KnowledgeConstitutionEmpty,
  KnowledgeDocsEmpty,
  KnowledgeRunbooksEmpty,
  KnowledgeSpecsEmpty,
  knowledgeFilterEmpty,
} from "./knowledge-empty";

export { featureKeyFromTitle, uniqueFeatureKey } from "./feature-key";
export type { KnowledgeCreateTemplate, KnowledgeTemplateKind } from "./create-templates";
export {
  KNOWLEDGE_CREATE_TEMPLATES,
  knowledgeTemplateById,
  knowledgeTemplatesForKind,
} from "./create-templates";
export {
  checklistHasOpenItems,
  CHECKLIST_BODY_TEMPLATE,
  CLARIFY_BODY_TEMPLATE,
  CONSTITUTION_BODY_TEMPLATE,
  PLAN_BODY_TEMPLATE,
  SPEC_BODY_TEMPLATE,
  TASKS_BODY_TEMPLATE,
} from "./spec-templates";

const TABS = [
  { id: "docs", label: "Docs" },
  { id: "adrs", label: "ADRs" },
  { id: "runbooks", label: "Runbooks" },
  { id: "specs", label: "Specs" },
] as const;

type KnowledgeTab = (typeof TABS)[number]["id"];

function normalizeTab(raw?: string): KnowledgeTab {
  if (raw === "runbooks" || raw === "specs" || raw === "adrs") return raw;
  return "docs";
}

export function KnowledgeView(props: {
  documents?: WorkspaceDocument[];
  adrs?: WorkspaceAdr[];
  runbooks?: WorkspaceRunbook[];
  specFeatures?: WorkspaceSpecFeature[];
  constitution?: WorkspaceConstitution | null;
  entities?: WorkspacePpmEntity[];
  tab?: string;
  error?: string;
  onTabChange?: (tab: string) => void;
  onOpenDoc?: (id: string) => void;
  onOpenAdr?: (id: string) => void;
  onOpenRunbook?: (id: string) => void;
  onOpenFeature?: (featureKey: string) => void;
  onOpenConstitution?: (id: string) => void;
  onOpenComponent?: (componentId: string) => void;
  onOpenEntity?: (entityId: string, refId: string, kind: string) => void;
  onLinkEntity?: (knowledgeId: string, entityId: string, kind: string) => void;
  onUnlinkEntity?: (linkId: string) => void;
  onOpenTerminal?: () => void;
  onOpenOperation?: (id?: string) => void;
  onSaveBody?: (id: string, body: string) => void;
  onCreate?: (kind: "doc" | "runbook" | "adr", title: string) => void;
  onCreateFeature?: (title: string) => void;
  onCreateFromTemplate?: (template: KnowledgeCreateTemplate) => void;
  onEnsureConstitution?: () => void;
  onArchive?: (id: string) => void;
  onArchiveFeature?: (featureKey: string) => void;
  onArchiveConstitution?: () => void;
  onSetFeatureStatus?: (featureKey: string, status: "draft" | "ready") => void;
  onDuplicate?: (id: string) => void;
  onRun?: (id: string) => void;
}) {
  const tab = () => normalizeTab(props.tab);
  const [query, setQuery] = createSignal("");
  const [filters, setFilters] = createSignal<readonly string[]>([]);
  const [bodyOverrides, setBodyOverrides] = createSignal<Record<string, string>>({});
  const [createOpen, setCreateOpen] = createSignal(false);
  const [artifactByFeature, setArtifactByFeature] = createSignal<Record<string, WorkspaceSpecArtifactKind>>({});
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

  const adrs = createMemo(() => {
    const needle = query().trim().toLowerCase();
    return (props.adrs ?? []).filter((adr) => {
      if (filters().length > 0 && !filters().includes(adr.status)) return false;
      if (!needle) return true;
      return [adr.title, adr.summary, ...adr.tags]
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

  const features = createMemo(() => {
    const needle = query().trim().toLowerCase();
    return (props.specFeatures ?? []).filter((feature) => {
      if (filters().length > 0 && !filters().includes(feature.status)) return false;
      if (!needle) return true;
      return [feature.title, feature.featureKey, feature.summary, feature.owner]
        .some((value) => value.toLowerCase().includes(needle));
    });
  });

  createEffect(() => {
    const scope = scopeId();
    const constitutionTarget = props.constitution
      ? [{ scopeId: scope, entityId: props.constitution.id, entityKind: "constitution" as const }]
      : [];
    const targets = tab() === "runbooks"
      ? runbooks().map((runbook) => ({ scopeId: scope, entityId: runbook.id, entityKind: "runbook" }))
      : tab() === "specs"
        ? [
            ...constitutionTarget,
            ...features().map((feature) => ({
              scopeId: scope,
              entityId: feature.featureKey,
              entityKind: "feature" as const,
            })),
          ]
        : tab() === "adrs"
          ? adrs().map((adr) => ({ scopeId: scope, entityId: adr.id, entityKind: "adr" }))
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

  const adr = createMemo((): WorkspaceAdr | undefined => {
    const target = activeTarget();
    return target?.scopeId === "knowledge:adrs" && target.entityKind === "adr"
      ? (props.adrs ?? []).find((item) => item.id === target.entityId)
      : undefined;
  });

  const runbook = createMemo((): WorkspaceRunbook | undefined => {
    const target = activeTarget();
    return target?.scopeId === "knowledge:runbooks" && target.entityKind === "runbook"
      ? (props.runbooks ?? []).find((item) => item.id === target.entityId)
      : undefined;
  });

  const feature = createMemo((): WorkspaceSpecFeature | undefined => {
    const target = activeTarget();
    return target?.scopeId === "knowledge:specs" && target.entityKind === "feature"
      ? (props.specFeatures ?? []).find((item) => item.featureKey === target.entityId)
      : undefined;
  });

  const constitution = createMemo((): WorkspaceConstitution | undefined => {
    const target = activeTarget();
    const current = props.constitution ?? undefined;
    return target?.scopeId === "knowledge:specs"
      && target.entityKind === "constitution"
      && current
      && current.id === target.entityId
      ? current
      : undefined;
  });

  const activeArtifact = createMemo((): WorkspaceSpecArtifactKind => {
    const current = feature();
    if (!current) return "spec";
    return artifactByFeature()[current.featureKey] ?? "spec";
  });

  const activeArtifactId = createMemo(() => {
    const current = feature();
    if (!current) return "";
    return current[activeArtifact()].id;
  });

  function activate(
    entityId: string,
    entityKind: "document" | "adr" | "runbook" | "feature" | "constitution",
  ) {
    detailPanelStore.getState().activateTarget({ scopeId: scopeId(), entityId, entityKind });
    if (entityKind === "document") props.onOpenDoc?.(entityId);
    else if (entityKind === "adr") props.onOpenAdr?.(entityId);
    else if (entityKind === "runbook") props.onOpenRunbook?.(entityId);
    else if (entityKind === "constitution") props.onOpenConstitution?.(entityId);
    else props.onOpenFeature?.(entityId);
  }

  const createKind = createMemo((): CreateKnowledgeKind => {
    if (tab() === "runbooks") return "runbook";
    if (tab() === "specs") return "feature";
    if (tab() === "adrs") return "adr";
    return "doc";
  });

  const filterOptions = createMemo(() => {
    const options = tab() === "docs"
      ? [{ id: "published", label: "Published" }, { id: "draft", label: "Draft" }]
      : tab() === "adrs"
        ? [{ id: "accepted", label: "Accepted" }, { id: "proposed", label: "Proposed" }, { id: "draft", label: "Draft" }]
      : tab() === "specs"
        ? [{ id: "ready", label: "Ready" }, { id: "draft", label: "Draft" }]
        : [
            { id: "critical", label: "Critical" },
            { id: "high", label: "High" },
            { id: "medium", label: "Medium" },
            { id: "low", label: "Low" },
          ];
    return options.map((option) => ({
      ...option,
      checked: filters().includes(option.id),
      onCheckedChange: (checked: boolean) => setFilters((current) =>
        checked ? [...current, option.id] : current.filter((value) => value !== option.id),
      ),
    }));
  });

  const toolbarCopy = createMemo(() => {
    if (tab() === "runbooks") {
      return {
        ariaLabel: "Runbook filters",
        title: "Runbook severity",
        placeholder: "Search runbooks…",
        groupLabel: "Severity",
        createLabel: "New runbook",
      };
    }
    if (tab() === "adrs") {
      return {
        ariaLabel: "ADR filters",
        title: "ADR status",
        placeholder: "Search ADRs…",
        groupLabel: "Status",
        createLabel: "New ADR",
      };
    }
    if (tab() === "specs") {
      return {
        ariaLabel: "Feature filters",
        title: "Feature status",
        placeholder: "Search features…",
        groupLabel: "Status",
        createLabel: "New feature",
      };
    }
    return {
      ariaLabel: "Document filters",
      title: "Document status",
      placeholder: "Search docs…",
      groupLabel: "Status",
      createLabel: "New document",
    };
  });

  function linksFor(knowledgeId: string, links: WorkspaceDocument["entityLinks"] | WorkspaceAdr["entityLinks"] | WorkspaceRunbook["entityLinks"]) {
    return (
      <EntityLinksSection
        links={links}
        entities={props.entities}
        onOpenEntity={props.onOpenEntity}
        onLinkEntity={
          props.onLinkEntity
            ? (entityId) => {
                const entity = (props.entities ?? []).find((item) => item.id === entityId);
                props.onLinkEntity?.(knowledgeId, entityId, entity?.kind ?? "component");
              }
            : undefined
        }
        onUnlink={props.onUnlinkEntity}
      />
    );
  }

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
              ariaLabel={toolbarCopy().ariaLabel}
              title={toolbarCopy().title}
              search={{
                placeholder: toolbarCopy().placeholder,
                value: query(),
                onInput: setQuery,
              }}
              active={filters().length > 0}
              onClear={() => setFilters([])}
              groups={[{
                id: "status",
                label: toolbarCopy().groupLabel,
                options: filterOptions(),
              }]}
            />
            <Show when={props.onCreate || props.onCreateFeature}>
              <ToolbarIconButton
                label={toolbarCopy().createLabel}
                onClick={() => setCreateOpen(true)}
              >
                <Plus size={14} />
              </ToolbarIconButton>
            </Show>
          </PageToolbar.Actions>
        </PageToolbar.Root>

        <Show when={tab() === "docs"}>
          <PageContent class="p-0">
            <Show
              when={(props.documents ?? []).filter((document) => !document.archived).length > 0}
              fallback={
                <KnowledgeDocsEmpty
                  onCreate={props.onCreate ? () => setCreateOpen(true) : undefined}
                  onCreateFromTemplate={props.onCreateFromTemplate}
                />
              }
            >
              <Show when={docs().length > 0} fallback={knowledgeFilterEmpty("docs")}>
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
            </Show>
          </PageContent>
        </Show>

        <Show when={tab() === "adrs"}>
          <PageContent class="p-0">
            <Show
              when={(props.adrs ?? []).length > 0}
              fallback={
                <KnowledgeAdrsEmpty
                  onCreate={props.onCreate ? () => setCreateOpen(true) : undefined}
                  onCreateFromTemplate={props.onCreateFromTemplate}
                />
              }
            >
              <Show when={adrs().length > 0} fallback={knowledgeFilterEmpty("adrs")}>
                <DataTable.Root
                  rows={adrs()}
                  getRowId={(item) => item.id}
                  activeRowId={activeTarget()?.scopeId === "knowledge:adrs" ? activeTarget()?.entityId : null}
                  activateFirstRow
                  onActiveRowChange={(id) => id && activate(id, "adr")}
                >
                  <DataTable.Table aria-label="ADRs">
                    <DataTable.Header>
                      <DataTable.ColumnHeader<WorkspaceAdr> id="title">ADR</DataTable.ColumnHeader>
                      <DataTable.ColumnHeader<WorkspaceAdr> id="status">Status</DataTable.ColumnHeader>
                      <DataTable.ColumnHeader<WorkspaceAdr> id="owner">Owner</DataTable.ColumnHeader>
                      <DataTable.ColumnHeader<WorkspaceAdr> id="updatedAt">Updated</DataTable.ColumnHeader>
                    </DataTable.Header>
                    <DataTable.Body<WorkspaceAdr>>
                      {(item) => (
                        <DataTable.Row row={item} rowId={item.id} onActivate={() => activate(item.id, "adr")}>
                          <DataTable.PrimaryCell icon={<FileText size={16} />} iconTone="knowledge" title={item.title} />
                          <DataTable.Cell><StatusChip tone="ok" label={item.status} /></DataTable.Cell>
                          <DataTable.Cell>{item.owner}</DataTable.Cell>
                          <DataTable.Cell class="text-[var(--color-text-muted)]">{item.updatedAt}</DataTable.Cell>
                        </DataTable.Row>
                      )}
                    </DataTable.Body>
                  </DataTable.Table>
                </DataTable.Root>
              </Show>
            </Show>
          </PageContent>
        </Show>

        <Show when={tab() === "runbooks"}>
          <PageContent class="p-0">
            <Show
              when={(props.runbooks ?? []).length > 0}
              fallback={
                <KnowledgeRunbooksEmpty
                  onCreate={props.onCreate ? () => setCreateOpen(true) : undefined}
                  onCreateFromTemplate={props.onCreateFromTemplate}
                />
              }
            >
              <Show when={runbooks().length > 0} fallback={knowledgeFilterEmpty("runbooks")}>
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
            </Show>
          </PageContent>
        </Show>

        <Show when={tab() === "specs"}>
          <PageContent class="p-0">
            <Show
              when={props.constitution}
              fallback={
                <KnowledgeConstitutionEmpty
                  onCreate={props.onEnsureConstitution ? () => props.onEnsureConstitution?.() : undefined}
                />
              }
            >
              {(item) => (
                <ConstitutionRow
                  constitution={item()}
                  active={
                    activeTarget()?.scopeId === "knowledge:specs"
                    && activeTarget()?.entityKind === "constitution"
                    && activeTarget()?.entityId === item().id
                  }
                  onActivate={() => activate(item().id, "constitution")}
                />
              )}
            </Show>
            <Show
              when={(props.specFeatures ?? []).length > 0}
              fallback={
                <KnowledgeSpecsEmpty
                  onCreate={props.onCreateFeature ? () => setCreateOpen(true) : undefined}
                  onCreateFromTemplate={props.onCreateFromTemplate}
                />
              }
            >
              <Show when={features().length > 0} fallback={knowledgeFilterEmpty("specs")}>
                <DataTable.Root
                  rows={features()}
                  getRowId={(item) => item.featureKey}
                  activeRowId={
                    activeTarget()?.scopeId === "knowledge:specs"
                    && activeTarget()?.entityKind === "feature"
                      ? activeTarget()?.entityId
                      : null
                  }
                  activateFirstRow={!props.constitution}
                  onActiveRowChange={(id) => id && activate(id, "feature")}
                >
                  <DataTable.Table aria-label="Features">
                    <DataTable.Header>
                      <DataTable.ColumnHeader<WorkspaceSpecFeature> id="title">Feature</DataTable.ColumnHeader>
                      <DataTable.ColumnHeader<WorkspaceSpecFeature> id="featureKey">Key</DataTable.ColumnHeader>
                      <DataTable.ColumnHeader<WorkspaceSpecFeature> id="status">Status</DataTable.ColumnHeader>
                      <DataTable.ColumnHeader<WorkspaceSpecFeature> id="updatedAt">Updated</DataTable.ColumnHeader>
                    </DataTable.Header>
                    <DataTable.Body<WorkspaceSpecFeature>>
                      {(item) => (
                        <DataTable.Row
                          row={item}
                          rowId={item.featureKey}
                          onActivate={() => activate(item.featureKey, "feature")}
                        >
                          <DataTable.PrimaryCell icon={<FileText size={16} />} iconTone="knowledge" title={item.title} />
                          <DataTable.Cell class="font-mono text-[12px]">{item.featureKey}</DataTable.Cell>
                          <DataTable.Cell><StatusChip tone={specStatusTone(item.status)} label={item.status} /></DataTable.Cell>
                          <DataTable.Cell class="text-[var(--color-text-muted)]">{item.updatedAt}</DataTable.Cell>
                        </DataTable.Row>
                      )}
                    </DataTable.Body>
                  </DataTable.Table>
                </DataTable.Root>
              </Show>
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
            links={linksFor(item().id, item().entityLinks)}
          />
        )}
      </Show>
      <Show when={adr()}>
        {(item) => (
          <DocumentDetail
            document={{
              id: item().id,
              title: item().title,
              summary: item().summary,
              bodyMarkdown: item().bodyMarkdown,
              author: item().owner,
              updatedAt: item().updatedAt,
              tags: item().tags,
              componentIds: [],
              componentNames: [],
              environment: "All",
              status: "published",
              owner: item().owner,
              entityLinks: item().entityLinks,
            }}
            bodyMarkdown={bodyOverrides()[item().id] ?? item().bodyMarkdown}
            onBodyChange={(markdown) => {
              setBodyOverrides((current) => ({ ...current, [item().id]: markdown }));
              props.onSaveBody?.(item().id, markdown);
            }}
            onArchive={props.onArchive ? () => props.onArchive?.(item().id) : undefined}
            links={linksFor(item().id, item().entityLinks)}
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
            onRun={props.onRun ? () => props.onRun?.(item().id) : undefined}
            onArchive={props.onArchive ? () => props.onArchive?.(item().id) : undefined}
            onDuplicate={props.onDuplicate ? () => props.onDuplicate?.(item().id) : undefined}
            links={linksFor(item().id, item().entityLinks)}
          />
        )}
      </Show>
      <Show when={feature()}>
        {(item) => (
          <SpecFeatureDetail
            feature={item()}
            artifact={activeArtifact()}
            bodyMarkdown={
              bodyOverrides()[activeArtifactId()]
              ?? item()[activeArtifact()].bodyMarkdown
            }
            checklistBody={
              bodyOverrides()[item().checklist.id] ?? item().checklist.bodyMarkdown
            }
            onArtifactChange={(next) => {
              setArtifactByFeature((current) => ({
                ...current,
                [item().featureKey]: next,
              }));
            }}
            onBodyChange={(markdown) => {
              const artifactId = activeArtifactId();
              if (!artifactId) return;
              setBodyOverrides((current) => ({ ...current, [artifactId]: markdown }));
              props.onSaveBody?.(artifactId, markdown);
            }}
            onArchive={
              props.onArchiveFeature
                ? () => props.onArchiveFeature?.(item().featureKey)
                : undefined
            }
            onSetStatus={
              props.onSetFeatureStatus
                ? (status) => props.onSetFeatureStatus?.(item().featureKey, status)
                : undefined
            }
          />
        )}
      </Show>
      <Show when={constitution()}>
        {(item) => (
          <ConstitutionDetail
            constitution={item()}
            bodyMarkdown={bodyOverrides()[item().id] ?? item().bodyMarkdown}
            onBodyChange={(markdown) => {
              setBodyOverrides((current) => ({ ...current, [item().id]: markdown }));
              props.onSaveBody?.(item().id, markdown);
            }}
            onArchive={
              props.onArchiveConstitution
                ? () => props.onArchiveConstitution?.()
                : undefined
            }
          />
        )}
      </Show>
      <CreateKnowledgeDialog
        open={createOpen()}
        kind={createKind()}
        onClose={() => setCreateOpen(false)}
        onCreateFromTemplate={
          props.onCreateFromTemplate
            ? (template) => {
                props.onCreateFromTemplate?.(template);
                setCreateOpen(false);
              }
            : undefined
        }
        onSubmit={(title) => {
          if (createKind() === "feature") props.onCreateFeature?.(title);
          else if (createKind() === "runbook") props.onCreate?.("runbook", title);
          else if (createKind() === "adr") props.onCreate?.("adr", title);
          else props.onCreate?.("doc", title);
          setCreateOpen(false);
        }}
      />
    </>
  );
}
