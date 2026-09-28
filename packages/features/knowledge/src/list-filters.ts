import type { WorkStage, WorkspaceSource, WorkspaceWorkCard } from "@polterware/feature-shell";
import { cardOpenQuestions } from "./work-stage";

export type CardQuestionsFilter = "any" | "open" | "clear";
export type LinkedFilter = "any" | "linked" | "unlinked";

export type CardListFilters = {
  stages: readonly WorkStage[];
  questions: CardQuestionsFilter;
  query: string;
};

export type SourceListFilters = {
  linked: LinkedFilter;
  pathPrefixes: readonly string[];
  query: string;
};

export const EMPTY_CARD_LIST_FILTERS: CardListFilters = {
  stages: [],
  questions: "any",
  query: "",
};

export const EMPTY_SOURCE_LIST_FILTERS: SourceListFilters = {
  linked: "any",
  pathPrefixes: [],
  query: "",
};

export function cardListFiltersActive(filters: CardListFilters): boolean {
  return filters.stages.length > 0 || filters.questions !== "any";
}

export function sourceListFiltersActive(filters: SourceListFilters): boolean {
  return filters.linked !== "any" || filters.pathPrefixes.length > 0;
}

export function sourcePathPrefix(path: string): string {
  const normalized = path.replace(/\\/g, "/").replace(/^\.\//, "");
  const parts = normalized.split("/").filter(Boolean);
  return parts[0] ?? (normalized || "/");
}

export function distinctSourcePathPrefixes(sources: readonly WorkspaceSource[]): string[] {
  const values = new Set<string>();
  for (const source of sources) {
    values.add(sourcePathPrefix(source.path));
  }
  return [...values].sort((a, b) => a.localeCompare(b));
}

export function filterWorkCards(
  cards: readonly WorkspaceWorkCard[],
  filters: CardListFilters,
): WorkspaceWorkCard[] {
  const needle = filters.query.trim().toLowerCase();
  return cards.filter((card) => {
    if (filters.stages.length > 0 && !filters.stages.includes(card.stage)) return false;
    if (filters.questions === "open" && cardOpenQuestions(card) === 0) return false;
    if (filters.questions === "clear" && cardOpenQuestions(card) > 0) return false;
    if (!needle) return true;
    return [card.title, card.slug, card.intent, card.owner, card.under].some((value) =>
      value.toLowerCase().includes(needle),
    );
  });
}

export function filterSources(
  sources: readonly WorkspaceSource[],
  filters: SourceListFilters,
): WorkspaceSource[] {
  const needle = filters.query.trim().toLowerCase();
  return sources.filter((source) => {
    if (filters.linked === "linked" && source.entityLinks.length === 0) return false;
    if (filters.linked === "unlinked" && source.entityLinks.length > 0) return false;
    if (
      filters.pathPrefixes.length > 0 &&
      !filters.pathPrefixes.includes(sourcePathPrefix(source.path))
    ) {
      return false;
    }
    if (!needle) return true;
    return [source.title, source.path, source.summary, source.owner].some((value) =>
      value.toLowerCase().includes(needle),
    );
  });
}
