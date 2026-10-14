import { PageHeader, PageEmpty, StatusChip } from "@polterware/frontend-design-system";
import type { SparkyFeature } from "./feature";
import { SHELL_ROUTES } from "./route";
const feature: SparkyFeature = {
  id: "shell",
  routes: [...SHELL_ROUTES, { path: "/space/:id", title: "Space" }].map(route => ({
    path: route.path, title: route.title,
    component: () => <><PageHeader title={route.title} subtitle={<StatusChip tone="muted" label="Unavailable" />} />
      <PageEmpty title={`${route.title} is unavailable`} description="This feature has not registered a route yet." /></>,
  })),
};
export default feature;
export { SparkyShell } from "./shell";
export { useSparkyShell } from "./context";
export { parseQuickAdd } from "./quick-add";
export { createShellBridge } from "./bridge";
export type { SparkyFeature, RouteDef, SidebarItemDef, CommandDef, CommandSourceDef, SettingsPanelDef, DetailSectionDef, NoteDetailProps, SidebarTreeDef } from "./feature";
export type { SparkyShellContext, NoteDetailRenderer } from "./context";
export type { ShellBridge, ShellWireBridge } from "./bridge";
export type { NoteShortcut } from "./shortcuts";
