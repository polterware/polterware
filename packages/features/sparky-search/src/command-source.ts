import type { CommandSourceDef } from "@polterware/feature-sparky-shell/feature";
import { createSearchStore, type SearchStore } from "./store";

/** Dedicated store instance keeps Commander queries separate from page filters. */
export function createSearchCommandSource(store: SearchStore = createSearchStore()): CommandSourceDef {
  return {
    id: "search.memories",
    async search(query) {
      const hits = await store.commandQuery(query);
      return { hits: hits.filter(hit => hit.memoryId).map(hit => ({
        id: hit.memoryId!,
        label: hit.title || "Untitled memory",
        snippet: hit.snippet,
        run: () => { void store.openHit(hit.memoryId!); },
      })) };
    },
  };
}
