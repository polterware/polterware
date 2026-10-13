import {
  ChevronLeft,
  ChevronRight,
  ControlButton,
  Search,
} from "@polterware/frontend-design-system";
import { createShortcutLabel } from "@polterware/frontend-state";
import { Show } from "solid-js";
import {
  WorkspaceContextBar,
  type WorkspaceContext,
} from "./context-switchers";
import { CoreStatusMenu, type CoreStatusMenuProps } from "./core-status-menu";
import { TitleBarFrame } from "@polterware/frontend-shell";
import type { EnvironmentItem, ProjectSwitcherItem } from "./types";

export type TitleBarProps = {
  canGoBack: boolean;
  canGoForward: boolean;
  onBack: () => void;
  onForward: () => void;
  workspace: WorkspaceContext;
  projects?: ProjectSwitcherItem[];
  environments?: EnvironmentItem[];
  onSelectProject?: (projectId: string) => void;
  onSelectEnvironment?: (environmentId: string) => void;
  onCreateProject?: () => void;
  onOpenCommander?: () => void;
  coreStatus?: CoreStatusMenuProps;
};

export function TitleBar(props: TitleBarProps) {
  const commanderShortcut = createShortcutLabel("shell.toggle_commander");

  return (
    <TitleBarFrame>
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
            projects={props.projects}
            environments={props.environments}
            onSelectProject={props.onSelectProject}
            onSelectEnvironment={props.onSelectEnvironment}
            onCreateProject={props.onCreateProject}
          />
        </div>

        <div class="min-w-3 flex-1" aria-hidden />

        <div class="flex shrink-0 items-center gap-1 [-webkit-app-region:no-drag]">
          <ControlButton
            icon
            tone="secondary"
            aria-label="Search or run a command"
            title={`Commander (${commanderShortcut()})`}
            onClick={() => props.onOpenCommander?.()}
          >
            <Search size={14} />
          </ControlButton>
          <Show when={props.coreStatus}>{(coreStatus) => <CoreStatusMenu {...coreStatus()} />}</Show>
        </div>
    </TitleBarFrame>
  );
}

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
