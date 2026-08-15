import type { ProjectEnvironment } from "./project-environments";
import {
  DEFAULT_PROJECT_ENVIRONMENTS,
  normalizeEnvironmentId,
} from "./project-environments";

/** @deprecated Prefer ProjectEnvironmentPort — kept as fixture alias for defaults. */
export const PLACE_ENVIRONMENTS = DEFAULT_PROJECT_ENVIRONMENTS;

export type PlaceEnvironmentId = string;

/** Optional operational role at place time (mock; stored on node metadata). */
export const PLACE_ROLES = [
  { id: "", label: "—" },
  { id: "primary", label: "Primary" },
  { id: "replica", label: "Replica" },
  { id: "edge", label: "Edge" },
  { id: "internal", label: "Internal" },
  { id: "cache", label: "Cache" },
  { id: "stream", label: "Stream" },
  { id: "other", label: "Other" },
] as const;

export type PlaceRoleId = (typeof PLACE_ROLES)[number]["id"];

export type PlaceConfig = {
  name: string;
  environmentId: string;
  /** Instance purpose on this system map — not the catalog type blurb. */
  description?: string;
  role?: string;
  owner?: string;
  /** Optional inventory resource link at place time. */
  resourceId?: string;
};

export type PlaceConfigInput = {
  name: string;
  environmentId?: string;
  /** @deprecated use environmentId */
  environment?: string;
  description?: string;
  role?: string;
  owner?: string;
  resourceId?: string;
};

function optionalTrim(value: string | undefined): string | undefined {
  const t = value?.trim();
  return t ? t : undefined;
}

export function normalizePlaceConfig(
  input: PlaceConfigInput,
  envs: readonly ProjectEnvironment[] = DEFAULT_PROJECT_ENVIRONMENTS,
): PlaceConfig | { error: string } {
  const name = input.name.trim();
  if (!name) return { error: "Name is required" };
  if (envs.length === 0) return { error: "Add a project environment first" };

  const rawEnv = input.environmentId ?? input.environment ?? "";
  const envId = normalizeEnvironmentId(rawEnv);
  const env = envs.find(
    (e) => e.id === envId || normalizeEnvironmentId(e.id) === envId || normalizeEnvironmentId(e.label) === envId,
  );
  if (!env) return { error: "Environment is required" };

  const roleRaw = optionalTrim(input.role);
  if (roleRaw) {
    const known = PLACE_ROLES.some((r) => r.id === roleRaw && r.id !== "");
    if (!known) return { error: "Role is invalid" };
  }

  const config: PlaceConfig = {
    name,
    environmentId: env.id,
  };
  const description = optionalTrim(input.description);
  if (description) config.description = description;
  if (roleRaw) config.role = roleRaw;
  const owner = optionalTrim(input.owner);
  if (owner) config.owner = owner;
  const resourceId = optionalTrim(input.resourceId);
  if (resourceId) config.resourceId = resourceId;
  return config;
}

export function environmentIdsFromConfig(config: PlaceConfig): string[] {
  return [config.environmentId];
}

/** Metadata patch for role (and future place-time keys). */
export function metadataFromPlaceConfig(
  config: PlaceConfig,
): Record<string, string | number | boolean> | undefined {
  if (!config.role) return undefined;
  return { role: config.role };
}

/** Build docAddNode partial lifecycle fields from place config. */
export function lifecycleFromPlaceConfig(config: PlaceConfig): {
  state: "planned" | "linked";
  linkStatus: "unlinked" | "linked";
  resourceId: string | null;
} {
  if (config.resourceId) {
    return { state: "linked", linkStatus: "linked", resourceId: config.resourceId };
  }
  return { state: "planned", linkStatus: "unlinked", resourceId: null };
}

export function roleLabel(roleId: string | undefined): string | undefined {
  if (!roleId) return undefined;
  return PLACE_ROLES.find((r) => r.id === roleId)?.label ?? roleId;
}
