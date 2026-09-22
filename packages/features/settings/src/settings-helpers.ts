import type { ProjectEnvironment, SettingsTabId } from "./types";

export const SETTINGS_TABS: ReadonlyArray<{ id: SettingsTabId; label: string }> = [
  { id: "general", label: "General" },
  { id: "environments", label: "Environments" },
  { id: "shortcuts", label: "Shortcuts" },
  { id: "agents", label: "Agents" },
  { id: "advanced", label: "Advanced" },
] as const;

/** Normalize route tab ids to the retained Settings IA. */
export function normalizeSettingsTab(raw?: string): SettingsTabId {
  const tab = raw ?? "general";
  if (
    tab === "general"
    || tab === "environments"
    || tab === "shortcuts"
    || tab === "agents"
    || tab === "advanced"
  ) {
    return tab;
  }
  return "general";
}

export function environmentSelectOptions(
  environments: readonly ProjectEnvironment[],
): Array<{ value: string; label: string }> {
  return environments.map((environment) => ({
    value: environment.id,
    label: environment.name,
  }));
}

export function environmentsDetailScope(): string {
  return "settings:environments";
}

export function environmentDetailTarget(entityId: string) {
  return {
    scopeId: environmentsDetailScope(),
    entityId,
    entityKind: "environment" as const,
  };
}
