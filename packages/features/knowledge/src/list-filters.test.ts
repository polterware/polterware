import { describe, expect, it } from "vitest";
import {
  EMPTY_SOURCE_LIST_FILTERS,
  EMPTY_CARD_LIST_FILTERS,
  distinctSourcePathPrefixes,
  filterSources,
  filterWorkCards,
  sourcePathPrefix,
} from "./list-filters";
import type { WorkspaceSource, WorkspaceWorkCard } from "@polterware/feature-shell";

const card = {
  id: "01J",
  slug: "auth",
  title: "Auth",
  intent: "Login",
  stage: "plan",
  owner: "Ada",
  under: "",
  decisions: [],
  updatedAt: "2026-01-01",
  tasks: [],
  questions: [{ id: "q1", text: "SSO?", blocking: false, options: [], state: "open", answer: "" }],
  invariants: [],
} satisfies WorkspaceWorkCard;

const source = {
  id: "src-1",
  title: "Contracts",
  path: "schemas/local/v1/ppm.proto",
  summary: "",
  bodyMarkdown: "",
  updatedAt: "2026-01-01",
  owner: "Ada",
  entityLinks: [],
} satisfies WorkspaceSource;

describe("list-filters", () => {
  it("derives path prefixes", () => {
    expect(sourcePathPrefix("schemas/local/v1/ppm.proto")).toBe("schemas");
    expect(distinctSourcePathPrefixes([source])).toEqual(["schemas"]);
  });

  it("filters cards by stage, open questions, and query", () => {
    expect(filterWorkCards([card], { ...EMPTY_CARD_LIST_FILTERS, stages: ["build"] })).toHaveLength(0);
    expect(filterWorkCards([card], { ...EMPTY_CARD_LIST_FILTERS, stages: ["plan"] })).toHaveLength(1);
    expect(filterWorkCards([card], { ...EMPTY_CARD_LIST_FILTERS, questions: "open" })).toHaveLength(1);
    expect(filterWorkCards([card], { ...EMPTY_CARD_LIST_FILTERS, questions: "clear" })).toHaveLength(0);
    expect(filterWorkCards([card], { ...EMPTY_CARD_LIST_FILTERS, query: "login" })).toHaveLength(1);
  });

  it("filters sources by path prefix and linked", () => {
    expect(
      filterSources([source], {
        ...EMPTY_SOURCE_LIST_FILTERS,
        pathPrefixes: ["schemas"],
      }),
    ).toHaveLength(1);
    expect(
      filterSources([source], {
        ...EMPTY_SOURCE_LIST_FILTERS,
        linked: "linked",
      }),
    ).toHaveLength(0);
  });
});
