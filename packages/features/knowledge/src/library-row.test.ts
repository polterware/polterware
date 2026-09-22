import { describe, expect, it } from "vitest";
import {
  EMPTY_LIBRARY_FILTERS,
  filterLibraryRows,
  libraryEmptyCategory,
  libraryFiltersActive,
  libraryRowsFromWorkspace,
  toggleFilterValue,
} from "./library-row";
import type { WorkspaceAdr, WorkspaceDocument, WorkspaceRunbook } from "@polterware/feature-shell";

const doc = {
  id: "doc-1",
  title: "Overview",
  summary: "",
  bodyMarkdown: "",
  author: "Ada",
  updatedAt: "2026-01-01",
  tags: [],
  componentIds: [],
  componentNames: [],
  environment: "All",
  status: "published",
  owner: "Ada",
  entityLinks: [{
    id: "link-1",
    entityId: "e1",
    entityTitle: "Auth",
    entityKind: "component",
    entityRefId: "auth",
    predicate: "documents",
    truthClass: "declared",
    source: "user",
  }],
} satisfies WorkspaceDocument;

const adr = {
  id: "adr-1",
  title: "Local-first",
  summary: "",
  bodyMarkdown: "",
  updatedAt: "2026-01-02",
  tags: ["adr"],
  status: "accepted",
  owner: "Ada",
  entityLinks: [],
} satisfies WorkspaceAdr;

const runbook = {
  id: "rb-1",
  title: "Restart",
  objective: "",
  bodyMarkdown: "",
  service: "api",
  severity: "high",
  prerequisites: [],
  steps: [],
  commands: [],
  rollback: "",
  owner: "Ops",
  lastValidated: "2026-01-03",
  entityLinks: [],
} satisfies WorkspaceRunbook;

describe("libraryRowsFromWorkspace", () => {
  it("merges docs, adrs, and runbooks with link flags", () => {
    const rows = libraryRowsFromWorkspace({
      documents: [doc],
      adrs: [adr],
      runbooks: [runbook],
    });
    expect(rows.map((row) => row.category)).toEqual(["doc", "adr", "runbook"]);
    expect(rows[0]?.hasLinks).toBe(true);
    expect(rows[1]?.hasLinks).toBe(false);
  });

  it("filters by kind, status, severity, linked, and query", () => {
    const rows = libraryRowsFromWorkspace({
      documents: [doc],
      adrs: [adr],
      runbooks: [runbook],
    });
    expect(
      filterLibraryRows(rows, {
        ...EMPTY_LIBRARY_FILTERS,
        kinds: ["adr"],
        statuses: ["accepted"],
      }),
    ).toHaveLength(1);
    expect(
      filterLibraryRows(rows, {
        ...EMPTY_LIBRARY_FILTERS,
        kinds: ["runbook"],
        query: "restart",
      })[0]?.id,
    ).toBe("rb-1");
    expect(
      filterLibraryRows(rows, {
        ...EMPTY_LIBRARY_FILTERS,
        severities: ["high"],
      }).map((row) => row.id),
    ).toEqual(["rb-1"]);
    expect(
      filterLibraryRows(rows, {
        ...EMPTY_LIBRARY_FILTERS,
        statuses: ["published"],
      }).map((row) => row.id),
    ).toEqual(["doc-1"]);
    expect(
      filterLibraryRows(rows, {
        ...EMPTY_LIBRARY_FILTERS,
        linked: "linked",
      }).map((row) => row.id),
    ).toEqual(["doc-1"]);
    expect(
      filterLibraryRows(rows, {
        ...EMPTY_LIBRARY_FILTERS,
        linked: "unlinked",
      }).map((row) => row.id),
    ).toEqual(["adr-1", "rb-1"]);
  });

  it("reports active filters and empty category", () => {
    expect(libraryFiltersActive(EMPTY_LIBRARY_FILTERS)).toBe(false);
    expect(
      libraryFiltersActive({ ...EMPTY_LIBRARY_FILTERS, kinds: ["doc"] }),
    ).toBe(true);
    expect(libraryEmptyCategory([])).toBe("all");
    expect(libraryEmptyCategory(["adr"])).toBe("adr");
    expect(libraryEmptyCategory(["doc", "adr"])).toBe("all");
    expect(toggleFilterValue(["doc"], "adr", true)).toEqual(["doc", "adr"]);
    expect(toggleFilterValue(["doc", "adr"], "doc", false)).toEqual(["adr"]);
  });
});
