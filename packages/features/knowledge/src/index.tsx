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
  type WorkspaceWorkCard,
  type WorkStage,
} from "@polterware/feature-shell";
import { createDetailPanelSelector, detailPanelStore } from "@polterware/frontend-state";
import { ConstitutionDetail } from "./constitution-detail";
import { KnowledgeConstitutionEmpty } from "./constitution-empty";
import { ConstitutionRow } from "./constitution-row";
import { CreateKnowledgeDialog, type CreateKnowledgeKind } from "./create-knowledge-dialog";
import { CreateSourceDialog } from "./create-source-dialog";
import type { KnowledgeCreateTemplate } from "./create-templates";
import { createDocumentAutosave } from "./document-autosave";
import { DecisionDetail, type DecisionAction } from "./decision-detail";
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
  EMPTY_CARD_LIST_FILTERS,
  EMPTY_SOURCE_LIST_FILTERS,
  cardListFiltersActive,
  distinctSourcePathPrefixes,
  filterSources,
  filterWorkCards,
  sourceListFiltersActive,
  type CardListFilters,
  type SourceListFilters,
} from "./list-filters";
import { LibraryTable } from "./library-table";
import { RunbookDetail } from "./runbook-detail";
import { SourceDetail } from "./source-detail";
import { SourcesEmpty, sourcesFilterEmpty } from "./sources-empty";
import { SourcesTable } from "./sources-table";
import { libraryStatusTone, runbookSeverityTone } from "./knowledge-document-helpers";
import { WorkCardsEmpty, workCardsFilterEmpty } from "./knowledge-empty";
import { WorkBoard } from "./work-board";
import { WorkCardDetail } from "./work-card-detail";
import { WORK_STAGES, cardTaskProgress, workStageLabel, workStageTone } from "./work-stage";

export { cardSlugFromTitle, uniqueCardSlug } from "./card-slug";
export type { KnowledgeCreateTemplate, KnowledgeTemplateKind } from "./create-templates";
export {
  KNOWLEDGE_CREATE_TEMPLATES,
  knowledgeTemplateById,
  knowledgeTemplatesForKind,
} from "./create-templates";
export { CONSTITUTION_BODY_TEMPLATE } from "./constitution-template";
export { ChangeEnvelopeDialogHost } from "./change-envelope-dialog";
export type { DecisionAction } from "./decision-detail";
export { WORK_STAGES, workStageLabel } from "./work-stage";
export { NewFeatureFlow, type FeatureFlowLine } from "./new-feature-flow";
export {
  requestChangeEnvelope,
  type ChangeEnvelope,
  type ChangeEnvelopeRequest,
} from "./change-envelope-request";

