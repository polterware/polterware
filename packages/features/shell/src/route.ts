import {
  capabilityEnabled,
  type PlatformCapability,
  type PlatformContext,
} from "@polterware/frontend-platform";

/** Project surfaces: Inbox, Atlas, Work, Decisions, Reality, Agents, plus Integrations and Settings. */
export type NavSectionId =
  | "inbox"
  | "atlas"
  | "work"
  | "decisions"
  | "reality"
  | "agents"
  | "integrations"
  | "settings";

/** Full pages reachable outside the sidebar (e.g. from the Help menu). */
export type PageSectionId = "cli";

export type SectionId = NavSectionId | PageSectionId;
export type ProjectSettingsTabId = "general" | "environments" | "shortcuts" | "advanced";
export type WorkTabId = "board" | "specs";
export type RealityTabId = "inventory" | "sources" | "facts" | "docs";
export type IntegrationsTabId = "connected" | "catalog";

export type DialogState =
  | { type: "resource-create" }
  | { type: "resource-edit"; entityId: string }
  | { type: "arch-node-create" }
  | { type: "integration-connect"; providerId?: string }
  | { type: "create-view" }
  | { type: "confirm"; message: string }
  | null;

export type ClientRoute = {
  section: string;
  tab: string;
  projectId?: string;
  environmentId?: string;
  search?: string;
  filters?: Readonly<Record<string, readonly string[]>>;
  dialog?: DialogState;
  selectedNodeId?: string | null;
  wizardStep?: number;
};

export type ShellStatus = {
  dataMode?: "fixture" | "offline" | "local";
};

export type NavItem = {
  id: NavSectionId;
  label: string;
  group?: string;
  icon?: string;
  requiredCapability?: PlatformCapability;
};

export const DEFAULT_TABS = {
  inbox: "all",
  atlas: "canvas",
  work: "board",
  decisions: "adrs",
  reality: "inventory",
  agents: "access",
  integrations: "connected",
  settings: "general",
  cli: "reference",
} satisfies Record<SectionId, string>;

export const SECTION_TABS = {
  // Route id only — Inbox is a flat page (no tab rail).
  inbox: [{ id: "all", label: "All" }],
  atlas: [
    { id: "canvas", label: "Canvas" },
    { id: "views", label: "Views" },
  ],
  work: [
    { id: "board", label: "Board" },
    { id: "specs", label: "Specs" },
  ],
  // Route id only — Decisions is a flat ADR list (no tab rail).
  decisions: [{ id: "adrs", label: "ADRs" }],
  reality: [
    { id: "inventory", label: "Inventory" },
    { id: "sources", label: "Sources" },
    { id: "facts", label: "Facts" },
    { id: "docs", label: "Docs" },
  ],
  agents: [
    { id: "access", label: "Access" },
    { id: "history", label: "History" },
    { id: "context", label: "Context" },
  ],
  integrations: [
    { id: "connected", label: "Connected" },
    { id: "catalog", label: "Catalog" },
  ],
  settings: [
    { id: "general", label: "General" },
    { id: "environments", label: "Environments" },
    { id: "shortcuts", label: "Shortcuts" },
    { id: "advanced", label: "Advanced" },
  ],
} satisfies Partial<Record<SectionId, { id: string; label: string }[]>>;

const NAV_SECTIONS = {
  inbox: true,
  atlas: true,
  work: true,
  decisions: true,
  reality: true,
  agents: true,
  integrations: true,
  settings: true,
} satisfies Record<string, boolean>;

const ROUTE_CAPABILITIES: Partial<Record<SectionId, PlatformCapability>> = {};

export function isNavSectionId(value: string): value is NavSectionId {
  return Object.prototype.hasOwnProperty.call(NAV_SECTIONS, value);
}

export function isSectionId(value: string): value is SectionId {
  return isNavSectionId(value) || value === "cli";
}

export type ResolvedRoute = { section: SectionId; tab: string };

export function requiredCapabilityForSection(
  section: string,
): PlatformCapability | undefined {
  return isSectionId(section) ? ROUTE_CAPABILITIES[section] : undefined;
}

export function sectionAvailable(
  section: string,
  platform?: PlatformContext,
): boolean {
  return (
    isSectionId(section) &&
    capabilityEnabled(platform, requiredCapabilityForSection(section))
  );
}

function normalizeTab(section: SectionId, tab?: string): string {
  const allowed = Object.entries(SECTION_TABS).find(([key]) => key === section)?.[1];
  if (!allowed) return DEFAULT_TABS[section];
  return allowed.some((item) => item.id === tab) ? tab! : DEFAULT_TABS[section];
}

