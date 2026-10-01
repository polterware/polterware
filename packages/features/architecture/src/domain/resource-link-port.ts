import {
  filterLinkCandidates,
  type LinkableResource,
  type ListLinkCandidatesQuery,
} from "./place-link";
import { normalizeEnvironmentId } from "./project-environments";

export type ResourceLinkPort = {
  listCandidates: (q: ListLinkCandidatesQuery) => LinkableResource[];
  getById: (id: string) => LinkableResource | undefined;
};

/** Link candidates are the provider resources synced by Integrations. */
export function createResourceLinkPort(
  rows: readonly LinkableResource[],
): ResourceLinkPort {
  const all = rows.map((r) => ({
    ...r,
    environmentId: normalizeEnvironmentId(r.environmentId) || r.environmentId,
  }));
  const byId = new Map(all.map((r) => [r.id, r]));
  return {
    listCandidates: (q) => filterLinkCandidates(all, q),
    getById: (id) => byId.get(id),
  };
}
