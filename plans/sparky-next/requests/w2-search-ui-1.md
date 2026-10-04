---
summary: Add a dynamic Commander result source so Search can own palette hits
rationale: CommandDef is static (id/label/keywords/run) and cannot return SearchService hits with safe highlighted snippets. W2-SEARCH-UI must not edit the shell.
---

W2-SEARCH-UI requests W2-SHELL:

`packages/features/sparky-shell/src/feature.ts` `CommandDef` is `{ id, label, keywords?, run }`. That cannot express a live SearchService result source.

The shell Commander (`commander.tsx`) already calls `bridge.search.search` itself and renders `hit.snippet` as plain text, so FTS5 `<b>` markers leak and the Search feature cannot supply:

- bounded result windows
- safe snippet highlighting (Solid text nodes, never `innerHTML`)
- Status/notice degraded copy
- opening a hit through `useSparkyShell().openMemory`

Until a source type exists, W2-SEARCH-UI registers the static command:

```ts
{ id: "search.open", label: "Search memories", keywords: ["find", "fts", "query", "search"], run }
```

which deep-links to `sparky://search`. The Search page owns filters, highlighting, keyboard, recents, and saved chips.

Requested registry addition (shape only; W2-SHELL owns the implementation):

```ts
export type CommandSourceDef = {
  id: string;
  search: (query: string) => Promise<{ hits: { id: string; label: string; snippet?: string; run: () => void }[] }>;
};
export type SparkyFeature = {
  // existing fields
  commandSources?: CommandSourceDef[];
};
```

Commander should query registered `commandSources` (debounced, stale-cancelled) instead of calling SearchService directly, and render `snippet` through a feature-supplied node or as plain text. Do not parse snippet HTML in the shell.

W2-SEARCH-UI does not edit `packages/features/sparky-shell/**`.
