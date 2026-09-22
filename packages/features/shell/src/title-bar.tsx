import { ChevronLeft, ChevronRight, Search, Terminal } from "@polterware/frontend-design-system";
import { createShortcutLabel } from "@polterware/frontend-state";
import {
  WorkspaceContextBar,
  type BreadcrumbLevel,
  type WorkspaceContext,
} from "./context-switchers";
import type { EnvironmentItem, ProjectSwitcherItem } from "./types";

export type TitleBarProps = {
  canGoBack: boolean;
  canGoForward: boolean;
  onBack: () => void;
  onForward: () => void;
  breadcrumbLevel?: BreadcrumbLevel;
  workspace: WorkspaceContext;
  projects?: ProjectSwitcherItem[];
  environments?: EnvironmentItem[];
  onSelectProject?: (projectId: string) => void;
  onSelectEnvironment?: (environmentId: string) => void;
  onCreateProject?: () => void;
  onOpenCommander?: () => void;
  onOpenTerminal?: () => void;
  terminalPickerOpen?: boolean;
  terminalButtonRef?: (element: HTMLButtonElement | undefined) => void;
};

export function TitleBar(props: TitleBarProps) {
  const commanderShortcut = createShortcutLabel("shell.toggle_commander");
  return (
    <header
      class="window-title-bar"
      role="banner"
      style={{
        display: "flex",
        "align-items": "center",
        height: "var(--titlebar-height, 44px)",
        "flex-shrink": "0",
        "padding-left": "var(--titlebar-traffic-inset, 78px)",
        "padding-right": "8px",
        gap: "6px",
        background: "var(--color-app-background)",
        "-webkit-app-region": "drag",
        "user-select": "none",
      }}
    >
      <div class="flex shrink-0 items-center [-webkit-app-region:no-drag]">
        <button
          type="button"
          aria-label="Go back"
          disabled={!props.canGoBack}
          onClick={props.onBack}
          style={navButtonStyle(!props.canGoBack)}
        >
          <ChevronLeft size={13} />
        </button>
        <button
          type="button"
          aria-label="Go forward"
          disabled={!props.canGoForward}
          onClick={props.onForward}
          style={navButtonStyle(!props.canGoForward)}
        >
          <ChevronRight size={13} />
        </button>
      </div>

      <div class="flex min-w-0 shrink items-center [-webkit-app-region:no-drag]">
        <WorkspaceContextBar
          context={props.workspace}
          level={props.breadcrumbLevel}
          projects={props.projects}
          environments={props.environments}
          onSelectProject={props.onSelectProject}
          onSelectEnvironment={props.onSelectEnvironment}
          onCreateProject={props.onCreateProject}
        />
      </div>

      <div class="min-w-3 flex-1" aria-hidden />

      <div class="flex shrink-0 items-center gap-1 [-webkit-app-region:no-drag]">
        <button
          type="button"
          aria-label="Search or run a command"
          title={`Commander (${commanderShortcut()})`}
          onClick={() => props.onOpenCommander?.()}
          class={headerIconControlClass}
        >
          <Search size={14} />
        </button>
        <button
          type="button"
          ref={(element) => props.terminalButtonRef?.(element)}
          aria-label="Open terminal"
          aria-expanded={Boolean(props.terminalPickerOpen)}
          title="Terminal"
          onClick={() => props.onOpenTerminal?.()}
          class={headerIconControlClass}
        >
          <Terminal size={14} />
        </button>
      </div>
    </header>
  );
}

const headerIconControlClass =
  "control-field control-field-md inline-flex w-[var(--control-height-md)] shrink-0 cursor-pointer items-center justify-center !rounded-full p-0 text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)] focus-visible:outline-none";

function navButtonStyle(disabled: boolean) {
  return {
    display: "inline-flex",
    "align-items": "center",
    "justify-content": "center",
    width: "24px",
    height: "24px",
    padding: "0",
    border: "none",
    "border-radius": "var(--radius-control)",
    background: "transparent",
    color: disabled ? "var(--color-text-disabled)" : "var(--color-text-secondary)",
    cursor: disabled ? "default" : "pointer",
  };
}
