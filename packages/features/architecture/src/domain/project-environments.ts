/** Project-scoped environment used while placing a component. */
export type ProjectEnvironment = {
  id: string;
  label: string;
  kind?: string;
  protected?: boolean;
};

export interface ProjectEnvironmentPort {
  listEnvironments(projectId?: string): ProjectEnvironment[];
}

/** Fixture default list when shell/core env RPC is not wired. */
export const DEFAULT_PROJECT_ENVIRONMENTS: readonly ProjectEnvironment[] = [
  { id: "production", label: "Production", kind: "production", protected: true },
  { id: "staging", label: "Staging", kind: "staging" },
  { id: "development", label: "Development", kind: "development" },
  { id: "local", label: "Local", kind: "local" },
] as const;

export function createDefaultProjectEnvironmentPort(): ProjectEnvironmentPort {
  return {
    listEnvironments: () => [...DEFAULT_PROJECT_ENVIRONMENTS],
  };
}

/** Normalize display labels and loose ids to stable lowercase ids. */
export function normalizeEnvironmentId(raw: string | undefined | null): string {
  if (!raw) return "";
  const t = raw.trim().toLowerCase().replace(/\s+/g, "-");
  const aliases = {
    prod: "production",
    production: "production",
    stage: "staging",
    staging: "staging",
    dev: "development",
    development: "development",
    local: "local",
  } satisfies Record<string, string>;
  return Object.entries(aliases).find(([key]) => key === t)?.[1] ?? t;
}

export function environmentLabel(
  envs: readonly ProjectEnvironment[],
  id: string | undefined,
): string {
  if (!id) return "—";
  const normalized = normalizeEnvironmentId(id);
  const found = envs.find((e) => e.id === normalized || normalizeEnvironmentId(e.id) === normalized);
  return found?.label ?? id;
}

export function defaultEnvironmentId(envs: readonly ProjectEnvironment[]): string | undefined {
  if (envs.length === 0) return undefined;
  const prod = envs.find((e) => e.id === "production" || e.kind === "production");
  return prod?.id ?? envs[0]?.id ?? "";
}
