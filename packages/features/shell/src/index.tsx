import type { JSX } from "solid-js";
import { Show, createMemo, createSignal } from "solid-js";
import { ResizableSidebar, ToastRegion, pushToast } from "@polterware/frontend-design-system";
import type { PlatformContext } from "@polterware/frontend-platform";
import type { ClientRoute, NavItem, ShellStatus } from "./route";
import { isNavSectionId, shellLayoutMode } from "./route";
import { SidebarNav, SidebarSettingsItem } from "./sidebar";
import { MainSurface } from "./main-surface";
import { TitleBar } from "./title-bar";
import { TargetPicker } from "./target-picker";
import { TerminalSidePanel } from "./terminal-side-panel";
import { unusableTerminalTargetFeedback } from "./terminal-session-model";
import type { LiveTerminalSession } from "./live-terminal-session";
import type { BreadcrumbLevel, WorkspaceContext } from "./context-switchers";
import type { EnvironmentItem, ProjectSwitcherItem, TerminalTargetItem } from "./types";
import { ENVIRONMENT_FIXTURES } from "./fixtures";

export type { LiveTerminalSession } from "./live-terminal-session";
export type {
  ClientRoute,
  DialogState,
  DrawerState,
  ShellStatus,
  NavItem,
  SectionId,
  NavSectionId,
  ChromeSectionId,
  ShellLayoutMode,
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
export type { WorkspaceContext, BreadcrumbLevel } from "./context-switchers";
export {
  createDefaultRoute,
  goSection,
  goCreateProject,
  enterProject,
  shellLayoutMode,
  isNeutralSection,
  setTab,
  openDialog,
  closeDialog,
  openDrawer,
  closeDrawer,
  setSearch,
  setFilters,
  setEnvironment,
  setSelectedNode,
  setWizardStep,
  setSessionId,
  routesEqual,
  historySnapshot,
  resolveSection,
  resolveRoute,
  defaultTabFor,
  DEFAULT_TABS,
  SECTION_TABS,
  isSectionId,
  isNavSectionId,
  isChromeSectionId,
  ensureRouteAvailable,
  requiredCapabilityForSection,
  sectionAvailable,
} from "./route";
export { MainSurface, PageHeader, PageContent } from "./main-surface";
export { SidebarNav, SidebarSettingsItem } from "./sidebar";
export { TitleBar } from "./title-bar";
export { TerminalPage, LOCAL_TERMINAL_TARGET } from "./terminal-page";
export { TerminalEmulator } from "./terminal-emulator";
export { TerminalTranscript } from "./terminal-transcript";
export { CreateProjectPage } from "./create-project";
export { ProjectSwitcher } from "./project-switcher";
export { WorkspaceContextBar } from "./context-switchers";
export { TargetPicker } from "./target-picker";
export { TerminalSidePanel } from "./terminal-side-panel";
export {
  terminalSessionStatusTone,
  terminalSessionStatusLabel,
  terminalSessionDataSourceLabel,
  unusableTerminalTargetFeedback,
  canOpenTerminalSessionOnTarget,
  createMockTerminalSession,
  closeMockTerminalSession,
  removeTerminalSession,
  formatTerminalStartedAt,
  findSessionForTarget,
  buildSessionCorrelationId,
} from "./terminal-session-model";
export {
  PROJECT_SWITCHER_FIXTURES,
  ENVIRONMENT_FIXTURES,
  TERMINAL_TARGET_FIXTURES,
  TERMINAL_OUTPUT_FIXTURES,
} from "./fixtures";
export type * from "./workspace-mocks";
export {
  TERMINAL_SESSION_MOCKS,
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
  { id: "operations", label: "Operations", group: "EXECUTION" },
  { id: "terminal", label: "Terminal", group: "EXECUTION" },
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

function breadcrumbLevel(section: string): BreadcrumbLevel {
  return section === "create-project" ? "create-project" : "project";
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
  onExpandTerminal?: (payload: { targetId: string; session: LiveTerminalSession }) => void;
  onOpenSettings?: () => void;
  workspace?: WorkspaceContext;
  projects?: ProjectSwitcherItem[];
  environments?: EnvironmentItem[];
  terminalTargets?: TerminalTargetItem[];
  projectId?: string;
  onCreateTerminalSession?: (input: {
    projectId: string;
    targetId: string;
    cols: number;
    rows: number;
  }) => Promise<LiveTerminalSession>;
  onWriteTerminalSession?: (id: string, data: string) => Promise<void>;
  onResizeTerminalSession?: (id: string, cols: number, rows: number) => Promise<void>;
  onCloseTerminalSession?: (id: string) => Promise<void>;
  onReadTerminalSession?: (id: string, offset: number) => Promise<{
    data: string;
    nextOffset: number;
    closed: boolean;
  }>;
  status?: ShellStatus;
  navItems?: NavItem[];
  platform?: PlatformContext;
  canGoBack?: boolean;
  canGoForward?: boolean;
  onBack?: () => void;
  onForward?: () => void;
  children: JSX.Element;
}) {
  const [targetPickerOpen, setTargetPickerOpen] = createSignal(false);
  const [terminalTargetId, setTerminalTargetId] = createSignal<string | null>(null);
  const [localWorkspace, setLocalWorkspace] = createSignal<WorkspaceContext | null>(null);
  let terminalButton: HTMLButtonElement | undefined;

  const currentSection = () => props.route?.().section ?? props.panel?.() ?? "architecture";
  const mode = () => shellLayoutMode(currentSection());
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
    <div class="flex h-full flex-col bg-[var(--color-app-background)] text-[var(--color-text-primary)]">
      <TitleBar
        canGoBack={Boolean(props.canGoBack)}
        canGoForward={Boolean(props.canGoForward)}
        onBack={() => props.onBack?.()}
        onForward={() => props.onForward?.()}
        breadcrumbLevel={breadcrumbLevel(currentSection())}
        workspace={workspace()}
        projects={props.projects}
        environments={props.environments}
        onSelectProject={selectProject}
        onSelectEnvironment={selectEnvironment}
        onCreateProject={() => props.onCreateProject?.()}
        onOpenCommander={() => props.onOpenCommander?.()}
        onOpenTerminal={() => setTargetPickerOpen((open) => !open)}
        terminalPickerOpen={targetPickerOpen()}
        terminalButtonRef={(element) => {
          terminalButton = element;
        }}
      />

      <div class="flex min-h-0 flex-1">
        <Show
          when={mode() === "project"}
          fallback={
            <div class="flex min-w-0 flex-1 flex-col px-2 pb-2">
              <MainSurface>{props.children}</MainSurface>
            </div>
          }
        >
          <ResizableSidebar
            class="bg-[var(--color-sidebar-background)]"
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
        </Show>
      </div>

      <TargetPicker
        open={targetPickerOpen()}
        onClose={() => setTargetPickerOpen(false)}
        targets={props.terminalTargets ?? []}
        environmentId={workspace().environmentId}
        onSelect={(targetId) => {
          setTerminalTargetId(targetId);
          setTargetPickerOpen(false);
        }}
        onUnusableSelect={(target) => pushToast(unusableTerminalTargetFeedback(target), "danger")}
        anchorRef={() => terminalButton}
      />
      <TerminalSidePanel
        open={Boolean(terminalTargetId())}
        targetId={terminalTargetId()}
        targets={props.terminalTargets ?? []}
        projectId={props.projectId}
        onCreateSession={props.onCreateTerminalSession}
        onWriteSession={props.onWriteTerminalSession}
        onResizeSession={props.onResizeTerminalSession}
        onCloseSession={props.onCloseTerminalSession}
        onReadSession={props.onReadTerminalSession}
        onClose={() => setTerminalTargetId(null)}
        onExpand={(payload) => {
          setTerminalTargetId(null);
          props.onExpandTerminal?.(payload);
        }}
      />
      <ToastRegion />
    </div>
  );
}
