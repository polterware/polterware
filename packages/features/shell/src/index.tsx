import type { JSX } from "solid-js";
import { createMemo, createSignal } from "solid-js";
import { ResizableSidebar, ToastRegion } from "@polterware/frontend-design-system";
import type { PlatformContext } from "@polterware/frontend-platform";
import type { ClientRoute, NavItem, ShellStatus } from "./route";
import { isNavSectionId } from "./route";
import { SidebarNav, SidebarSettingsItem } from "./sidebar";
import { MainSurface } from "./main-surface";
import { TitleBar } from "./title-bar";
import type { WorkspaceContext } from "./context-switchers";
import type { EnvironmentItem, ProjectSwitcherItem } from "./types";
import { ENVIRONMENT_FIXTURES } from "./fixtures";

export type {
  ClientRoute,
  DialogState,
  ShellStatus,
  NavItem,
  SectionId,
  NavSectionId,
  ProjectSettingsTabId,
  KnowledgeTabId,
  IntegrationsTabId,
} from "./route";
export type {
  ProjectSwitcherItem,
  EnvironmentItem,
  EnvironmentKind,
  TerminalTargetItem,
  TerminalTargetStatus,
  DataSource,
  DeploymentMode,
  PlatformCapability,
  PlatformCapabilities,
  PlatformContext,
} from "./types";
export {
  isTerminalTargetUsable,
  terminalTargetStatusTone,
  terminalTargetStatusLabel,
} from "./types";
export type { WorkspaceContext } from "./context-switchers";
export {
  createDefaultRoute,
  goSection,
  enterProject,
  setTab,
  openDialog,
  closeDialog,
  setSearch,
  setFilters,
  setEnvironment,
  setSelectedNode,
  setWizardStep,
  routesEqual,
  historySnapshot,
  resolveSection,
  resolveRoute,
  defaultTabFor,
  DEFAULT_TABS,
  SECTION_TABS,
  isSectionId,
  isNavSectionId,
  ensureRouteAvailable,
  requiredCapabilityForSection,
  sectionAvailable,
} from "./route";
export { MainSurface, PageHeader, PageContent } from "./main-surface";
export { SidebarNav, SidebarSettingsItem } from "./sidebar";
export { TitleBar } from "./title-bar";
export { TerminalEmulator } from "./terminal-emulator";
export { TerminalTranscript } from "./terminal-transcript";
export { CreateProjectDialog } from "./create-project-dialog";
export { ProjectSwitcher } from "./project-switcher";
export { WorkspaceContextBar } from "./context-switchers";
export {
  PROJECT_SWITCHER_FIXTURES,
  ENVIRONMENT_FIXTURES,
} from "./fixtures";
export type * from "./workspace-mocks";
export {
  INTEGRATION_MOCKS,
  INTEGRATION_RESOURCE_MOCKS,
  INTEGRATION_SYNC_RUN_MOCKS,
  CATALOG_PROVIDER_MOCKS,
  DOCUMENT_MOCKS,
  RUNBOOK_MOCKS,
  SPEC_FEATURE_MOCKS,
  CONSTITUTION_MOCK,
} from "./workspace-mocks";

export const DEFAULT_NAV: NavItem[] = [
  { id: "architecture", label: "Architecture", group: "SYSTEM" },
  { id: "resources", label: "Resources", group: "SYSTEM" },
  { id: "knowledge", label: "Knowledge", group: "SYSTEM" },
  { id: "agents", label: "Agents", group: "MANAGEMENT" },
  { id: "integrations", label: "Integrations", group: "MANAGEMENT" },
];

function workspaceFromRoute(
  route: ClientRoute | undefined,
  environments: readonly EnvironmentItem[],
): WorkspaceContext {
  const projectId = route?.projectId ?? environments[0]?.projectId ?? "proj-polter";
  const environment =
    environments.find((item) => item.id === route?.environmentId) ??
    environments.find((item) => item.projectId === projectId) ??
    environments[0];
  return { projectId, environmentId: environment?.id ?? "" };
}

