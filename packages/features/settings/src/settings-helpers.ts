import {
  FileText,
  GitBranch,
  Hammer,
  Zap,
  type TabItem,
} from "@polterware/frontend-design-system";
import type { ProjectEnvironment, SettingsTabId } from "./types";

export const SETTINGS_TABS: TabItem[] = [
  {
    id: "general",
    label: "General",
    tooltip: {
      title: "Project profile",
      description:
        "Name, description, and basic project identity stored locally. Changes apply to this project only.",
      icon: FileText,
    },
  },
  {
    id: "environments",
    label: "Environments",
    tooltip: {
      title: "Environments",
      description:
        "Development, Staging, Production, or custom. Switch context in the title bar; each environment is local SQLite data, not a hosted workspace.",
      icon: GitBranch,
    },
  },
  {
    id: "shortcuts",
    label: "Shortcuts",
    tooltip: {
      title: "Keyboard shortcuts",
      description:
        "Remap Shell and Architecture chords for this computer. Architecture shortcuts work when the canvas is focused and you are not typing in a field.",
      icon: Zap,
    },
  },
  {
    id: "advanced",
    label: "Advanced",
    tooltip: {
      title: "Danger zone",
      description:
        "Archive the project or reset the local demonstration store. Destructive and local-only — use when you intentionally want a clean slate.",
      icon: Hammer,
    },
  },
];

/** Normalize route tab ids to the retained Settings IA. */
export function normalizeSettingsTab(raw?: string): SettingsTabId {
  const tab = raw ?? "general";
  if (
    tab === "general"
    || tab === "environments"
    || tab === "shortcuts"
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