export function resolveRoute(
  rawSection: string,
  rawTab?: string,
  platform?: PlatformContext,
): ResolvedRoute {
  let section = rawSection.toLowerCase();
  let tab = rawTab;

  // Removed surfaces (Ops / Terminal / Nodes) redirect to Atlas.
  if (
    section === "ops" ||
    section === "operations" ||
    section === "terminal" ||
    section === "search" ||
    section === "logs"
  ) {
    section = "atlas";
    tab = "canvas";
  }
  // Pre-Atlas sections map onto their new homes.
  if (section === "architecture") {
    section = "atlas";
    if (tab === "components") tab = "canvas";
  }
  if (
    section === "resources" ||
    section === "inventory" ||
    section === "nodes" ||
    section === "processes"
  ) {
    section = "reality";
    tab = "inventory";
  }
  if (section === "knowledge") {
    if (tab === "sources") {
      section = "reality";
    } else if (tab === "docs" || tab === "runbooks") {
      section = "reality";
      tab = "docs";
    } else if (tab === "library" || tab === "adrs") {
      section = "decisions";
      tab = "adrs";
    } else {
      section = "work";
      tab = "specs";
    }
  }
  if (section === "sync") {
    section = "settings";
    tab = "advanced";
  }
  if (section === "agents") {
    // Retired Agents Compile / Drift tabs → Context; Facts moved to Reality.
    if (tab === "compile" || tab === "drift") tab = "context";
    if (tab === "facts") section = "reality";
  }

  if (isSectionId(section) && sectionAvailable(section, platform)) {
    return { section, tab: normalizeTab(section, tab) };
  }
  return { section: "atlas", tab: "canvas" };
}

export function resolveSection(raw: string, platform?: PlatformContext): NavSectionId {
  const route = resolveRoute(raw, undefined, platform);
  return isNavSectionId(route.section) ? route.section : "atlas";
}

export function defaultTabFor(section: string, platform?: PlatformContext): string {
  return resolveRoute(section, undefined, platform).tab;
}

export function createDefaultRoute(
  section = "atlas",
  platform?: PlatformContext,
): ClientRoute {
  const resolved = resolveRoute(section, undefined, platform);
  return {
    ...resolved,
    projectId: "proj-polter",
    environmentId: "env-polter-prod",
    dialog: null,
  };
}

export function enterProject(
  route: ClientRoute,
  projectId: string,
  options?: { environmentId?: string; section?: NavSectionId; tab?: string },
): ClientRoute {
  const resolved = resolveRoute(options?.section ?? "atlas", options?.tab);
  return {
    ...route,
    ...resolved,
    projectId,
    environmentId: options?.environmentId ?? route.environmentId,
    dialog: null,
    selectedNodeId: null,
  };
}

export function goSection(
  route: ClientRoute,
  section: string,
  platform?: PlatformContext,
): ClientRoute {
  const resolved = resolveRoute(section, undefined, platform);
  return {
    ...route,
    ...resolved,
    dialog: null,
    selectedNodeId: null,
    search: undefined,
    filters: undefined,
  };
}

export function setTab(
  route: ClientRoute,
  tab: string,
  platform?: PlatformContext,
): ClientRoute {
  return {
    ...route,
    ...resolveRoute(route.section, tab, platform),
    dialog: null,
    selectedNodeId: null,
  };
}

export function ensureRouteAvailable(
  route: ClientRoute,
  platform?: PlatformContext,
): ClientRoute {
  return {
    ...route,
    ...resolveRoute(route.section, route.tab, platform),
  };
}

export function openDialog(route: ClientRoute, dialog: NonNullable<DialogState>): ClientRoute {
  return { ...route, dialog };
}

export function closeDialog(route: ClientRoute): ClientRoute {
  return { ...route, dialog: null };
}

export function setSearch(route: ClientRoute, search: string): ClientRoute {
  return { ...route, search };
}

export function setFilters(
  route: ClientRoute,
  filters: Readonly<Record<string, readonly string[]>>,
): ClientRoute {
  return { ...route, filters };
}

export function setEnvironment(route: ClientRoute, environmentId: string): ClientRoute {
  return { ...route, environmentId };
}

export function setSelectedNode(route: ClientRoute, selectedNodeId: string | null): ClientRoute {
  return { ...route, selectedNodeId };
}

export function setWizardStep(route: ClientRoute, wizardStep: number): ClientRoute {
  return { ...route, wizardStep };
}

export function historySnapshot(route: ClientRoute): ClientRoute {
  return { ...route, dialog: null };
}

export function routesEqual(a: ClientRoute, b: ClientRoute): boolean {
  return JSON.stringify(historySnapshot(a)) === JSON.stringify(historySnapshot(b));
}
