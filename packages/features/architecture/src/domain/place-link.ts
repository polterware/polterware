import { normalizeEnvironmentId } from "./project-environments";

export type LinkableResource = {
  id: string;
  name: string;
  type: string;
  environmentId: string;
  environmentLabel?: string;
  provider?: string;
  dataSource: "fixture" | "core";
};

export type ListLinkCandidatesQuery = {
  environmentId?: string;
  componentType?: string;
  subtype?: string;
  query?: string;
  typeFilterStrict?: boolean;
};

/** Map architecture semantic types / categories to inventory type strings. */
const TYPE_FAMILY = {
  database: ["database", "db", "rds", "postgres", "postgresql", "mysql", "mongo", "mongodb"],
  cache: ["cache", "redis", "memcached"],
  queue: ["queue", "sqs", "rabbitmq", "broker"],
  stream: ["stream", "kafka", "kinesis", "pubsub"],
  storage: ["storage", "bucket", "s3", "blob", "object"],
  api: ["api", "service", "gateway"],
  service: ["service", "api", "worker"],
  worker: ["service", "worker"],
  web: ["service", "web", "app"],
  host: ["host", "vps", "node", "server", "ec2"],
  function: ["function", "lambda", "serverless"],
} satisfies Record<string, string[]>;

function resourceTypeTokens(type: string): string[] {
  return type
    .toLowerCase()
    .split(/[\s/_-]+/)
    .filter(Boolean);
}

export function typesCompatible(componentType: string | undefined, resourceType: string): boolean {
  if (!componentType) return true;
  const key = componentType.toLowerCase();
  const family = Object.entries(TYPE_FAMILY).find(([type]) => type === key)?.[1] ?? [key];
  const tokens = resourceTypeTokens(resourceType);
  return family.some((f) => tokens.some((t) => t.includes(f) || f.includes(t)));
}

export function filterLinkCandidates(
  all: readonly LinkableResource[],
  q: ListLinkCandidatesQuery,
): LinkableResource[] {
  const env = q.environmentId ? normalizeEnvironmentId(q.environmentId) : "";
  const query = q.query?.trim().toLowerCase() ?? "";
  const strict = q.typeFilterStrict !== false;

  let list = all.filter((r) => {
    if (env && normalizeEnvironmentId(r.environmentId) !== env) return false;
    if (query && !r.name.toLowerCase().includes(query)) return false;
    return true;
  });

  if (strict && q.componentType) {
    const matched = list.filter((r) => typesCompatible(q.componentType, r.type));
    if (matched.length > 0) list = matched;
  }

  return list;
}

export function validateLink(
  componentEnvId: string,
  resource: LinkableResource | undefined | null,
) {
  if (!resource) return { error: "Resource unavailable" };
  const cEnv = normalizeEnvironmentId(componentEnvId);
  const rEnv = normalizeEnvironmentId(resource.environmentId);
  if (cEnv && rEnv && cEnv !== rEnv) {
    return { error: "Resource environment does not match" };
  }
  return { resourceId: resource.id };
}

/** Suggest place fields when starting from a resource (Connected flow). */
export function suggestPlaceFromResource(resource: LinkableResource) {
  const envId = normalizeEnvironmentId(resource.environmentId) || resource.environmentId;
  const tokens = resourceTypeTokens(resource.type);
  let suggested: string | undefined;
  for (const [comp, family] of Object.entries(TYPE_FAMILY)) {
    if (family.some((f) => tokens.some((t) => t.includes(f) || f.includes(t)))) {
      suggested = comp;
      break;
    }
  }
  return {
    name: resource.name,
    environmentId: envId,
    resourceId: resource.id,
    suggestedComponentType: suggested,
  };
}
