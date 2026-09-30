import { For, Show, createMemo, createSignal, onCleanup } from "solid-js";
import { Plus } from "@polterware/frontend-design-system";
import {
  MenuItem,
  MenuList,
  Popover,
  SELECTION_ACK_MS,
  SelectionCheck,
  StatusChip,
} from "@polterware/frontend-design-system";
import type { EnvironmentItem, EnvironmentKind, ProjectSwitcherItem } from "./types";
import { ENVIRONMENT_FIXTURES, PROJECT_SWITCHER_FIXTURES } from "./fixtures";
import { PopoverSearchField } from "./popover-search";

export type WorkspaceContext = {
  projectId: string;
  environmentId: string;
};

type MenuId = "project" | "env" | null;

function kindLabel(kind: EnvironmentKind): string {
  return kind.toUpperCase();
}

function kindTone(kind: EnvironmentKind): "warn" | "syncing" | "muted" {
  if (kind === "production") return "warn";
  if (kind === "staging") return "syncing";
  return "muted";
}

const labelButtonClass =
  "inline-flex h-[var(--control-height-md)] max-w-[168px] shrink items-center gap-1 border-0 bg-transparent px-3 text-left text-[12px] font-medium text-[var(--color-text-primary)] hover:bg-[var(--color-surface-hover)] rounded-[var(--radius-item)]";

export function WorkspaceContextBar(props: {
  context: WorkspaceContext;
  projects?: ProjectSwitcherItem[];
  environments?: EnvironmentItem[];
  onSelectProject?: (projectId: string) => void;
  onSelectEnvironment?: (environmentId: string) => void;
  onCreateProject?: () => void;
}) {
  const [openMenu, setOpenMenu] = createSignal<MenuId>(null);
  const [query, setQuery] = createSignal("");
  let projectTrigger: HTMLButtonElement | undefined;
  let environmentTrigger: HTMLButtonElement | undefined;

  const projects = () => props.projects ?? PROJECT_SWITCHER_FIXTURES;
  const environments = () => props.environments ?? ENVIRONMENT_FIXTURES;
  const currentProject = createMemo(
    () => projects().find((project) => project.projectId === props.context.projectId) ?? projects()[0],
  );
  const currentEnvironment = createMemo(
    () =>
      environments().find((environment) => environment.id === props.context.environmentId) ??
      environments().find((environment) => environment.projectId === props.context.projectId) ??
      environments()[0],
  );
  const filteredProjects = createMemo(() => {
    const normalized = query().trim().toLowerCase();
    if (!normalized) return projects();
    return projects().filter((project) => project.name.toLowerCase().includes(normalized));
  });
  // A pick holds the popover open briefly so the marker glide and check are seen.
  let acknowledgeTimer: ReturnType<typeof setTimeout> | undefined;
  const clearAcknowledge = () => {
    if (acknowledgeTimer !== undefined) clearTimeout(acknowledgeTimer);
    acknowledgeTimer = undefined;
  };
  onCleanup(clearAcknowledge);
  const closeAfterAcknowledge = () => {
    clearAcknowledge();
    acknowledgeTimer = setTimeout(() => {
      acknowledgeTimer = undefined;
      close();
    }, SELECTION_ACK_MS);
  };
  const projectEnvironments = createMemo(() =>
    environments().filter((environment) => environment.projectId === props.context.projectId),
  );

  function close() {
    clearAcknowledge();
    setOpenMenu(null);
    setQuery("");
  }

  return (
    <div class="flex min-w-0 items-center gap-1">
      <button
        type="button"
        class={labelButtonClass}
        ref={(element) => {
          projectTrigger = element;
        }}
        aria-label={`Switch project: ${currentProject()?.name ?? "Project"}`}
        aria-expanded={openMenu() === "project"}
        onClick={() => setOpenMenu((current) => (current === "project" ? null : "project"))}
      >
        <span class="truncate">{currentProject()?.name ?? "Project"}</span>
      </button>

      <span class="text-[11px] text-[var(--color-text-disabled)]" aria-hidden>
        /
      </span>

      <button
        type="button"
        class={labelButtonClass}
        ref={(element) => {
          environmentTrigger = element;
        }}
        aria-label={`Switch environment: ${currentEnvironment()?.name ?? "Environment"}`}
        aria-expanded={openMenu() === "env"}
        onClick={() => setOpenMenu((current) => (current === "env" ? null : "env"))}
      >
        <Show when={currentEnvironment()}>
          {(environment) => (
            <StatusChip
              appearance="filled"
              tone={kindTone(environment().kind)}
              label={kindLabel(environment().kind)}
            />
          )}
        </Show>
      </button>

      <Popover
        open={openMenu() === "project"}
        onClose={close}
        title="Switch project"
        width={300}
        placement="bottom-start"
        anchorRef={() => projectTrigger}
      >
        <PopoverSearchField
          placeholder="Find project…"
          value={query()}
          onInput={setQuery}
          autofocus
        />
        <MenuList class="min-h-0 flex-1 overflow-auto p-2">
          <For each={filteredProjects()}>
            {(project) => (
              <MenuItem
                class="justify-between py-2 text-[13.5px]"
                selected={project.projectId === props.context.projectId}
                onClick={() => {
                  props.onSelectProject?.(project.projectId);
                  closeAfterAcknowledge();
                }}
              >
                <span class="truncate text-[var(--color-text-primary)]">{project.name}</span>
                <SelectionCheck checked={project.projectId === props.context.projectId} size={15} />
              </MenuItem>
            )}
          </For>
        </MenuList>
        <div class="border-t border-[var(--color-border-subtle)] p-2">
          <MenuItem
            class="gap-2 py-2 text-[13px]"
            onClick={() => {
              props.onCreateProject?.();
              close();
            }}
          >
            <Plus size={14} />
            New project
          </MenuItem>
        </div>
      </Popover>

      <Popover
        open={openMenu() === "env"}
        onClose={close}
        title="Switch environment"
        width={300}
        placement="bottom-start"
        anchorRef={() => environmentTrigger}
      >
        <MenuList class="min-h-0 flex-1 overflow-auto p-2">
          <For each={projectEnvironments()}>
            {(environment) => (
              <MenuItem
                class="justify-between py-2 text-[13.5px]"
                selected={environment.id === props.context.environmentId}
                onClick={() => {
                  props.onSelectEnvironment?.(environment.id);
                  closeAfterAcknowledge();
                }}
              >
                <StatusChip
                  appearance="filled"
                  tone={kindTone(environment.kind)}
                  label={kindLabel(environment.kind)}
                />
                <span class="min-w-0 flex-1 truncate text-[var(--color-text-secondary)]">
                  {environment.name}
                </span>
                <SelectionCheck checked={environment.id === props.context.environmentId} />
              </MenuItem>
            )}
          </For>
        </MenuList>
      </Popover>
    </div>
  );
}
