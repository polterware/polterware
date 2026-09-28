import { Show, createEffect, createMemo, createSignal, onCleanup } from "solid-js";
import {
  ActiveFilterBar,
  BookOpen,
  FileText,
  FileUp,
  FilterMark,
  FilterPopover,
  Folder,
  LibraryBig,
  Link,
  ListChecks,
  PageContent,
  PageLoading,
  PageToolbar,
  Plus,
  ScrollText,
  SearchBar,
  StatusChip,
  StatusDot,
  Tabs,
  ToolbarIconButton,
  type ActiveFilterPill,
  type FilterPopoverGroup,
  type TabItem,
  DataTable,
} from "@polterware/frontend-design-system";
import {
  type WorkspaceAdr,
  type WorkspaceConstitution,
  type WorkspaceDocument,
  type WorkspacePpmEntity,
  type WorkspaceRunbook,
  type WorkspaceSource,
  type WorkspaceSpecArtifactKind,
  type WorkspaceSpecFeature,
} from "@polterware/feature-shell";
import { createDetailPanelSelector, detailPanelStore } from "@polterware/frontend-state";
import { ConstitutionDetail } from "./constitution-detail";
import { KnowledgeConstitutionEmpty } from "./constitution-empty";
import { ConstitutionRow } from "./constitution-row";
import { CreateKnowledgeDialog, type CreateKnowledgeKind } from "./create-knowledge-dialog";
import { CreateSourceDialog } from "./create-source-dialog";
import type { KnowledgeCreateTemplate } from "./create-templates";
import { createDocumentAutosave } from "./document-autosave";
import { DocumentDetail } from "./document-detail";
import { EntityLinks } from "./entity-links";
import { LibraryEmpty, libraryFilterEmpty } from "./library-empty";
import {
  EMPTY_LIBRARY_FILTERS,
  filterLibraryRows,
  libraryEmptyCategory,
  libraryFiltersActive,
  libraryRowsFromWorkspace,
  toggleFilterValue,
  type LibraryCategory,
  type LibraryFilters,
  type LibraryRow,
} from "./library-row";
import {
  EMPTY_SOURCE_LIST_FILTERS,
  EMPTY_SPEC_LIST_FILTERS,
  distinctSourcePathPrefixes,
  filterSources,
  filterSpecFeatures,
  sourceListFiltersActive,
  specListFiltersActive,
  type SourceListFilters,
  type SpecListFilters,
} from "./list-filters";
import { LibraryTable } from "./library-table";
import { RunbookDetail } from "./runbook-detail";
import { SourceDetail } from "./source-detail";
import { SourcesEmpty, sourcesFilterEmpty } from "./sources-empty";
import { SourcesTable } from "./sources-table";
import { SpecFeatureDetail } from "./spec-feature-detail";
import {
  libraryStatusTone,
  runbookSeverityTone,
  specStatusTone,
} from "./knowledge-document-helpers";
import { KnowledgeSpecsEmpty, knowledgeFilterEmpty } from "./knowledge-empty";

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
export { ChangeEnvelopeDialogHost } from "./change-envelope-dialog";
export {
  requestChangeEnvelope,
  type ChangeEnvelope,
  type ChangeEnvelopeRequest,
} from "./change-envelope-request";

const TABS: TabItem[] = [
  {
    id: "specs",
    label: "Specs",
    tooltip: {
      title: "Feature specs",
      description:
        "One Constitution per project, plus features as Spec · Plan · Tasks · Clarify · Checklist. Mark ready is blocked while Checklist still has unchecked - [ ] items.",
      icon: ListChecks,
    },
  },
  {
    id: "library",
    label: "Library",
    tooltip: {
      title: "Knowledge library",
      description:
        "Docs, ADRs, and runbooks in one list. Filter by kind, status, severity, and links.",
      icon: LibraryBig,
    },
  },
  {
    id: "sources",
    label: "Sources",
    tooltip: {
      title: "Project sources",
      description:
        "Register project paths that ground the model — manifests, contracts, and repo files linked to Architecture and Knowledge.",
      icon: FileText,
    },
  },
];

