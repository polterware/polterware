import type { Component } from "solid-js";

export type RouteDef = {
  path: string;
  component: Component;
  title: string;
};

export type SidebarItemDef = {
  id: string;
  label: string;
  path: string;
  icon?: string;
  badge?: number;
};

export type CommandDef = {
  id: string;
  label: string;
  keywords?: string[];
  run: () => void | Promise<void>;
};

export type CommandSourceDef = {
  id: string;
  search: (query: string) => Promise<{ hits: { id: string; label: string; snippet?: string; run: () => void }[] }>;
};

export type SettingsPanelDef = {
  id: string;
  label: string;
  group: string;
  component: Component;
};

export type DetailSectionDef = {
  id: string;
  order: number;
  component: Component;
  visible?: (memoryId: string) => boolean;
};

export type MemoryDetailProps = {
  memoryId: string;
  detailSections: DetailSectionDef[];
};

export type SidebarTreeDef = {
  id: string;
  component: Component;
};

export type SparkyFeature = {
  id: string;
  routes: RouteDef[];
  sidebar?: SidebarItemDef[];
  commands?: CommandDef[];
  commandSources?: CommandSourceDef[];
  settingsPanels?: SettingsPanelDef[];
  detailSections?: DetailSectionDef[];
  memoryDetail?: Component<MemoryDetailProps>;
  memoryCollection?: Component<{ mindId: string }>;
  sidebarTree?: SidebarTreeDef;
  onDeepLink?: (url: URL) => boolean;
};

// Features access host services through this registry entry point only.
export { useSparkyShell } from "./context";
export type { SparkyShellContext, MemoryDetailRenderer } from "./context";
export type { MemoryShortcut } from "./shortcuts";