export function Shell(props: {
  panel?: () => string;
  setPanel?: (panel: string) => void;
  route?: () => ClientRoute;
  onNavigate?: (section: string) => void;
  onProjectSelect?: (projectId: string) => void;
  onCreateProject?: () => void;
  onSelectEnvironment?: (environmentId: string) => void;
  onOpenCommander?: () => void;
  onOpenSettings?: () => void;
  workspace?: WorkspaceContext;
  projects?: ProjectSwitcherItem[];
  environments?: EnvironmentItem[];
  status?: ShellStatus;
  navItems?: NavItem[];
  platform?: PlatformContext;
  canGoBack?: boolean;
  canGoForward?: boolean;
  onBack?: () => void;
  onForward?: () => void;
  children: JSX.Element;
}) {
  const [localWorkspace, setLocalWorkspace] = createSignal<WorkspaceContext | null>(null);

  const currentSection = () => props.route?.().section ?? props.panel?.() ?? "architecture";
  const items = () => props.navItems ?? DEFAULT_NAV;
  const workspace = createMemo(
    () => props.workspace ?? localWorkspace() ?? workspaceFromRoute(props.route?.(), props.environments ?? ENVIRONMENT_FIXTURES),
  );
  const activeId = () => (isNavSectionId(currentSection()) ? currentSection() : "");

  function select(section: string) {
    props.onNavigate?.(section);
    props.setPanel?.(section);
  }

  function selectProject(projectId: string) {
    const environments = props.environments ?? ENVIRONMENT_FIXTURES;
    const firstEnvironment =
      environments.find((environment) => environment.projectId === projectId) ?? environments[0];
    if (firstEnvironment) {
      setLocalWorkspace({ projectId, environmentId: firstEnvironment.id });
      props.onSelectEnvironment?.(firstEnvironment.id);
    }
    props.onProjectSelect?.(projectId);
  }

  function selectEnvironment(environmentId: string) {
    setLocalWorkspace({ ...workspace(), environmentId });
    props.onSelectEnvironment?.(environmentId);
  }

  return (
    <div class="pw-shell-root flex h-full flex-col text-[var(--color-text-primary)]">
      <TitleBar
        canGoBack={Boolean(props.canGoBack)}
        canGoForward={Boolean(props.canGoForward)}
        onBack={() => props.onBack?.()}
        onForward={() => props.onForward?.()}
        workspace={workspace()}
        projects={props.projects}
        environments={props.environments}
        onSelectProject={selectProject}
        onSelectEnvironment={selectEnvironment}
        onCreateProject={() => props.onCreateProject?.()}
        onOpenCommander={() => props.onOpenCommander?.()}
      />

      <div class="flex min-h-0 flex-1">
        <ResizableSidebar
          class="pw-shell-sidebar"
          defaultWidth={232}
          minWidth={184}
          maxWidth={320}
          collapseThreshold={112}
          storageKey="polterware.shell.sidebar.sizes"
          ariaLabel="Project navigation"
          sidebar={
            <div class="flex h-full min-h-0 flex-col pt-2">
              <div class="flex min-h-0 flex-1 flex-col">
                <SidebarNav
                  items={items()}
                  activeId={activeId()}
                  onSelect={select}
                  platform={props.platform}
                />
              </div>
              <div class="mt-auto px-2 py-3">
                <SidebarSettingsItem
                  active={currentSection() === "settings"}
                  onSelect={() => props.onOpenSettings?.() ?? select("settings")}
                />
              </div>
            </div>
          }
        >
          <div class="pw-shell-main-frame flex h-full min-w-0 flex-1 flex-col">
            <MainSurface>{props.children}</MainSurface>
          </div>
        </ResizableSidebar>
      </div>

      <ToastRegion />
    </div>
  );
}
