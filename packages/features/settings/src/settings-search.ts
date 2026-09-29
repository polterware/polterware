import { SHORTCUT_GROUP_LABELS, shortcutsInGroup } from "@polterware/frontend-state";
import type { ProjectEnvironment, SettingsTabId } from "./types";

export type SettingsSearchEntry = {
  id: string;
  tab: SettingsTabId;
  /** Section title shown in the result path; also the scroll target slug. */
  section: string;
  label: string;
  description?: string;
  keywords?: string;
};

export function settingsSectionId(section: string): string {
  return `settings-section-${section.toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-|-$/g, "")}`;
}

export const SETTINGS_SEARCH_ENTRIES: SettingsSearchEntry[] = [
  { id: "project-name", tab: "general", section: "Project", label: "Name" },
  { id: "project-description", tab: "general", section: "Project", label: "Description" },
  { id: "project-repository", tab: "general", section: "Project", label: "Primary repository", keywords: "git org repo" },
  { id: "project-default-env", tab: "general", section: "Project", label: "Default environment" },
  { id: "project-id", tab: "general", section: "Project", label: "Project ID", keywords: "copy identifier" },
  {
    id: "local-repositories",
    tab: "general",
    section: "Local repositories",
    label: "Local repositories",
    description: "Extra Git checkouts read on scan.",
    keywords: "git checkout path browse",
  },
  {
    id: "privacy",
    tab: "general",
    section: "Privacy",
    label: "Privacy",
    description: "Local-first data and credential handling.",
    keywords: "credentials tokens account",
  },
  {
    id: "reduce-transparency",
    tab: "advanced",
    section: "Appearance",
    label: "Reduce transparency",
    description: "Use a solid fill instead of the macOS frosted title bar and sidebar.",
    keywords: "appearance glass vibrancy theme",
  },
  {
    id: "core-status",
    tab: "advanced",
    section: "Local core",
    label: "Local core",
    description: "Status, start on open, and quit behavior.",
    keywords: "server serve process quit start",
  },
  {
    id: "archive-project",
    tab: "advanced",
    section: "Danger zone",
    label: "Archive project",
    description: "Hide this project from the switcher.",
  },
  {
    id: "reset-store",
    tab: "advanced",
    section: "Danger zone",
    label: "Reset local store",
    description: "Delete local data and reseed the demonstration project.",
    keywords: "database sqlite",
  },
];

/** Entries that depend on live data: environments and shortcut commands. */
export function dynamicSettingsEntries(environments: readonly ProjectEnvironment[]): SettingsSearchEntry[] {
  const shortcuts = (["shell", "architecture"] as const).flatMap((group) =>
    shortcutsInGroup(group).map<SettingsSearchEntry>((command) => ({
      id: `shortcut-${command.id}`,
      tab: "shortcuts",
      section: SHORTCUT_GROUP_LABELS[group],
      label: command.label,
      keywords: "keyboard chord",
    })),
  );
  const envs = environments.map<SettingsSearchEntry>((env) => ({
    id: `environment-${env.id}`,
    tab: "environments",
    section: "Environments",
    label: env.name,
    description: env.description,
    keywords: `${env.kind} ${env.protection}`,
  }));
  return [...envs, ...shortcuts];
}

/** Case-insensitive substring match over label, description, section, and keywords. */
export function searchSettings(
  entries: readonly SettingsSearchEntry[],
  query: string,
): SettingsSearchEntry[] {
  const terms = query.trim().toLowerCase().split(/\s+/).filter(Boolean);
  if (terms.length === 0) return [];
  return entries.filter((entry) => {
    const haystack = [entry.label, entry.description, entry.section, entry.keywords]
      .filter(Boolean)
      .join(" ")
      .toLowerCase();
    return terms.every((term) => haystack.includes(term));
  });
}
