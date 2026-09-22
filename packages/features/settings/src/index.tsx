import { createEffect, createMemo, createSignal, onCleanup, Show, type JSX } from "solid-js";
import {
  A11yDialog,
  Button,
  Card,
  DataTable,
  Field,
  Input,
  PageContent,
  PageToolbar,
  Plus,
  Select,
  Server,
  StatusChip,
  Tabs,
  Textarea,
  ToolbarIconButton,
  pushToast,
} from "@polterware/frontend-design-system";
import { createDetailPanelSelector, detailPanelStore } from "@polterware/frontend-state";
import { CreateEnvironmentDialog, type CreateEnvironmentDraft } from "./create-environment-dialog";
import { EnvironmentsEmpty } from "./environments-empty";
import { EnvironmentDetail } from "./environment-detail";
import { environmentKindLabel, environmentKindTone, protectionLabel, protectionTone, archiveProjectToast, saveGeneralSettingsToast } from "./environment-status";
import type { ProjectEnvironment, ProjectGeneralSettings } from "./types";
import {
  findEnvironmentById,
} from "./fixtures";
import {
  environmentDetailTarget,
  environmentSelectOptions,
  environmentsDetailScope,
  normalizeSettingsTab,
  SETTINGS_TABS,
} from "./settings-helpers";
import { ShortcutsPage } from "./shortcuts-page";
import { AgentsPage } from "./agents-page";

export type { EnvironmentKind, ProjectEnvironment, ProjectGeneralSettings, ProtectionLevel, SettingsTabId } from "./types";
export type { CreateEnvironmentDraft } from "./create-environment-dialog";
export { SETTINGS_ENVIRONMENT_MOCKS, PROJECT_GENERAL_SETTINGS_MOCK } from "./fixtures";
export {
  environmentKindLabel,
  environmentKindTone,
  protectionLabel,
  protectionTone,
} from "./environment-status";
export { normalizeSettingsTab, SETTINGS_TABS } from "./settings-helpers";
export { EnvironmentDetail } from "./environment-detail";
export { CreateEnvironmentDialog } from "./create-environment-dialog";
export { ShortcutsPage } from "./shortcuts-page";
export { AgentsPage } from "./agents-page";
export { hydrateShortcutPreferences, persistShortcutOverrides } from "./shortcut-persist";

