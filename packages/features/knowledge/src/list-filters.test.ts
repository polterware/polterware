import { describe, expect, it } from "vitest";
import {
  EMPTY_SOURCE_LIST_FILTERS,
  EMPTY_SPEC_LIST_FILTERS,
  distinctSourcePathPrefixes,
  filterSources,
  filterSpecFeatures,
  sourcePathPrefix,
} from "./list-filters";
import type { WorkspaceSource, WorkspaceSpecFeature } from "@polterware/feature-shell";

const feature = {
  featureKey: "auth",
  title: "Auth",
  status: "draft",
  updatedAt: "2026-01-01",
  owner: "Ada",
  summary: "Login",
  spec: {
    id: "s",
    kind: "spec",
    title: "Spec",
    bodyMarkdown: "",
    status: "draft",
    updatedAt: "2026-01-01",
    owner: "Ada",
    featureKey: "auth",
  },
  plan: {
    id: "p",
    kind: "plan",
    title: "Plan",
    bodyMarkdown: "",
    status: "draft",
    updatedAt: "2026-01-01",
    owner: "Ada",
    featureKey: "auth",
  },
  tasks: {
    id: "t",
    kind: "tasks",
    title: "Tasks",
    bodyMarkdown: "",
    status: "draft",
    updatedAt: "2026-01-01",
    owner: "Ada",
    featureKey: "auth",
  },
  clarify: {
    id: "c",
    kind: "clarify",
    title: "Clarify",
    bodyMarkdown: "",
    status: "draft",
    updatedAt: "2026-01-01",
    owner: "Ada",
    featureKey: "auth",
  },
  checklist: {
    id: "ch",
    kind: "checklist",
    title: "Checklist",
    bodyMarkdown: "- [ ] Open item\n",
    status: "draft",
    updatedAt: "2026-01-01",
    owner: "Ada",
    featureKey: "auth",
  },
} satisfies WorkspaceSpecFeature;

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

  it("filters specs by status and checklist", () => {
    expect(
      filterSpecFeatures([feature], {
        ...EMPTY_SPEC_LIST_FILTERS,
        statuses: ["ready"],
      }),
    ).toHaveLength(0);
    expect(
      filterSpecFeatures([feature], {
        ...EMPTY_SPEC_LIST_FILTERS,
        checklist: "open",
      }),
    ).toHaveLength(1);
    expect(
      filterSpecFeatures([feature], {
        ...EMPTY_SPEC_LIST_FILTERS,
        checklist: "clear",
      }),
    ).toHaveLength(0);
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
