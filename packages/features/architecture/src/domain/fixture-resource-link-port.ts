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

/** Local mock inventory — keeps architecture free of feature-resources package dep. */
const FIXTURE_ROWS: LinkableResource[] = [
  {
    id: "res-pg",
    name: "prod-postgres",
    type: "Database",
    environmentId: "production",
    environmentLabel: "Production",
    provider: "AWS",
    dataSource: "fixture",
  },
  {
    id: "res-pg-stg",
    name: "staging-postgres",
    type: "Database",
    environmentId: "staging",
    environmentLabel: "Staging",
    provider: "AWS",
    dataSource: "fixture",
  },
  {
    id: "res-payment",
    name: "payment-service",
    type: "Service",
    environmentId: "production",
    environmentLabel: "Production",
    provider: "AWS",
    dataSource: "fixture",
  },
  {
    id: "res-redis",
    name: "redis-cache",
    type: "Cache",
    environmentId: "production",
    environmentLabel: "Production",
    provider: "AWS",
    dataSource: "fixture",
  },
  {
    id: "res-queue",
    name: "events-queue",
    type: "Queue",
    environmentId: "production",
    environmentLabel: "Production",
    provider: "AWS",
    dataSource: "fixture",
  },
  {
    id: "res-api-dev",
    name: "dev-api",
    type: "Service",
    environmentId: "development",
    environmentLabel: "Development",
    provider: "Local",
    dataSource: "fixture",
  },
];

export function createFixtureResourceLinkPort(
  rows: readonly LinkableResource[] = FIXTURE_ROWS,
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