/** Project-scoped settings persisted in the local store. */
export function SettingsView(props: {
  tab?: string;
  projectId?: string;
  general?: ProjectGeneralSettings;
  environments?: ProjectEnvironment[];
  onTabChange?: (t: string) => void;
  onSaveGeneral?: (settings: ProjectGeneralSettings) => void;
  onCreateEnvironment?: (draft: CreateEnvironmentDraft) => void;
  onSetDefaultEnvironment?: (environmentId: string) => void;
  onDeleteEnvironment?: (environmentId: string) => void;
  onArchiveProject?: () => void;
  onResetLocalStore?: () => void;
  children?: JSX.Element;
}) {
  const tab = () => normalizeSettingsTab(props.tab);
  const scopeId = createMemo(() => environmentsDetailScope());
  const activeTarget = createDetailPanelSelector((state) => state.activeTarget);
  const general = () => props.general;
  const environments = () => props.environments ?? [];

  const [name, setName] = createSignal(general()?.name ?? "");
  const [description, setDescription] = createSignal(general()?.description ?? "");
  const [defaultEnvironmentId, setDefaultEnvironmentId] = createSignal(general()?.defaultEnvironmentId ?? "");
  const [repository, setRepository] = createSignal(general()?.repository ?? "");
  const [archiveOpen, setArchiveOpen] = createSignal(false);
  const [resetOpen, setResetOpen] = createSignal(false);
  const [createOpen, setCreateOpen] = createSignal(false);

  createEffect(() => {
    const next = general();
    if (!next) return;
    setName(next.name);
    setDescription(next.description);
    setDefaultEnvironmentId(next.defaultEnvironmentId);
    setRepository(next.repository);
  });

  const currentEnvironment = createMemo(() => {
    const target = activeTarget();
    if (
      target?.scopeId === environmentsDetailScope() &&
      target.entityKind === "environment"
    ) {
      return findEnvironmentById(target.entityId, environments());
    }
    return undefined;
  });

  createEffect(() => {
    const scope = scopeId();
    const targets =
      tab() === "environments"
        ? environments().map((environment) => environmentDetailTarget(environment.id))
        : [];
    detailPanelStore.getState().registerCollection({
      scopeId: scope,
      status: "ready",
      targets,
    });
    onCleanup(() => detailPanelStore.getState().unregisterCollection(scope));
  });

  function activate(entityId: string) {
    detailPanelStore.getState().activateTarget(environmentDetailTarget(entityId));
  }

  function handleSaveGeneral() {
    const current = general();
    props.onSaveGeneral?.({
      name: name(),
      description: description(),
      defaultEnvironmentId: defaultEnvironmentId(),
      projectId: current?.projectId ?? "",
      repository: repository(),
      dataSource: current?.dataSource ?? "empty",
    });
    if (!props.onSaveGeneral) {
      pushToast(saveGeneralSettingsToast(), "success");
    }
  }

  function handleArchiveConfirm() {
    setArchiveOpen(false);
    if (!props.onArchiveProject) {
      pushToast(archiveProjectToast(general()?.name), "success");
      return;
    }
    props.onArchiveProject();
  }

  async function handleCopyProjectId() {
    const id = general()?.projectId ?? "";
    try {
      if ("navigator" in globalThis && globalThis.navigator.clipboard?.writeText) {
        await globalThis.navigator.clipboard.writeText(id);
      }
    } catch {
      // Clipboard may be unavailable in some runtimes — toast still honest.
    }
    pushToast("Project ID copied.", "success");
  }

  return (
    <>
      <div class="flex h-full min-h-0 flex-col">
        <PageToolbar.Root>
          <PageToolbar.Tabs>
            <Tabs
              items={[...SETTINGS_TABS]}
              value={tab()}
              onChange={(id) => props.onTabChange?.(id)}
              aria-label="Project settings"
            />
          </PageToolbar.Tabs>
          <Show when={tab() === "environments"}>
            <PageToolbar.Actions>
              <ToolbarIconButton label="Add environment" onClick={() => setCreateOpen(true)}>
                <Plus size={14} />
              </ToolbarIconButton>
            </PageToolbar.Actions>
          </Show>
        </PageToolbar.Root>

        <Show when={tab() === "general"}>
          <div class="flex min-h-0 flex-1 flex-col">
            <PageContent class="max-w-xl space-y-4 p-4 text-[12.5px]">
              <p class="m-0 text-[11px] text-[var(--color-text-muted)]">
                Local project details. Changes persist in the local SQLite store.
              </p>
              <Field label="Name" for="settings-project-name">
                <Input
                  id="settings-project-name"
                  size="md"
                  value={name()}
                  onInput={(e) => setName(e.currentTarget.value)}
                />
              </Field>
              <Field label="Description" for="settings-project-description">
                <Textarea
                  id="settings-project-description"
                  value={description()}
                  onInput={(e) => setDescription(e.currentTarget.value)}
                />
              </Field>
              <Field label="Primary repository" for="settings-project-repo">
                <Input
                  id="settings-project-repo"
                  size="md"
                  placeholder="org/repo"
                  value={repository()}
                  onInput={(e) => setRepository(e.currentTarget.value)}
                />
              </Field>
              <Field label="Default environment" for="settings-default-env">
                <Select
                  id="settings-default-env"
                  size="md"
                  value={defaultEnvironmentId()}
                  options={environmentSelectOptions(environments())}
                  onChange={(value) => setDefaultEnvironmentId(value)}
                />
              </Field>
              <div>
                <p class="m-0 text-[11px] font-medium tracking-wide text-[var(--color-text-muted)]">
                  Project ID (System)
                </p>
                <p class="m-0 mt-1 font-mono text-[12px] text-[var(--color-text-primary)]">
                  {general()?.projectId ?? "—"}{" "}
                  <Button size="sm" variant="ghost" onClick={() => void handleCopyProjectId()}>
                    Copy
                  </Button>
                </p>
              </div>
              <div class="flex justify-end gap-2">
                <Button
                  size="sm"
                  variant="ghost"
                  onClick={() => {
                    setName(general()?.name ?? "");
                    setDescription(general()?.description ?? "");
                    setDefaultEnvironmentId(general()?.defaultEnvironmentId ?? "");
                    setRepository(general()?.repository ?? "");
                  }}
                >
                  Cancel
                </Button>
                <Button size="sm" variant="secondary" onClick={handleSaveGeneral}>
                  Save changes
                </Button>
              </div>
            </PageContent>
          </div>
        </Show>

        <Show when={tab() === "environments"}>
          <div class="flex min-h-0 flex-1 flex-col">
            <PageContent class="p-0">
              <DataTable.Root
                rows={environments()}
                getRowId={(environment) => environment.id}
                activeRowId={
                  activeTarget()?.scopeId === environmentsDetailScope()
                    ? activeTarget()?.entityId
                    : null
                }
                activateFirstRow
                onActiveRowChange={(id) => {
                  if (id) activate(id);
                }}
              >
                <DataTable.Table aria-label="Environments">
                  <DataTable.Header>
                    <DataTable.ColumnHeader
                      id="name"
                      sortBy={(environment) => environment.name}
                    >
                      Environment
                    </DataTable.ColumnHeader>
                    <DataTable.ColumnHeader
                      id="kind"
                      sortBy={(environment) => environment.kind}
                    >
                      Kind
                    </DataTable.ColumnHeader>
                    <DataTable.ColumnHeader
                      id="protection"
                      sortBy={(environment) => environment.protection}
                    >
                      Protection
                    </DataTable.ColumnHeader>
                    <DataTable.ColumnHeader
                      id="default"
                      sortBy={(environment) => (environment.isDefault ? 1 : 0)}
                    >
                      Default
                    </DataTable.ColumnHeader>
                  </DataTable.Header>
                  <DataTable.Body>
                    {(environment) => (
                      <DataTable.Row
                        row={environment}
                        onActivate={() => activate(environment.id)}
                      >
                        <DataTable.PrimaryCell
                          icon={<Server size={16} />}
                          iconTone="infrastructure"
                          title={environment.name}
                        />
                        <DataTable.Cell>
                          <StatusChip
                            tone={environmentKindTone(environment.kind)}
                            label={environmentKindLabel(environment.kind)}
                          />
                        </DataTable.Cell>
                        <DataTable.Cell>
                          <StatusChip
                            tone={protectionTone(environment.protection)}
                            label={protectionLabel(environment.protection)}
                          />
                        </DataTable.Cell>
                        <DataTable.Cell>
                          {environment.isDefault ? "Yes" : "—"}
                        </DataTable.Cell>
                      </DataTable.Row>
                    )}
                  </DataTable.Body>
                  <DataTable.Empty>
                    <EnvironmentsEmpty
                      onCreate={() => setCreateOpen(true)}
                      onCreateFromTemplate={
                        props.onCreateEnvironment
                          ? (template) => props.onCreateEnvironment?.(template.draft)
                          : undefined
                      }
                    />
                  </DataTable.Empty>
                </DataTable.Table>
              </DataTable.Root>
            </PageContent>
          </div>
        </Show>

        <Show when={tab() === "shortcuts"}>
          <ShortcutsPage />
        </Show>

        <Show when={tab() === "agents"}>
          <AgentsPage projectId={props.projectId ?? props.general?.projectId ?? ""} />
        </Show>

        <Show when={tab() === "advanced"}>
          <PageContent class="max-w-xl space-y-6 overflow-auto p-4 text-[12.5px]">
            <Card>
              <p class="m-0 text-[12px] font-medium text-[var(--color-text-primary)]">
                Danger zone
              </p>
              <p class="m-0 mt-1 text-[12px] text-[var(--color-text-muted)]">
                Reset recreates the demonstration project in the local store. Architecture, resources, and knowledge are replaced.
              </p>
              <div class="mt-2 flex gap-2">
                <Button size="sm" variant="danger" onClick={() => setArchiveOpen(true)}>
                  Archive project
                </Button>
                <Button size="sm" variant="ghost" onClick={() => setResetOpen(true)}>
                  Reset local store
                </Button>
              </div>
            </Card>
          </PageContent>
        </Show>

        {props.children}
      </div>

      <Show when={tab() === "environments"}>
        <EnvironmentDetail
          environment={currentEnvironment()}
          notFound={
            activeTarget()?.scopeId === environmentsDetailScope() &&
            activeTarget()?.entityKind === "environment" &&
            !currentEnvironment()
          }
          canDelete={environments().length > 1}
          onSetDefault={
            props.onSetDefaultEnvironment
              ? (environmentId) => props.onSetDefaultEnvironment?.(environmentId)
              : undefined
          }
          onDelete={
            props.onDeleteEnvironment
              ? (environmentId) => props.onDeleteEnvironment?.(environmentId)
              : undefined
          }
        />
      </Show>

      <CreateEnvironmentDialog
        open={createOpen()}
        onClose={() => setCreateOpen(false)}
        onCreateFromTemplate={(template) => {
          setCreateOpen(false);
          props.onCreateEnvironment?.(template.draft);
        }}
        onSubmit={(draft) => {
          setCreateOpen(false);
          props.onCreateEnvironment?.(draft);
        }}
      />

      <A11yDialog
        open={archiveOpen()}
        title="Archive project?"
        onClose={() => setArchiveOpen(false)}
        footer={
          <>
            <Button size="sm" variant="ghost" onClick={() => setArchiveOpen(false)}>
              Cancel
            </Button>
            <Button size="sm" variant="danger" onClick={handleArchiveConfirm}>
              Archive
            </Button>
          </>
        }
      >
        <p class="m-0 text-[12.5px] text-[var(--color-text-secondary)]">
          The project will be hidden from the switcher. Local data stays in SQLite until you reset the store.
        </p>
      </A11yDialog>
      <A11yDialog
        open={resetOpen()}
        title="Reset local store?"
        onClose={() => setResetOpen(false)}
        footer={
          <>
            <Button size="sm" variant="ghost" onClick={() => setResetOpen(false)}>
              Cancel
            </Button>
            <Button
              size="sm"
              variant="danger"
              onClick={() => {
                setResetOpen(false);
                props.onResetLocalStore?.();
              }}
            >
              Reset store
            </Button>
          </>
        }
      >
        <p class="m-0 text-[12.5px] text-[var(--color-text-secondary)]">
          This deletes local project data and reseeds the demonstration project.
        </p>
      </A11yDialog>
    </>
  );
}
