import type {
  WorkspaceAdr,
  WorkspaceDocument,
  WorkspaceRunbook,
} from "@polterware/feature-shell";
import type { StatusTone } from "@polterware/frontend-design-system";
import {
  documentStatusTone,
  runbookSeverityTone,
} from "./knowledge-document-helpers";

export type LibraryCategory = "doc" | "adr" | "runbook";

export type LibraryEntityKind = "document" | "adr" | "runbook";

export type LibraryLinkedFilter = "any" | "linked" | "unlinked";

export type LibraryFilters = {
  kinds: readonly LibraryCategory[];
  statuses: readonly string[];
  severities: readonly string[];
  linked: LibraryLinkedFilter;
  query: string;
};

export const EMPTY_LIBRARY_FILTERS: LibraryFilters = {
  kinds: [],
  statuses: [],
  severities: [],
  linked: "any",
  query: "",
};

export type LibraryRow = {
  id: string;
  category: LibraryCategory;
  entityKind: LibraryEntityKind;
  title: string;
  categoryLabel: string;
  statusLabel: string;
  statusTone: StatusTone;
  meta: string;
  updatedAt: string;
  hasLinks: boolean;
};

export const LIBRARY_CATEGORY_LABEL: Record<LibraryCategory, string> = {
  doc: "Doc",
  adr: "ADR",
  runbook: "Runbook",
};

export function libraryRowsFromWorkspace(input: {
  documents?: WorkspaceDocument[];
  adrs?: WorkspaceAdr[];
  runbooks?: WorkspaceRunbook[];
}): LibraryRow[] {
  const docs = (input.documents ?? [])
    .filter((document) => !document.archived)
    .map((document): LibraryRow => ({
      id: document.id,
      category: "doc",
      entityKind: "document",
      title: document.title,
      categoryLabel: LIBRARY_CATEGORY_LABEL.doc,
      statusLabel: document.status,
      statusTone: documentStatusTone(document.status),
      meta: document.author,
      updatedAt: document.updatedAt,
      hasLinks: document.entityLinks.length > 0,
    }));

  const adrs = (input.adrs ?? []).map((adr): LibraryRow => ({
    id: adr.id,
    category: "adr",
    entityKind: "adr",
    title: adr.title,
    categoryLabel: LIBRARY_CATEGORY_LABEL.adr,
    statusLabel: adr.state,
    statusTone: decisionStateTone(adr.state),
    meta: adr.owner,
    updatedAt: adr.updatedAt,
    hasLinks: adr.entityLinks.length > 0,
  }));

  const runbooks = (input.runbooks ?? []).map((runbook): LibraryRow => ({
    id: runbook.id,
    category: "runbook",
    entityKind: "runbook",
    title: runbook.title,
    categoryLabel: LIBRARY_CATEGORY_LABEL.runbook,
    statusLabel: runbook.severity,
    statusTone: runbookSeverityTone(runbook.severity),
    meta: runbook.service,
    updatedAt: runbook.lastValidated,
    hasLinks: runbook.entityLinks.length > 0,
  }));

  return [...docs, ...adrs, ...runbooks];
}

export function libraryFiltersActive(filters: LibraryFilters): boolean {
  return (
    filters.kinds.length > 0 ||
    filters.statuses.length > 0 ||
    filters.severities.length > 0 ||
    filters.linked !== "any"
  );
}

/** Empty-state category when exactly one kind is selected; otherwise all. */
export function libraryEmptyCategory(
  kinds: readonly LibraryCategory[],
): "all" | LibraryCategory {
  return kinds.length === 1 ? kinds[0]! : "all";
}

export function filterLibraryRows(
  rows: LibraryRow[],
  filters: LibraryFilters,
): LibraryRow[] {
  const needle = filters.query.trim().toLowerCase();
  return rows.filter((row) => {
    if (filters.kinds.length > 0 && !filters.kinds.includes(row.category)) {
      return false;
    }
    if (filters.statuses.length > 0) {
      if (row.category === "runbook") return false;
      if (!filters.statuses.includes(row.statusLabel)) return false;
    }
    if (filters.severities.length > 0) {
      if (row.category !== "runbook") return false;
      if (!filters.severities.includes(row.statusLabel)) return false;
    }
    if (filters.linked === "linked" && !row.hasLinks) return false;
    if (filters.linked === "unlinked" && row.hasLinks) return false;
    if (!needle) return true;
    return [row.title, row.meta, row.categoryLabel, row.statusLabel].some(
      (value) => value.toLowerCase().includes(needle),
    );
  });
}

export function toggleFilterValue<T extends string>(
  current: readonly T[],
  id: T,
  checked: boolean,
): T[] {
  return checked ? [...current, id] : current.filter((value) => value !== id);
}

/** Tone of an ADR lifecycle state chip. */
export function decisionStateTone(state: WorkspaceAdr["state"]): StatusTone {
  switch (state) {
    case "active":
      return "ok";
    case "challenged":
      return "warn";
    default:
      return "muted";
  }
}
