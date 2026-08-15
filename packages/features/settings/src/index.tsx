import { createEffect, createMemo, createSignal, onCleanup, Show, type JSX } from "solid-js";
import {
  A11yDialog,
  Button,
  Card,
  DataTable,
  Field,
  Input,
  PageContent,
  PageEmpty,
  PageToolbar,
  Select,
  Server,
  StatusChip,
  Tabs,
  Textarea,
  pushToast,
} from "@polterware/frontend-design-system";
import { createDetailPanelSelector, detailPanelStore } from "@polterware/frontend-state";
import { EnvironmentDetail } from "./environment-detail";
import {
  archiveProjectToast,
  environmentKindLabel,
  environmentKindTone,
  protectionLabel,
  protectionTone,
  saveGeneralSettingsToast,
} from "./environment-status";
import {
  findEnvironmentById,
  PROJECT_GENERAL_SETTINGS_MOCK,
  SETTINGS_ENVIRONMENT_MOCKS,
} from "./fixtures";
import {
  environmentDetailTarget,
  environmentSelectOptions,
  environmentsDetailScope,
  normalizeSettingsTab,
  SETTINGS_TABS,
} from "./settings-helpers";

export type { ProjectEnvironment, ProjectGeneralSettings, SettingsTabId } from "./types";
export { SETTINGS_ENVIRONMENT_MOCKS, PROJECT_GENERAL_SETTINGS_MOCK } from "./fixtures";
export {
  environmentKindLabel,
  environmentKindTone,
  protectionLabel,
  protectionTone,
} from "./environment-status";
export { normalizeSettingsTab, SETTINGS_TABS } from "./settings-helpers";
export { EnvironmentDetail } from "./environment-detail";

/** Project-scoped settings for the local visual version. */
export function SettingsView(props: {
  tab?: string;
  onTabChange?: (t: string) => void;
  children?: JSX.Element;
}) {
  const tab = () => normalizeSettingsTab(props.tab);
  const scopeId = createMemo(() => environmentsDetailScope());
  const activeTarget = createDetailPanelSelector((state) => state.activeTarget);

  const [name, setName] = createSignal(PROJECT_GENERAL_SETTINGS_MOCK.name);
  const [description, setDescription] = createSignal(
    PROJECT_GENERAL_SETTINGS_MOCK.description,
  );
  const [defaultEnvironmentId, setDefaultEnvironmentId] = createSignal(
    PROJECT_GENERAL_SETTINGS_MOCK.defaultEnvironmentId,
  );
  const [repository, setRepository] = createSignal(
    PROJECT_GENERAL_SETTINGS_MOCK.repository,
  );
  const [archiveOpen, setArchiveOpen] = createSignal(false);

  const environments = () => SETTINGS_ENVIRONMENT_MOCKS;

  const currentEnvironment = createMemo(() => {
    const target = activeTarget();
    if (
      target?.scopeId === environmentsDetailScope() &&
      target.entityKind === "environment"
    ) {
      return findEnvironmentById(target.entityId);
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
    pushToast(saveGeneralSettingsToast(), "success");
  }

  function handleArchiveConfirm() {
    setArchiveOpen(false);
    pushToast(archiveProjectToast(), "default");
  }

  async function handleCopyProjectId() {
    const id = PROJECT_GENERAL_SETTINGS_MOCK.projectId;
    try {
      if (typeof navigator !== "undefined" && navigator.clipboard?.writeText) {
        await navigator.clipboard.writeText(id);
      }
    } catch {
      // Clipboard may be unavailable in some runtimes — toast still honest.
    }
    pushToast("Project ID copied (mock session clipboard)", "success");
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
        </PageToolbar.Root>

        <Show when={tab() === "general"}>
          <div class="flex min-h-0 flex-1 flex-col">
            <PageContent class="max-w-xl space-y-4 p-4 text-[12.5px]">
              <p class="m-0 text-[11px] text-[var(--color-text-muted)]">
                Local project details. Changes are stored in this preview session only.
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
                  options={environmentSelectOptions()}
                  onChange={(value) => setDefaultEnvironmentId(value)}
                />
              </Field>
              <div>
                <p class="m-0 text-[11px] font-medium tracking-wide text-[var(--color-text-muted)]">
                  Project ID (System)
                </p>
                <p class="m-0 mt-1 font-mono text-[12px] text-[var(--color-text-primary)]">
                  {PROJECT_GENERAL_SETTINGS_MOCK.projectId}{" "}
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
                    setName(PROJECT_GENERAL_SETTINGS_MOCK.name);
                    setDescription(PROJECT_GENERAL_SETTINGS_MOCK.description);
                    setDefaultEnvironmentId(
                      PROJECT_GENERAL_SETTINGS_MOCK.defaultEnvironmentId,
                    );
                    setRepository(PROJECT_GENERAL_SETTINGS_MOCK.repository);
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
                    <PageEmpty
                      title="No environments"
                      description="This project has no environments in fixtures."
                    />
                  </DataTable.Empty>
                </DataTable.Table>
              </DataTable.Root>
            </PageContent>
          </div>
        </Show>

        <Show when={tab() === "advanced"}>
          <PageContent class="max-w-xl space-y-6 overflow-auto p-4 text-[12.5px]">
            <Card>
              <p class="m-0 text-[12px] font-medium text-[var(--color-text-primary)]">
                Danger zone
              </p>
              <p class="m-0 mt-1 text-[12px] text-[var(--color-text-muted)]">
                Preview archiving this project. Files and persisted data are not changed.
              </p>
              <div class="mt-2 flex gap-2">
                <Button size="sm" variant="danger" onClick={() => setArchiveOpen(true)}>
                  Archive project
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
        />
      </Show>

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
              Archive (mock)
            </Button>
          </>
        }
      >
        <p class="m-0 text-[12.5px] text-[var(--color-text-secondary)]">
          This confirms an archive preview for the current session only. It does not
          modify project files or external services.
        </p>
      </A11yDialog>
    </>
  );
}