const TABS: TabItem[] = [
  {
    id: "specs",
    label: "Register",
    tooltip: {
      title: "Register",
      description:
        "One Constitution per project, plus every Work card as a list. Agents write cards with polterware apply.",
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

type KnowledgeTab = "board" | "specs" | "library" | "sources";

type CollectionTarget = {
  scopeId: string;
  entityId: string;
  entityKind: string;
};

function normalizeTab(raw?: string): KnowledgeTab {
  if (raw === "board" || raw === "library" || raw === "sources") return raw;
  if (raw === "docs" || raw === "adrs" || raw === "runbooks") return "library";
  return "specs";
}

function isCardScope(scopeId?: string): boolean {
  return scopeId === "knowledge:specs" || scopeId === "knowledge:board";
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
  workCards?: WorkspaceWorkCard[];
  constitution?: WorkspaceConstitution | null;
  entities?: WorkspacePpmEntity[];
  /** Content mode: board, specs, library, or sources. */
  tab?: string;
  /** Tab rail supplied by the host section; defaults to the Knowledge tabs. */
  tabs?: TabItem[];
  /** Selected rail id when the rail differs from the content mode. */
  activeTab?: string;
  /** Accessible name for the rail, or the toolbar title when the rail has one item. */
  tabsLabel?: string;
  /** Restrict Library to these kinds (e.g. ADRs only). */
  libraryKinds?: readonly LibraryCategory[];
  /** Restrict ADR rows to one lifecycle state (Decisions tabs). */
  libraryDecisionState?: WorkspaceAdr["state"];
  loading?: boolean;
  error?: string;
  onTabChange?: (tab: string) => void;
  onOpenDoc?: (id: string) => void;
  onOpenAdr?: (id: string) => void;
  /** Writes one ADR lifecycle move. */
  onDecisionAction?: (adrId: string, action: DecisionAction) => void;
  onOpenRunbook?: (id: string) => void;
  onOpenSource?: (id: string) => void;
  onOpenCard?: (slug: string) => void;
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
  onCreateCard?: (title: string) => void;
  /** Opens the new-feature flow; when set it replaces the plain card dialog. */
  onNewFeature?: () => void;
  /** Reopens the flow for a card in shape or plan. */
  onContinueFeature?: (slug: string) => void;
  onCreateFromTemplate?: (template: KnowledgeCreateTemplate) => void;
  onEnsureConstitution?: () => void;
  onArchive?: (id: string) => void;
  onRemoveCard?: (slug: string) => void;
  onArchiveConstitution?: () => void;
  onSetCardStage?: (slug: string, stage: WorkStage) => void;
  onDuplicate?: (id: string) => void;
}) {
  const tab = () => normalizeTab(props.tab);
  /** Board and Specs both list Work cards and share the card detail. */
  const cardTab = () => tab() === "board" || tab() === "specs";
  const railItems = () => props.tabs ?? TABS;
  const [libraryFilters, setLibraryFilters] = createSignal<LibraryFilters>({
    ...EMPTY_LIBRARY_FILTERS,
  });
  const [cardFilters, setCardFilters] = createSignal<CardListFilters>({
    ...EMPTY_CARD_LIST_FILTERS,
  });
  const [sourceFilters, setSourceFilters] = createSignal<SourceListFilters>({
    ...EMPTY_SOURCE_LIST_FILTERS,
  });
  const [bodyOverrides, setBodyOverrides] = createSignal<Record<string, string>>({});
  const [pathOverrides, setPathOverrides] = createSignal<Record<string, string>>({});
  const [createOpen, setCreateOpen] = createSignal(false);
  const [sourceCreateOpen, setSourceCreateOpen] = createSignal(false);
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

  const allLibraryRows = createMemo(() => {
    const rows = libraryRowsFromWorkspace({
      documents: props.documents,
      adrs: props.adrs,
      runbooks: props.runbooks,
    });
    const kinds = props.libraryKinds;
    const state = props.libraryDecisionState;
    const byKind = kinds && kinds.length > 0
      ? rows.filter((row) => kinds.includes(row.category))
      : rows;
    return state ? byKind.filter((row) => row.category !== "adr" || row.statusLabel === state) : byKind;
  });

  /** Filter kinds when set, otherwise the locked kinds, for create and empty copy. */
  const effectiveLibraryKinds = createMemo(() =>
    libraryFilters().kinds.length > 0 ? libraryFilters().kinds : [...(props.libraryKinds ?? [])],
  );

  const libraryRows = createMemo(() =>
    filterLibraryRows(allLibraryRows(), libraryFilters()),
  );

  const sources = createMemo(() =>
    filterSources(props.sources ?? [], sourceFilters()),
  );

  const cards = createMemo(() =>
    filterWorkCards(props.workCards ?? [], cardFilters()),
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
      : tab() === "board"
        ? cards().map((card) => ({
            scopeId: scope,
            entityId: card.slug,
            entityKind: "card",
          }))
      : tab() === "specs"
        ? [
            ...constitutionTarget,
            ...cards().map((card) => ({
              scopeId: scope,
              entityId: card.slug,
              entityKind: "card",
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

  const card = createMemo((): WorkspaceWorkCard | undefined => {
    const target = activeTarget();
    return isCardScope(target?.scopeId) && target?.entityKind === "card"
      ? (props.workCards ?? []).find((item) => item.slug === target.entityId)
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

  function activate(
    entityId: string,
    entityKind: "document" | "adr" | "runbook" | "source" | "card" | "constitution",
  ) {
    detailPanelStore.getState().activateTarget({ scopeId: scopeId(), entityId, entityKind });
    if (entityKind === "document") props.onOpenDoc?.(entityId);
    else if (entityKind === "adr") props.onOpenAdr?.(entityId);
    else if (entityKind === "runbook") props.onOpenRunbook?.(entityId);
    else if (entityKind === "source") props.onOpenSource?.(entityId);
    else if (entityKind === "constitution") props.onOpenConstitution?.(entityId);
    else props.onOpenCard?.(entityId);
  }

  function activateLibraryRow(row: LibraryRow) {
    activate(row.id, row.entityKind);
  }

  const createKind = createMemo((): CreateKnowledgeKind => {
    if (cardTab()) return "card";
    const kind = libraryEmptyCategory(effectiveLibraryKinds());
    if (kind === "adr") return "adr";
    if (kind === "runbook") return "runbook";
    return "doc";
  });

  const pickLibraryKind = createMemo(
    () => tab() === "library" && effectiveLibraryKinds().length !== 1,
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
    if (cardTab()) {
      return {
        placeholder: "Search",
        value: cardFilters().query,
        onInput: (value: string) =>
          setCardFilters((current) => ({ ...current, query: value })),
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
    if (cardTab()) return "New card";
    const kind = libraryEmptyCategory(effectiveLibraryKinds());
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
        ].filter((option) =>
          !props.libraryKinds?.length || props.libraryKinds.includes(option.id as LibraryCategory),
        ).map((option) => ({
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
          { id: "active", label: "Active" },
          { id: "challenged", label: "Challenged" },
          { id: "proposed", label: "Proposed" },
          { id: "superseded", label: "Superseded" },
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
    ].filter((group) => group.id !== "kind" || group.options.length > 1);
  });

  const cardFilterGroups = createMemo((): FilterPopoverGroup[] => {
    const current = cardFilters();
    const icon = (node: typeof FileText) => {
      const Icon = node;
      return <Icon size={14} aria-hidden />;
    };
    return [
      {
        id: "stage",
        label: "Stage",
        leading: <StatusDot tone="ok" showLabel={false} />,
        options: WORK_STAGES.map((stage) => ({
          id: stage.id,
          label: stage.label,
          leading: <StatusDot tone={workStageTone(stage.id)} showLabel={false} />,
          checked: current.stages.includes(stage.id),
          onCheckedChange: (checked: boolean) =>
            setCardFilters((prev) => ({
              ...prev,
              stages: toggleFilterValue(prev.stages, stage.id, checked),
            })),
        })),
      },
      {
        id: "questions",
        label: "Questions",
        leading: icon(ListChecks),
        selectionMode: "single" as const,
        options: [
          {
            id: "open",
            label: "Open questions",
            leading: icon(ListChecks),
          },
          {
            id: "clear",
            label: "No open questions",
            leading: <StatusDot tone="healthy" showLabel={false} />,
          },
        ].map((option) => ({
          ...option,
          checked: current.questions === option.id,
          onCheckedChange: (checked: boolean) =>
            setCardFilters((prev) => ({
              ...prev,
              questions: checked
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
    if (cardTab()) {
      return {
        ariaLabel: "Card filters",
        title: "Filter cards",
        active: cardListFiltersActive(cardFilters()),
        onClear: () =>
          setCardFilters((current) => ({
            ...EMPTY_CARD_LIST_FILTERS,
            query: current.query,
          })),
        groups: cardFilterGroups(),
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
    if (cardTab()) {
      const current = cardFilters();
      const pills: ActiveFilterPill[] = current.stages.map((stage) => ({
        id: `stage:${stage}`,
        label: `Stage: ${workStageLabel(stage)}`,
        onRemove: () =>
          setCardFilters((prev) => ({
            ...prev,
            stages: prev.stages.filter((value) => value !== stage),
          })),
      }));
      if (current.questions !== "any") {
        pills.push({
          id: "questions",
          label:
            current.questions === "open"
              ? "Questions: Open"
              : "Questions: None open",
          onRemove: () =>
            setCardFilters((prev) => ({ ...prev, questions: "any" })),
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
    libraryEmptyCategory(effectiveLibraryKinds()),
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
          <Show
            when={railItems().length > 1}
            fallback={
              <PageToolbar.Leading>
                <span class="pw-page-toolbar-title">
                  {props.tabsLabel ?? railItems()[0]?.label ?? "Knowledge"}
                </span>
              </PageToolbar.Leading>
            }
          >
            <PageToolbar.Tabs>
              <Tabs
                items={[...railItems()]}
                value={props.activeTab ?? tab()}
                onChange={(next) => {
                  setLibraryFilters({ ...EMPTY_LIBRARY_FILTERS });
                  setCardFilters({ ...EMPTY_CARD_LIST_FILTERS });
                  setSourceFilters({ ...EMPTY_SOURCE_LIST_FILTERS });
                  setCreateOpen(false);
                  setSourceCreateOpen(false);
                  props.onTabChange?.(next);
                }}
                aria-label={props.tabsLabel ?? "Knowledge"}
              />
            </PageToolbar.Tabs>
          </Show>
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
            <Show when={tab() !== "sources" && (cardTab() ? props.onCreateCard : props.onCreate)}>
              <ToolbarIconButton
                label={createLabel()}
                onClick={() => (cardTab() && props.onNewFeature ? props.onNewFeature() : setCreateOpen(true))}
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

        <Show when={tab() === "board"}>
          <PageContent class="p-0">
            <Show
              when={(props.workCards ?? []).length > 0}
              fallback={
                <WorkCardsEmpty
                  onCreate={props.onNewFeature ?? (props.onCreateCard ? () => setCreateOpen(true) : undefined)}
                  onCreateFromTemplate={props.onCreateFromTemplate}
                />
              }
            >
              <Show when={cards().length > 0} fallback={workCardsFilterEmpty()}>
                <WorkBoard
                  cards={cards()}
                  activeSlug={
                    activeTarget()?.scopeId === "knowledge:board"
                    && activeTarget()?.entityKind === "card"
                      ? activeTarget()?.entityId
                      : null
                  }
                  onActivate={(slug) => activate(slug, "card")}
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
              when={(props.workCards ?? []).length > 0}
              fallback={
                <WorkCardsEmpty
                  onCreate={props.onNewFeature ?? (props.onCreateCard ? () => setCreateOpen(true) : undefined)}
                  onCreateFromTemplate={props.onCreateFromTemplate}
                />
              }
            >
              <Show when={cards().length > 0} fallback={workCardsFilterEmpty()}>
                <DataTable.Root
                  rows={cards()}
                  getRowId={(item) => item.slug}
                  activeRowId={
                    activeTarget()?.scopeId === "knowledge:specs"
                    && activeTarget()?.entityKind === "card"
                      ? activeTarget()?.entityId
                      : null
                  }
                  activateFirstRow={!props.constitution}
                  onActiveRowChange={(id) => id && activate(id, "card")}
                >
                  <DataTable.Table aria-label="Cards">
                    <DataTable.Header>
                      <DataTable.ColumnHeader<WorkspaceWorkCard> id="title">Card</DataTable.ColumnHeader>
                      <DataTable.ColumnHeader<WorkspaceWorkCard> id="slug">Slug</DataTable.ColumnHeader>
                      <DataTable.ColumnHeader<WorkspaceWorkCard> id="stage">Stage</DataTable.ColumnHeader>
                      <DataTable.ColumnHeader<WorkspaceWorkCard> id="tasks">Tasks</DataTable.ColumnHeader>
                      <DataTable.ColumnHeader<WorkspaceWorkCard> id="updatedAt">Updated</DataTable.ColumnHeader>
                    </DataTable.Header>
                    <DataTable.Body<WorkspaceWorkCard>>
                      {(item) => (
                        <DataTable.Row
                          row={item}
                          rowId={item.slug}
                          onActivate={() => activate(item.slug, "card")}
                        >
                          <DataTable.PrimaryCell icon={<ListChecks size={16} />} iconTone="knowledge" title={item.title} />
                          <DataTable.Cell class="font-mono pw-selectable text-[12px]">{item.slug}</DataTable.Cell>
                          <DataTable.Cell><StatusChip tone={workStageTone(item.stage)} label={workStageLabel(item.stage)} /></DataTable.Cell>
                          <DataTable.Cell class="tabular-nums text-[var(--color-text-muted)]">
                            {cardTaskProgress(item).total > 0
                              ? `${cardTaskProgress(item).done}/${cardTaskProgress(item).total}`
                              : "—"}
                          </DataTable.Cell>
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
          <DecisionDetail
            adr={item()}
            allAdrs={props.adrs ?? []}
            bodyMarkdown={bodyOverrides()[item().id] ?? item().bodyMarkdown}
            onBodyChange={(markdown) => {
              noteBodyChange(item().id, markdown, item().bodyMarkdown);
            }}
            onAction={props.onDecisionAction ? (action) => props.onDecisionAction?.(item().id, action) : undefined}
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
      <Show when={card()}>
        {(item) => (
          <WorkCardDetail
            card={item()}
            onSetStage={
              props.onSetCardStage
                ? (stage) => props.onSetCardStage?.(item().slug, stage)
                : undefined
            }
            onRemove={
              props.onRemoveCard
                ? () => props.onRemoveCard?.(item().slug)
                : undefined
            }
            onContinueFlow={
              props.onContinueFeature
                ? () => props.onContinueFeature?.(item().slug)
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
          if (kind === "card") props.onCreateCard?.(title);
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
