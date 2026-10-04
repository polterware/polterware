import { createContext, useContext, type Accessor, type Component } from "solid-js";
import type { SparkyFeature, DetailSectionDef, SettingsPanelDef } from "./feature";

export type MemoryDetailRenderer = Component<{ memoryId: string }>;
export type SparkyShellContext = {
  features: readonly SparkyFeature[];
  path: Accessor<string>;
  params: Accessor<Record<string, string>>;
  searchParams: Accessor<URLSearchParams>;
  navigate: (path: string) => void;
  selectedMemoryId: Accessor<string | null>;
  openMemory: (id: string) => void;
  closeDetail: () => void;
  /** The memories route registers its editor. Returns a cleanup; the host owns Content/Header. */
  registerMemoryDetail: (component: MemoryDetailRenderer) => () => void;
  detailSections: Accessor<DetailSectionDef[]>;
  settingsPanels: Accessor<SettingsPanelDef[]>;
};
export const ShellContext = createContext<SparkyShellContext>();
export function useSparkyShell(): SparkyShellContext {
  const context = useContext(ShellContext);
  if (!context) throw new Error("Sparky features must render inside SparkyShell.");
  return context;
}