type KnowledgeTab = "specs" | "library" | "sources";

type CollectionTarget = {
  scopeId: string;
  entityId: string;
  entityKind: string;
};

function normalizeTab(raw?: string): KnowledgeTab {
  if (raw === "library" || raw === "sources") return raw;
  if (raw === "docs" || raw === "adrs" || raw === "runbooks") return "library";
  return "specs";
}

function collectionFingerprint(
  scopeId: string,
  status: string,
  targets: readonly CollectionTarget[],
): string {
  return `${scopeId}|${status}|${targets.map((t) => `${t.entityId}:${t.entityKind}`).join(",")}`;
}

export function KnowledgeView(props: {
  documents?: WorkspaceDocument[];
  adrs?: WorkspaceAdr[];
  runbooks?: WorkspaceRunbook[];
  sources?: WorkspaceSource[];
  specFeatures?: WorkspaceSpecFeature[];
  constitution?: WorkspaceConstitution | null;
  entities?: WorkspacePpmEntity[];
  tab?: string;
  loading?: boolean;
  error?: string;
  onTabChange?: (tab: string) => void;
  onOpenDoc?: (id: string) => void;
  onOpenAdr?: (id: string) => void;
  onOpenRunbook?: (id: string) => void;
  onOpenSource?: (id: string) => void;
  onOpenFeature?: (featureKey: string) => void;
  onOpenConstitution?: (id: string) => void;
  onOpenComponent?: (componentId: string) => void;
  onOpenEntity?: (entityId: string, refId: string, kind: string) => void;
  onLinkEntity?: (knowledgeId: string, entityId: string, kind: string) => void;
  onUnlinkEntity?: (linkId: string) => void;
  onSaveBody?: (id: string, body: string) => void;
  onSaveSourcePath?: (id: string, path: string) => void;
  onCreate?: (kind: "doc" | "runbook" | "adr", title: string) => void;
  onCreateSource?: (input: { title: string; path: string }) => void;
  onImportProjectSources?: () => void;
  importBusy?: boolean;
  onCreateFeature?: (title: string) => void;
  onCreateFromTemplate?: (template: KnowledgeCreateTemplate) => void;
  onEnsureConstitution?: () => void;
  onArchive?: (id: string) => void;
  onArchiveFeature?: (featureKey: string) => void;
  onArchiveConstitution?: () => void;
  onSetFeatureStatus?: (featureKey: string, status: "draft" | "ready") => void;
  onDuplicate?: (id: string) => void;
}) {
  const tab = () => normalizeTab(props.tab);
  const [libraryFilters, setLibraryFilters] = createSignal<LibraryFilters>({
    ...EMPTY_LIBRARY_FILTERS,
  });
  const [specFilters, setSpecFilters] = createSignal<SpecListFilters>({
    ...EMPTY_SPEC_LIST_FILTERS,
  });
  const [sourceFilters, setSourceFilters] = createSignal<SourceListFilters>({
    ...EMPTY_SOURCE_LIST_FILTERS,
  });
  const [bodyOverrides, setBodyOverrides] = createSignal<Record<string, string>>({});
  const [pathOverrides, setPathOverrides] = createSignal<Record<string, string>>({});
  const [createOpen, setCreateOpen] = createSignal(false);
  const [sourceCreateOpen, setSourceCreateOpen] = createSignal(false);
  const [artifactByFeature, setArtifactByFeature] = createSignal<Record<string, WorkspaceSpecArtifactKind>>({});
  const scopeId = createMemo(() => `knowledge:${tab()}`);
  const activeTarget = createDetailPanelSelector((state) => state.activeTarget);

  const bodyAutosave = createDocumentAutosave({
    save: (id, body) => props.onSaveBody?.(id, body),
  });
  const pathAutosave = createDocumentAutosave({
    save: (id, path) => props.onSaveSourcePath?.(id, path),
  });
  onCleanup(() => {
    bodyAutosave.flush();
    pathAutosave.flush();
    bodyAutosave.dispose();
    pathAutosave.dispose();
  });

  /** Persist only when the editor value differs from the stored baseline. */
  function noteBodyChange(id: string, markdown: string, baseline: string) {
    if (markdown === baseline) {
      setBodyOverrides((current) => {
        if (!(id in current)) return current;
        const { [id]: _removed, ...rest } = current;
        return rest;
      });
      bodyAutosave.cancel(id);
      return;
    }
    setBodyOverrides((current) => ({ ...current, [id]: markdown }));
    bodyAutosave.schedule(id, markdown);
  }

  function notePathChange(id: string, path: string, baseline: string) {
    if (path === baseline) {
      setPathOverrides((current) => {
        if (!(id in current)) return current;
        const { [id]: _removed, ...rest } = current;
        return rest;
      });
      pathAutosave.cancel(id);
      return;
    }
    setPathOverrides((current) => ({ ...current, [id]: path }));
    pathAutosave.schedule(id, path);
  }

  const allLibraryRows = createMemo(() =>
    libraryRowsFromWorkspace({
      documents: props.documents,
      adrs: props.adrs,
      runbooks: props.runbooks,
    }),
  );

  const libraryRows = createMemo(() =>
    filterLibraryRows(allLibraryRows(), libraryFilters()),
  );

  const sources = createMemo(() =>
    filterSources(props.sources ?? [], sourceFilters()),
  );

  const features = createMemo(() =>
    filterSpecFeatures(props.specFeatures ?? [], specFilters()),
  );

  let lastCollectionFingerprint = "";
  createEffect(() => {
    const scope = scopeId();
    const status = props.loading ? "loading" : props.error ? "error" : "ready";
    const constitutionTarget: CollectionTarget[] = props.loading
      ? []
      : props.constitution
      ? [{ scopeId: scope, entityId: props.constitution.id, entityKind: "constitution" }]
      : [];
    const targets: CollectionTarget[] = props.loading
      ? []
      : tab() === "library"
      ? libraryRows().map((row) => ({
          scopeId: scope,
          entityId: row.id,
          entityKind: row.entityKind,
        }))
      : tab() === "sources"
        ? sources().map((source) => ({
            scopeId: scope,
            entityId: source.id,
            entityKind: "source",
          }))
      : tab() === "specs"
        ? [
            ...constitutionTarget,
            ...features().map((feature) => ({
              scopeId: scope,
              entityId: feature.featureKey,
              entityKind: "feature",
            })),
          ]
        : [];

    const fingerprint = collectionFingerprint(scope, status, targets);
    if (fingerprint === lastCollectionFingerprint) return;

    const active = detailPanelStore.getState().activeTarget;
    if (
      active?.scopeId === scope
      && !targets.some(
        (target) => target.entityId === active.entityId && target.entityKind === active.entityKind,
      )
    ) {
      // Filtered-out selection: dismiss instead of falling back to targets[0] (constitution).
      detailPanelStore.getState().dismissTarget(scope);
    }

    lastCollectionFingerprint = fingerprint;
    detailPanelStore.getState().registerCollection({
      scopeId: scope,
      status,
      targets,
    });
    onCleanup(() => {
      lastCollectionFingerprint = "";
      detailPanelStore.getState().unregisterCollection(scope);
    });
  });

  const document = createMemo((): WorkspaceDocument | undefined => {
    const target = activeTarget();
    return target?.scopeId === "knowledge:library" && target.entityKind === "document"
      ? (props.documents ?? []).find((item) => item.id === target.entityId)
      : undefined;
  });

  const adr = createMemo((): WorkspaceAdr | undefined => {
    const target = activeTarget();
    return target?.scopeId === "knowledge:library" && target.entityKind === "adr"
      ? (props.adrs ?? []).find((item) => item.id === target.entityId)
      : undefined;
  });

  const runbook = createMemo((): WorkspaceRunbook | undefined => {
    const target = activeTarget();
    return target?.scopeId === "knowledge:library" && target.entityKind === "runbook"
      ? (props.runbooks ?? []).find((item) => item.id === target.entityId)
      : undefined;
  });

  const source = createMemo((): WorkspaceSource | undefined => {
    const target = activeTarget();
    return target?.scopeId === "knowledge:sources" && target.entityKind === "source"
      ? (props.sources ?? []).find((item) => item.id === target.entityId)
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
    entityKind: "document" | "adr" | "runbook" | "source" | "feature" | "constitution",
  ) {
    detailPanelStore.getState().activateTarget({ scopeId: scopeId(), entityId, entityKind });
    if (entityKind === "document") props.onOpenDoc?.(entityId);
    else if (entityKind === "adr") props.onOpenAdr?.(entityId);
    else if (entityKind === "runbook") props.onOpenRunbook?.(entityId);
    else if (entityKind === "source") props.onOpenSource?.(entityId);
    else if (entityKind === "constitution") props.onOpenConstitution?.(entityId);
    else props.onOpenFeature?.(entityId);
  }

  function activateLibraryRow(row: LibraryRow) {
    activate(row.id, row.entityKind);
  }

  const createKind = createMemo((): CreateKnowledgeKind => {
    if (tab() === "specs") return "feature";
    const kind = libraryEmptyCategory(libraryFilters().kinds);
    if (kind === "adr") return "adr";
    if (kind === "runbook") return "runbook";
    return "doc";
  });

  const pickLibraryKind = createMemo(
    () => tab() === "library" && libraryFilters().kinds.length !== 1,
  );

  const listSearch = createMemo(() => {
    if (tab() === "sources") {
      return {
        placeholder: "Search",
        value: sourceFilters().query,
        onInput: (value: string) =>
          setSourceFilters((current) => ({ ...current, query: value })),
      };
    }
    if (tab() === "specs") {
      return {
        placeholder: "Search",
        value: specFilters().query,
        onInput: (value: string) =>
          setSpecFilters((current) => ({ ...current, query: value })),
      };
    }
    return {
      placeholder: "Search",
      value: libraryFilters().query,
      onInput: (value: string) =>
        setLibraryFilters((current) => ({ ...current, query: value })),
    };
  });

  const createLabel = createMemo(() => {
    if (tab() === "sources") return "Add source";
    if (tab() === "specs") return "New feature";
    const kind = libraryEmptyCategory(libraryFilters().kinds);
    if (kind === "runbook") return "New runbook";
    if (kind === "adr") return "New ADR";
    if (kind === "doc") return "New document";
    return "New";
  });

  const libraryFilterGroups = createMemo((): FilterPopoverGroup[] => {
    const current = libraryFilters();
    const icon = (node: typeof FileText) => {
      const Icon = node;
      return <Icon size={14} aria-hidden />;
    };
    return [
      {
        id: "kind",
        label: "Kind",
        leading: icon(FileText),
        options: [
          { id: "doc", label: "Doc", leading: icon(FileText) },
          { id: "adr", label: "ADR", leading: icon(ScrollText) },
          { id: "runbook", label: "Runbook", leading: icon(BookOpen) },
        ].map((option) => ({
          ...option,
          checked: current.kinds.includes(option.id as LibraryCategory),
          onCheckedChange: (checked: boolean) =>
            setLibraryFilters((prev) => ({
              ...prev,
              kinds: toggleFilterValue(prev.kinds, option.id as LibraryCategory, checked),
            })),
        })),
      },
      {
        id: "status",
        label: "Status",
        leading: <StatusDot tone="ok" showLabel={false} />,
        options: [
          { id: "published", label: "Published" },
          { id: "draft", label: "Draft" },
          { id: "accepted", label: "Accepted" },
          { id: "proposed", label: "Proposed" },
        ].map((option) => ({
          ...option,
          leading: <StatusDot tone={libraryStatusTone(option.id)} showLabel={false} />,
          checked: current.statuses.includes(option.id),
          onCheckedChange: (checked: boolean) =>
            setLibraryFilters((prev) => ({
              ...prev,
              statuses: toggleFilterValue(prev.statuses, option.id, checked),
            })),
        })),
      },
      {
        id: "severity",
        label: "Severity",
        leading: <StatusDot tone="danger" showLabel={false} />,
        options: (
          [
            { id: "critical", label: "Critical" },
            { id: "high", label: "High" },
            { id: "medium", label: "Medium" },
            { id: "low", label: "Low" },
          ] as const
        ).map((option) => ({
          ...option,
          leading: (
            <StatusDot tone={runbookSeverityTone(option.id)} showLabel={false} />
          ),
          checked: current.severities.includes(option.id),
          onCheckedChange: (checked: boolean) =>
            setLibraryFilters((prev) => ({
              ...prev,
              severities: toggleFilterValue(prev.severities, option.id, checked),
            })),
        })),
      },
      {
        id: "linked",
        label: "Linked",
        leading: icon(Link),
        selectionMode: "single" as const,
        options: [
          {
            id: "linked",
            label: "Has links",
            leading: icon(Link),
          },
          {
            id: "unlinked",
            label: "No links",
            leading: <FilterMark shape="circle" tone="muted" variant="outline" />,
          },
        ].map((option) => ({
          ...option,
          checked: current.linked === option.id,
          onCheckedChange: (checked: boolean) =>
            setLibraryFilters((prev) => ({
              ...prev,
              linked: checked
                ? (option.id as "linked" | "unlinked")
                : "any",
            })),
        })),
      },
    ];
  });

  const specFilterGroups = createMemo((): FilterPopoverGroup[] => {
    const current = specFilters();
    const icon = (node: typeof FileText) => {
      const Icon = node;
      return <Icon size={14} aria-hidden />;
    };
    return [
      {
        id: "status",
        label: "Status",
        leading: <StatusDot tone="ok" showLabel={false} />,
        options: (
          [
            { id: "ready", label: "Ready" },
            { id: "draft", label: "Draft" },
          ] as const
        ).map((option) => ({
          ...option,
          leading: <StatusDot tone={specStatusTone(option.id)} showLabel={false} />,
          checked: current.statuses.includes(option.id),
          onCheckedChange: (checked: boolean) =>
            setSpecFilters((prev) => ({
              ...prev,
              statuses: toggleFilterValue(prev.statuses, option.id, checked),
            })),
        })),
      },
      {
        id: "checklist",
        label: "Checklist",
        leading: icon(ListChecks),
        selectionMode: "single" as const,
        options: [
          {
            id: "open",
            label: "Open items",
            leading: icon(ListChecks),
          },
          {
            id: "clear",
            label: "All clear",
            leading: <StatusDot tone="healthy" showLabel={false} />,
          },
        ].map((option) => ({
          ...option,
          checked: current.checklist === option.id,
          onCheckedChange: (checked: boolean) =>
            setSpecFilters((prev) => ({
              ...prev,
              checklist: checked
                ? (option.id as "open" | "clear")
                : "any",
            })),
        })),
      },
    ];
  });

  const sourceFilterGroups = createMemo((): FilterPopoverGroup[] => {
    const current = sourceFilters();
    const prefixes = distinctSourcePathPrefixes(props.sources ?? []);
    const icon = (node: typeof FileText) => {
      const Icon = node;
      return <Icon size={14} aria-hidden />;
    };
    return [
      {
        id: "linked",
        label: "Linked",
        leading: icon(Link),
        selectionMode: "single" as const,
        options: [
          {
            id: "linked",
            label: "Has links",
            leading: icon(Link),
          },
          {
            id: "unlinked",
            label: "No links",
            leading: <FilterMark shape="circle" tone="muted" variant="outline" />,
          },
        ].map((option) => ({
          ...option,
          checked: current.linked === option.id,
          onCheckedChange: (checked: boolean) =>
            setSourceFilters((prev) => ({
              ...prev,
              linked: checked
                ? (option.id as "linked" | "unlinked")
                : "any",
            })),
        })),
      },
      {
        id: "path",
        label: "Path prefix",
        leading: icon(Folder),
        options: prefixes.map((prefix) => ({
          id: prefix,
          label: prefix,
          checked: current.pathPrefixes.includes(prefix),
          onCheckedChange: (checked: boolean) =>
            setSourceFilters((prev) => ({
              ...prev,
              pathPrefixes: toggleFilterValue(prev.pathPrefixes, prefix, checked),
            })),
        })),
      },
    ].filter((group) => group.id !== "path" || group.options.length > 0);
  });

  const filterPopoverProps = createMemo(() => {
    if (tab() === "sources") {
      return {
        ariaLabel: "Source filters",
        title: "Filter sources",
        active: sourceListFiltersActive(sourceFilters()),
        onClear: () =>
          setSourceFilters((current) => ({
            ...EMPTY_SOURCE_LIST_FILTERS,
            query: current.query,
          })),
        groups: sourceFilterGroups(),
      };
    }
    if (tab() === "specs") {
      return {
        ariaLabel: "Feature filters",
        title: "Filter features",
        active: specListFiltersActive(specFilters()),
        onClear: () =>
          setSpecFilters((current) => ({
            ...EMPTY_SPEC_LIST_FILTERS,
            query: current.query,
          })),
        groups: specFilterGroups(),
      };
    }
    return {
      ariaLabel: "Library filters",
      title: "Filter library",
      active: libraryFiltersActive(libraryFilters()),
      onClear: () =>
        setLibraryFilters((current) => ({
          ...EMPTY_LIBRARY_FILTERS,
          query: current.query,
        })),
      groups: libraryFilterGroups(),
    };
  });

  const activeFilterPills = createMemo((): ActiveFilterPill[] => {
    if (tab() === "library") {
      const current = libraryFilters();
      const pills: ActiveFilterPill[] = [];
      for (const kind of current.kinds) {
        pills.push({
          id: `kind:${kind}`,
          label: `Kind: ${kind === "doc" ? "Doc" : kind === "adr" ? "ADR" : "Runbook"}`,
          onRemove: () =>
            setLibraryFilters((prev) => ({
              ...prev,
              kinds: prev.kinds.filter((value) => value !== kind),
            })),
        });
      }
      for (const status of current.statuses) {
        pills.push({
          id: `status:${status}`,
          label: `Status: ${status}`,
          onRemove: () =>
            setLibraryFilters((prev) => ({
              ...prev,
              statuses: prev.statuses.filter((value) => value !== status),
            })),
        });
      }
      for (const severity of current.severities) {
        pills.push({
          id: `severity:${severity}`,
          label: `Severity: ${severity}`,
          onRemove: () =>
            setLibraryFilters((prev) => ({
              ...prev,
              severities: prev.severities.filter((value) => value !== severity),
            })),
        });
      }
      if (current.linked !== "any") {
        pills.push({
          id: "linked",
          label: current.linked === "linked" ? "Linked: Has links" : "Linked: No links",
          onRemove: () =>
            setLibraryFilters((prev) => ({ ...prev, linked: "any" })),
        });
      }
      return pills;
    }
    if (tab() === "specs") {
      const current = specFilters();
      const pills: ActiveFilterPill[] = current.statuses.map((status) => ({
        id: `status:${status}`,
        label: `Status: ${status}`,
        onRemove: () =>
          setSpecFilters((prev) => ({
            ...prev,
            statuses: prev.statuses.filter((value) => value !== status),
          })),
      }));
      if (current.checklist !== "any") {
        pills.push({
          id: "checklist",
          label:
            current.checklist === "open"
              ? "Checklist: Open items"
              : "Checklist: All clear",
          onRemove: () =>
            setSpecFilters((prev) => ({ ...prev, checklist: "any" })),
        });
      }
      return pills;
    }
    const current = sourceFilters();
    const pills: ActiveFilterPill[] = [];
    if (current.linked !== "any") {
      pills.push({
        id: "linked",
        label: current.linked === "linked" ? "Linked: Has links" : "Linked: No links",
        onRemove: () =>
          setSourceFilters((prev) => ({ ...prev, linked: "any" })),
      });
    }
    for (const prefix of current.pathPrefixes) {
      pills.push({
        id: `path:${prefix}`,
        label: `Path: ${prefix}`,
        onRemove: () =>
          setSourceFilters((prev) => ({
            ...prev,
            pathPrefixes: prev.pathPrefixes.filter((value) => value !== prefix),
          })),
      });
    }
    return pills;
  });

  function linksFor(
    knowledgeId: string,
    links: WorkspaceDocument["entityLinks"] | WorkspaceAdr["entityLinks"] | WorkspaceRunbook["entityLinks"] | WorkspaceSource["entityLinks"],
  ) {
    return (
      <EntityLinks
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

  const hasLibraryItems = createMemo(() => allLibraryRows().length > 0);

  const libraryEmptyKind = createMemo(() =>
    libraryEmptyCategory(libraryFilters().kinds),
  );

  const libraryHasScopedItems = createMemo(() => {
    const kind = libraryEmptyKind();
    if (kind === "all") return hasLibraryItems();
    return allLibraryRows().some((row) => row.category === kind);
  });

  return (
    <Show when={!props.loading} fallback={<PageLoading layout="table" label="Loading knowledge…" />}>
    <>
      <div class="flex h-full min-h-0 flex-col">
        <PageToolbar.Root>
          <PageToolbar.Tabs>
            <Tabs
              items={[...TABS]}
              value={tab()}
              onChange={(next) => {
                setLibraryFilters({ ...EMPTY_LIBRARY_FILTERS });
                setSpecFilters({ ...EMPTY_SPEC_LIST_FILTERS });
                setSourceFilters({ ...EMPTY_SOURCE_LIST_FILTERS });
                setCreateOpen(false);
                setSourceCreateOpen(false);
                props.onTabChange?.(next);
              }}
              aria-label="Knowledge"
            />
          </PageToolbar.Tabs>
          <PageToolbar.Actions>
            <SearchBar
              placeholder={listSearch().placeholder}
              value={listSearch().value}
              onInput={listSearch().onInput}
              aria-label={listSearch().placeholder}
            />
            <FilterPopover
              ariaLabel={filterPopoverProps().ariaLabel}
              title={filterPopoverProps().title}
              active={filterPopoverProps().active}
              onClear={filterPopoverProps().onClear}
              groups={filterPopoverProps().groups}
            />
            <Show when={tab() === "sources" && props.onImportProjectSources}>
              <ToolbarIconButton
                label="Import from project root"
                disabled={props.importBusy}
                onClick={() => props.onImportProjectSources?.()}
              >
                <FileUp size={14} />
              </ToolbarIconButton>
            </Show>
            <Show when={tab() === "sources" && props.onCreateSource}>
              <ToolbarIconButton
                label={createLabel()}
                onClick={() => setSourceCreateOpen(true)}
              >
                <Plus size={14} />
              </ToolbarIconButton>
            </Show>
            <Show when={tab() !== "sources" && (props.onCreate || props.onCreateFeature)}>
              <ToolbarIconButton
                label={createLabel()}
                onClick={() => setCreateOpen(true)}
              >
                <Plus size={14} />
              </ToolbarIconButton>
            </Show>
          </PageToolbar.Actions>
        </PageToolbar.Root>

        <ActiveFilterBar
          items={activeFilterPills()}
          onClear={filterPopoverProps().onClear}
        />

        <Show when={tab() === "library"}>
          <PageContent class="p-0">
            <Show
              when={libraryHasScopedItems()}
              fallback={
                <LibraryEmpty
                  category={libraryEmptyKind()}
                  onCreate={props.onCreate ? () => setCreateOpen(true) : undefined}
                  onCreateFromTemplate={props.onCreateFromTemplate}
                />
              }
            >
              <Show when={libraryRows().length > 0} fallback={libraryFilterEmpty()}>
                <LibraryTable
                  rows={libraryRows()}
                  activeRowId={
                    activeTarget()?.scopeId === "knowledge:library"
                      ? activeTarget()?.entityId
                      : null
                  }
                  onActivate={activateLibraryRow}
                />
              </Show>
            </Show>
          </PageContent>
        </Show>

        <Show when={tab() === "sources"}>
          <PageContent class="p-0">
            <Show
              when={(props.sources ?? []).length > 0}
              fallback={
                <SourcesEmpty
                  onCreate={props.onCreateSource ? () => setSourceCreateOpen(true) : undefined}
                  onImport={
                    props.onImportProjectSources
                      ? () => props.onImportProjectSources?.()
                      : undefined
                  }
                  importBusy={props.importBusy}
                  onCreateFromTemplate={props.onCreateFromTemplate}
                />
              }
            >
              <Show when={sources().length > 0} fallback={sourcesFilterEmpty()}>
                <SourcesTable
                  rows={sources()}
                  activeRowId={
                    activeTarget()?.scopeId === "knowledge:sources"
                      ? activeTarget()?.entityId
                      : null
                  }
                  onActivate={(id) => activate(id, "source")}
                />
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
                          <DataTable.Cell class="font-mono pw-selectable text-[12px]">{item.featureKey}</DataTable.Cell>
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
              noteBodyChange(item().id, markdown, item().bodyMarkdown);
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
              environment: "",
              status: "published",
              owner: item().owner,
              entityLinks: item().entityLinks,
            }}
            bodyMarkdown={bodyOverrides()[item().id] ?? item().bodyMarkdown}
            onBodyChange={(markdown) => {
              noteBodyChange(item().id, markdown, item().bodyMarkdown);
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
              noteBodyChange(item().id, markdown, item().bodyMarkdown);
            }}
            onOpenComponent={props.onOpenComponent}
            onArchive={props.onArchive ? () => props.onArchive?.(item().id) : undefined}
            onDuplicate={props.onDuplicate ? () => props.onDuplicate?.(item().id) : undefined}
            links={linksFor(item().id, item().entityLinks)}
          />
        )}
      </Show>
      <Show when={source()}>
        {(item) => (
          <SourceDetail
            source={item()}
            bodyMarkdown={bodyOverrides()[item().id] ?? item().bodyMarkdown}
            pathValue={pathOverrides()[item().id] ?? item().path}
            onBodyChange={(markdown) => {
              noteBodyChange(item().id, markdown, item().bodyMarkdown);
            }}
            onPathChange={
              props.onSaveSourcePath
                ? (path) => {
                    notePathChange(item().id, path, item().path);
                  }
                : undefined
            }
            onArchive={props.onArchive ? () => props.onArchive?.(item().id) : undefined}
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
              const baseline = item()[activeArtifact()].bodyMarkdown;
              noteBodyChange(artifactId, markdown, baseline);
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
              noteBodyChange(item().id, markdown, item().bodyMarkdown);
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
        pickLibraryKind={pickLibraryKind()}
        onClose={() => setCreateOpen(false)}
        onCreateFromTemplate={
          props.onCreateFromTemplate
            ? (template) => {
                props.onCreateFromTemplate?.(template);
                setCreateOpen(false);
              }
            : undefined
        }
        onSubmit={(title, kind) => {
          if (kind === "feature") props.onCreateFeature?.(title);
          else if (kind === "runbook") props.onCreate?.("runbook", title);
          else if (kind === "adr") props.onCreate?.("adr", title);
          else props.onCreate?.("doc", title);
          setCreateOpen(false);
        }}
      />
      <CreateSourceDialog
        open={sourceCreateOpen()}
        onClose={() => setSourceCreateOpen(false)}
        onCreateFromTemplate={
          props.onCreateFromTemplate
            ? (template) => {
                props.onCreateFromTemplate?.(template);
                setSourceCreateOpen(false);
              }
            : undefined
        }
        onSubmit={(input) => {
          props.onCreateSource?.(input);
          setSourceCreateOpen(false);
        }}
      />
    </>
    </Show>
  );
}
