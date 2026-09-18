import { Match, Switch, createEffect, createSignal, onCleanup, onMount } from "solid-js";
import {
  Shell,
  CreateProjectPage,
  TerminalPage,
  createDefaultRoute,
  goSection,
  goCreateProject,
  enterProject,
  setTab,
  openDialog,
  closeDialog,
  setSearch,
  setFilters,
  setSelectedNode,
  setEnvironment,
  setSessionId,
  historySnapshot,
  ensureRouteAvailable,
  routesEqual,
  type ClientRoute,
  type EnvironmentItem,
  type LiveTerminalSession,
} from "@polterware/feature-shell";
import { ArchitectureView } from "@polterware/feature-architecture";
import { ResourcesView } from "@polterware/feature-resources";
import { OperationsView } from "@polterware/feature-operations";
import { KnowledgeView } from "@polterware/feature-knowledge";
import { IntegrationsView } from "@polterware/feature-integrations";
import { SettingsView } from "@polterware/feature-settings";
import { CommandPalette } from "@polterware/feature-commander";
import { pushToast } from "@polterware/frontend-design-system";
import { detailPanelStore } from "@polterware/frontend-state";
import { bootstrapPlatform } from "./platform-bootstrap";
import { emptyWorkspace, loadProjects, loadWorkspace, type WorkspaceSnapshot } from "./workspace/load";
import {
  persistCreatedProject,
  persistDeleteEnvironment,
  persistDeleteView,
  persistArchiveProject,
  persistEnvironment,
  persistEnvironmentDefault,
  persistIntegrationConnection,
  persistIntegrationSync,
  persistKnowledgeBody,
  persistKnowledgeCreate,
  persistKnowledgeDelete,
  persistKnowledgeDuplicate,
  persistLayout,
  persistOperationPreview,
  persistProjectSettings,
  persistNode,
  persistResource,
  persistStartProcess,
  persistStopProcess,
  persistRunbookRun,
  persistSchedule,
  persistView,
  persistWorkflow,
  persistWorkflowRun,
  resetLocalStore,
} from "./workspace/persist";

const SYSTEM_ID = "sys-local";
const PLATFORM_CONTEXT = bootstrapPlatform();

function integrationSetupProvider(route: ClientRoute) {
  return route.dialog?.type === "integration-connect"
    ? route.dialog.providerId
    : undefined;
}

function isEnvironmentKind(kind: string): kind is EnvironmentItem["kind"] {
  return kind === "production"
    || kind === "staging"
    || kind === "development"
    || kind === "local";
}

export function App() {
  const initialRoute = createDefaultRoute("architecture", PLATFORM_CONTEXT);
  const [route, setRoute] = createSignal<ClientRoute>(initialRoute);
  const [history, setHistory] = createSignal<ClientRoute[]>([historySnapshot(initialRoute)]);
  const [historyIndex, setHistoryIndex] = createSignal(0);
  const [paletteOpen, setPaletteOpen] = createSignal(false);
  const [workspace, setWorkspace] = createSignal<WorkspaceSnapshot>(emptyWorkspace());
  const [loading, setLoading] = createSignal(true);

  async function refreshWorkspace(projectId = route().projectId ?? "") {
    try {
      const snapshot = await loadWorkspace(projectId || "proj-polter");
      setWorkspace(snapshot);
      const current = route();
      if (!current.environmentId) {
        const environment = snapshot.environments.find((item) => item.projectId === snapshot.settings.projectId || item.projectId === current.projectId);
        if (environment) applyRoute(setEnvironment(current, environment.id), "silent");
      }
    } catch (error) {
      setWorkspace({
        ...emptyWorkspace(),
        healthMessage: error instanceof Error ? error.message : "Core unavailable",
      });
    } finally {
      setLoading(false);
    }
  }

  let loadedProjectId = "";
  createEffect(() => {
    const projectId = route().projectId ?? "proj-polter";
    if (projectId === loadedProjectId) return;
    loadedProjectId = projectId;
    void refreshWorkspace(projectId);
  });

  onMount(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "k") {
        event.preventDefault();
        setPaletteOpen((open) => !open);
      }
    };
    window.addEventListener("keydown", onKeyDown);
    onCleanup(() => window.removeEventListener("keydown", onKeyDown));
  });

  type NavigationMode = "push" | "replace" | "silent";

  function applyRoute(next: ClientRoute, mode: NavigationMode = "push") {
    const availableRoute = ensureRouteAvailable(next, PLATFORM_CONTEXT);
    setRoute(availableRoute);
    if (mode === "silent") return;
    const snapshot = historySnapshot(availableRoute);
    if (mode === "replace") {
      setHistory((items) => {
        const copy = items.slice();
        copy[historyIndex()] = snapshot;
        return copy;
      });
      return;
    }
    setHistory((items) => {
      const base = items.slice(0, historyIndex() + 1);
      const lastBase = base[base.length - 1];
      if (lastBase && routesEqual(lastBase, snapshot)) return base;
      const nextItems = [...base, snapshot];
      setHistoryIndex(nextItems.length - 1);
      return nextItems;
    });
  }

  function navigateSection(section: string) {
    applyRoute(goSection(route(), section));
  }

  function activateDetail(scopeId: string, entityId: string, entityKind: string) {
    detailPanelStore.getState().activateTarget({ scopeId, entityId, entityKind });
  }

  function navigateToDetail(
    nextRoute: ClientRoute,
    scopeId: string,
    entityId: string,
    entityKind: string,
  ) {
    detailPanelStore.getState().requestTarget({ scopeId, entityId, entityKind });
    applyRoute(nextRoute);
  }

  function goBack() {
    if (historyIndex() <= 0) return;
    const nextIndex = historyIndex() - 1;
    const targetRoute = history()[nextIndex];
    if (!targetRoute) return;
    setHistoryIndex(nextIndex);
    setRoute({ ...targetRoute, dialog: null });
  }

  function goForward() {
    if (historyIndex() >= history().length - 1) return;
    const nextIndex = historyIndex() + 1;
    const targetRoute = history()[nextIndex];
    if (!targetRoute) return;
    setHistoryIndex(nextIndex);
    setRoute({ ...targetRoute, dialog: null });
  }

  function handlePaletteCommand(id: string) {
    if (id.startsWith("nav.")) {
      navigateSection(id.slice(4));
      return;
    }
    if (id === "action.open-sync") {
      applyRoute(setTab(goSection(route(), "settings"), "advanced"));
      return;
    }
    if (id === "action.create-node") {
      applyRoute(openDialog(goSection(route(), "architecture"), { type: "arch-node-create" }));
      return;
    }
    if (id === "action.run-command") {
      applyRoute(openDialog(goSection(route(), "operations"), { type: "run-command" }));
      return;
    }
    if (id === "action.connect-integration") {
      applyRoute(setTab(goSection(route(), "integrations"), "catalog"));
    }
  }

  function selectProject(projectId: string) {
    const environment =
      workspace().environments.find((item) => item.projectId === projectId) ?? workspace().environments[0];
    applyRoute(
      enterProject(route(), projectId, {
        environmentId: environment?.id,
      }),
    );
  }

  const currentProjectId = () => route().projectId ?? workspace().settings.projectId;
  const dataMode = () => {
    if (!workspace().available) return "offline" as const;
    if (workspace().quarantined) return "offline" as const;
    return "local" as const;
  };

  const [handedOffTerminalSession, setHandedOffTerminalSession] = createSignal<LiveTerminalSession | null>(
    null,
  );

  return (
    <Shell
      route={route}
      projects={workspace().projects}
      environments={workspace().environments}
      terminalTargets={workspace().terminalTargets}
      projectId={currentProjectId()}
      onNavigate={navigateSection}
      onProjectSelect={selectProject}
      onCreateProject={() => applyRoute(goCreateProject(route()))}
      onSelectEnvironment={(environmentId) =>
        applyRoute(setEnvironment(route(), environmentId))
      }
      onOpenCommander={() => setPaletteOpen(true)}
      onExpandTerminal={(payload) => {
        setHandedOffTerminalSession(payload.session);
        applyRoute(setSessionId(goSection(route(), "terminal"), payload.session.id));
      }}
      onCreateTerminalSession={async (input) => {
        const { createLiveTerminalSession } = await import("./workspace/terminal");
        const session = await createLiveTerminalSession(input);
        const target = workspace().terminalTargets.find((item) => item.id === input.targetId);
        return {
          ...session,
          name: target?.name ?? session.name,
          nodeName: target?.name ?? session.nodeName,
        };
      }}
      onWriteTerminalSession={async (id, data) => {
        const { writeLiveTerminalSession } = await import("./workspace/terminal");
        await writeLiveTerminalSession(id, data);
      }}
      onResizeTerminalSession={async (id, cols, rows) => {
        const { resizeLiveTerminalSession } = await import("./workspace/terminal");
        await resizeLiveTerminalSession(id, cols, rows);
      }}
      onCloseTerminalSession={async (id) => {
        const { closeLiveTerminalSession } = await import("./workspace/terminal");
        await closeLiveTerminalSession(id);
      }}
      onReadTerminalSession={async (id, offset) => {
        const { readLiveTerminalOutput } = await import("./workspace/terminal");
        return readLiveTerminalOutput(id, offset);
      }}
      onOpenSettings={() => applyRoute(setTab(goSection(route(), "settings"), "general"))}
      canGoBack={historyIndex() > 0}
      canGoForward={historyIndex() < history().length - 1}
      onBack={goBack}
      onForward={goForward}
      status={{ dataMode: dataMode() }}
      platform={PLATFORM_CONTEXT}
    >
      <Switch>
        <Match when={route().section === "create-project"}>
          <CreateProjectPage
            onCancel={() => applyRoute(goSection(route(), "architecture"))}
            onCreate={(input) => {
              void (async () => {
                const kinds = input.environments.filter(isEnvironmentKind);
                const created = await persistCreatedProject({
                  ...input,
                  environments: kinds.length > 0 ? kinds : ["development"],
                });
                applyRoute(
                  enterProject(route(), created.projectId, {
                    environmentId: created.environmentId,
                  }),
                );
                pushToast(`Created ${input.name}.`, "success");
                await refreshWorkspace(created.projectId);
              })().catch((error: unknown) => {
                pushToast(error instanceof Error ? error.message : "Could not create project.", "danger");
              });
            }}
          />
        </Match>

        <Match when={route().section === "architecture"}>
          <ArchitectureView
            systems={[{ id: SYSTEM_ID, name: workspace().settings.name || "Polterware" }]}
            resources={workspace().resources.map((resource) => ({
              id: resource.id,
              name: resource.name,
              type: resource.type,
              source: resource.provider,
              systemId: SYSTEM_ID,
            }))}
            layoutBody={workspace().layoutBody}
            layoutRevision={workspace().layoutRevision}
            dataSource="empty"
            fixtureMode="off"
            loading={loading()}
            inspector={{ knowledge: [], ops: [], logs: [] }}
            tab={route().tab}
            search={route().search}
            selectedNodeId={route().selectedNodeId}
            views={workspace().views}
            createDialogOpen={route().dialog?.type === "arch-node-create"}
            onSelect={(id) => applyRoute(setSelectedNode(route(), id), "silent")}
            onOpenNode={(id) => activateDetail("architecture:canvas", id, "arch-node")}
            onTabChange={(tab) => applyRoute(setTab(route(), tab))}
            onCreateNode={() =>
              applyRoute(openDialog(route(), { type: "arch-node-create" }), "silent")
            }
            onCloseCreateNode={() => applyRoute(closeDialog(route()), "silent")}
            onSubmitCreateNode={(result) => {
              applyRoute(closeDialog(route()), "silent");
              if (!result.ok) {
                pushToast(result.error || "Could not save architecture node.", "danger");
                return;
              }
              pushToast("Architecture node saved to the local canvas.", "success");
            }}
            onSaveLayout={async (bodyJson, expectedRevision) => {
              const result = await persistLayout(currentProjectId(), bodyJson, expectedRevision);
              setWorkspace((current) => ({
                ...current,
                layoutBody: bodyJson,
                layoutRevision: result.revision,
              }));
              return result;
            }}
            onSaveView={(draft) =>
              persistView({
                projectId: currentProjectId(),
                ...draft,
              }).then(async () => {
                pushToast("View saved locally.", "success");
                await refreshWorkspace(currentProjectId());
              }).catch((error: unknown) => {
                pushToast(error instanceof Error ? error.message : "Could not save view.", "danger");
                throw error;
              })
            }
            onDeleteView={(id) =>
              persistDeleteView(id).then(async () => {
                pushToast("View deleted.", "success");
                await refreshWorkspace(currentProjectId());
              }).catch((error: unknown) => {
                pushToast(error instanceof Error ? error.message : "Could not delete view.", "danger");
                throw error;
              })
            }
          />
        </Match>

        <Match when={route().section === "resources"}>
          <ResourcesView
            catalog={workspace().resources}
            nodes={workspace().nodes}
            processes={workspace().processes}
            dataSource="empty"
            tab={route().tab}
            search={route().search}
            filters={[...(route().filters?.status ?? [])]}
            environmentId={route().environmentId}
            dialogOpen={route().dialog?.type === "resource-create"}
            nodeDialogOpen={route().dialog?.type === "node-create"}
            processDialogOpen={route().dialog?.type === "process-create"}
            onSearch={(search) => applyRoute(setSearch(route(), search), "replace")}
            onFiltersChange={(filters) =>
              applyRoute(setFilters(route(), { status: filters }), "replace")
            }
            onTabChange={(tab) => applyRoute(setTab(route(), tab))}
            onOpen={(id) => activateDetail("resources:inventory", id, "resource")}
            onOpenNode={(id) => activateDetail("resources:nodes", id, "node")}
            onOpenProcess={(id) => activateDetail("resources:processes", id, "process")}
            onOpenArchitecture={(nodeId) =>
              navigateToDetail(
                setSelectedNode(setTab(goSection(route(), "architecture"), "canvas"), nodeId),
                "architecture:canvas",
                nodeId,
                "arch-node",
              )
            }
            onCreate={() => applyRoute(openDialog(route(), { type: "resource-create" }), "silent")}
            onCreateNode={() => applyRoute(openDialog(route(), { type: "node-create" }), "silent")}
            onCreateProcess={() => applyRoute(openDialog(route(), { type: "process-create" }), "silent")}
            onCloseDialog={() => applyRoute(closeDialog(route()), "silent")}
            onCloseNodeDialog={() => applyRoute(closeDialog(route()), "silent")}
            onCloseProcessDialog={() => applyRoute(closeDialog(route()), "silent")}
            onSubmitDialog={(draft) => {
              void persistResource({
                projectId: currentProjectId(),
                environmentId: route().environmentId,
                name: draft.name,
                type: draft.type,
              }).then(async () => {
                applyRoute(closeDialog(route()), "silent");
                pushToast("Resource saved locally.", "success");
                await refreshWorkspace(currentProjectId());
              }).catch((error: unknown) => {
                pushToast(error instanceof Error ? error.message : "Could not save resource.", "danger");
              });
            }}
            onSubmitNodeDialog={(draft) => {
              void persistNode({
                projectId: currentProjectId(),
                environmentId: route().environmentId,
                name: draft.name,
                platform: draft.platform,
                shell: draft.shell,
              }).then(async () => {
                applyRoute(closeDialog(route()), "silent");
                pushToast("Node saved locally.", "success");
                await refreshWorkspace(currentProjectId());
              }).catch((error: unknown) => {
                pushToast(error instanceof Error ? error.message : "Could not save node.", "danger");
              });
            }}
            onSubmitProcessDialog={(draft) => {
              void persistStartProcess({
                projectId: currentProjectId(),
                environmentId: route().environmentId,
                name: draft.name,
                command: draft.command,
                cwd: draft.cwd,
              }).then(async () => {
                applyRoute(closeDialog(route()), "silent");
                pushToast("Process started on the local host.", "success");
                await refreshWorkspace(currentProjectId());
              }).catch((error: unknown) => {
                pushToast(error instanceof Error ? error.message : "Could not start process.", "danger");
              });
            }}
            onStopProcess={(id) => {
              void persistStopProcess(id).then(async () => {
                pushToast("Process stopped.", "success");
                await refreshWorkspace(currentProjectId());
              }).catch((error: unknown) => {
                pushToast(error instanceof Error ? error.message : "Could not stop process.", "danger");
              });
            }}
            onTerminal={() => applyRoute(goSection(route(), "terminal"))}
            onRunCommand={() =>
              applyRoute(openDialog(goSection(route(), "operations"), { type: "run-command" }))
            }
          />
        </Match>

        <Match when={route().section === "operations"}>
          <OperationsView
            runs={workspace().runs}
            workflows={workspace().workflows}
            schedules={workspace().schedules}
            tab={route().tab}
            search={route().search}
            filters={[...(route().filters?.status ?? [])]}
            runCommandOpen={route().dialog?.type === "run-command"}
            onTabChange={(tab) => applyRoute(setTab(route(), tab))}
            onSearch={(search) => applyRoute(setSearch(route(), search), "replace")}
            onFiltersChange={(filters) =>
              applyRoute(setFilters(route(), { status: filters }), "replace")
            }
            onOpenRun={(id) => activateDetail("operations:runs", id, "run")}
            onOpenWorkflow={(id) => activateDetail("operations:workflows", id, "workflow")}
            onOpenSchedule={(id) => activateDetail("operations:schedules", id, "schedule")}
            onCreateWorkflow={(draft) => {
              void persistWorkflow({
                projectId: currentProjectId(),
                name: draft.name,
                description: draft.description,
              }).then(async () => {
                pushToast(`Workflow "${draft.name}" saved locally.`, "success");
                await refreshWorkspace(currentProjectId());
              }).catch((error: unknown) => {
                pushToast(error instanceof Error ? error.message : "Could not save workflow.", "danger");
              });
            }}
            onCreateSchedule={(draft) => {
              void persistSchedule({
                projectId: currentProjectId(),
                environmentId: route().environmentId,
                name: draft.name,
                workflowId: draft.workflowId,
                expression: draft.expression,
                timezone: draft.timezone,
              }).then(async () => {
                pushToast(`Schedule "${draft.name}" saved locally.`, "success");
                await refreshWorkspace(currentProjectId());
              }).catch((error: unknown) => {
                pushToast(error instanceof Error ? error.message : "Could not save schedule.", "danger");
              });
            }}
            onRunWorkflow={(workflowId) => {
              void persistWorkflowRun({
                workflowId,
                environmentId: route().environmentId,
              }).then(async () => {
                pushToast("Workflow started locally.", "success");
                await refreshWorkspace(currentProjectId());
                // Brief refresh loop while steps settle.
                window.setTimeout(() => {
                  void refreshWorkspace(currentProjectId());
                }, 1500);
              }).catch((error: unknown) => {
                pushToast(error instanceof Error ? error.message : "Could not run workflow.", "danger");
              });
            }}
            onRunCommand={() => applyRoute(openDialog(route(), { type: "run-command" }), "silent")}
            onCloseRunCommand={() => applyRoute(closeDialog(route()), "silent")}
            onSubmitRunCommand={(input) => {
              void persistOperationPreview({
                projectId: currentProjectId(),
                environmentId: route().environmentId,
                ...input,
              }).then(async () => {
                applyRoute(closeDialog(route()), "silent");
                pushToast(`Running command: ${input.command}`, "success");
                await refreshWorkspace(currentProjectId());
                window.setTimeout(() => {
                  void refreshWorkspace(currentProjectId());
                }, 1500);
              }).catch((error: unknown) => {
                pushToast(error instanceof Error ? error.message : "Could not run operation.", "danger");
              });
            }}
          />
        </Match>

        <Match when={route().section === "terminal"}>
          <TerminalPage
            projectId={currentProjectId()}
            sessionId={route().sessionId}
            initialSession={handedOffTerminalSession()}
            targets={workspace().terminalTargets}
            onSelectSession={(id) => {
              applyRoute(setSessionId(route(), id), "silent");
              if (handedOffTerminalSession()?.id === id) {
                setHandedOffTerminalSession(null);
              }
            }}
            onCreateSession={async (input) => {
              const { createLiveTerminalSession } = await import("./workspace/terminal");
              const session = await createLiveTerminalSession(input);
              const target = workspace().terminalTargets.find((item) => item.id === input.targetId);
              return {
                ...session,
                name: target?.name ?? session.name,
                nodeName: target?.name ?? session.nodeName,
              };
            }}
            onWriteSession={async (id, data) => {
              const { writeLiveTerminalSession } = await import("./workspace/terminal");
              await writeLiveTerminalSession(id, data);
            }}
            onResizeSession={async (id, cols, rows) => {
              const { resizeLiveTerminalSession } = await import("./workspace/terminal");
              await resizeLiveTerminalSession(id, cols, rows);
            }}
            onCloseSession={async (id) => {
              const { closeLiveTerminalSession } = await import("./workspace/terminal");
              await closeLiveTerminalSession(id);
            }}
            onReadSession={async (id, offset) => {
              const { readLiveTerminalOutput } = await import("./workspace/terminal");
              return readLiveTerminalOutput(id, offset);
            }}
          />
        </Match>

        <Match when={route().section === "knowledge"}>
          <KnowledgeView
            documents={workspace().documents}
            runbooks={workspace().runbooks}
            tab={route().tab}
            onTabChange={(tab) => applyRoute(setTab(route(), tab))}
            onOpenDoc={(id) => activateDetail("knowledge:docs", id, "document")}
            onOpenRunbook={(id) => activateDetail("knowledge:runbooks", id, "runbook")}
            onSaveBody={(id, body) => {
              void persistKnowledgeBody(workspace(), currentProjectId(), id, body).then(async () => {
                await refreshWorkspace(currentProjectId());
              }).catch((error: unknown) => {
                pushToast(error instanceof Error ? error.message : "Could not save knowledge.", "danger");
              });
            }}
            onCreate={(kind, title) => {
              void persistKnowledgeCreate({
                projectId: currentProjectId(),
                kind,
                title,
              }).then(async (id) => {
                await refreshWorkspace(currentProjectId());
                if (kind === "runbook") {
                  activateDetail("knowledge:runbooks", id, "runbook");
                } else {
                  activateDetail("knowledge:docs", id, "document");
                }
                pushToast(kind === "runbook" ? "Runbook saved locally." : "Document saved locally.", "success");
              }).catch((error: unknown) => {
                pushToast(error instanceof Error ? error.message : "Could not create knowledge.", "danger");
              });
            }}
            onArchive={(id) => {
              void persistKnowledgeDelete(id).then(async () => {
                pushToast("Knowledge archived.", "success");
                await refreshWorkspace(currentProjectId());
              }).catch((error: unknown) => {
                pushToast(error instanceof Error ? error.message : "Could not archive knowledge.", "danger");
              });
            }}
            onDuplicate={(id) => {
              void persistKnowledgeDuplicate(workspace(), currentProjectId(), id).then(async (nextId) => {
                await refreshWorkspace(currentProjectId());
                if (nextId) activateDetail("knowledge:runbooks", nextId, "runbook");
                pushToast("Runbook duplicated.", "success");
              }).catch((error: unknown) => {
                pushToast(error instanceof Error ? error.message : "Could not duplicate runbook.", "danger");
              });
            }}
            onRun={(id) => {
              const runbook = workspace().runbooks.find((item) => item.id === id);
              void persistRunbookRun({
                projectId: currentProjectId(),
                environmentId: route().environmentId,
                runbookId: id,
                title: runbook?.title ?? id,
                body: runbook?.bodyMarkdown ?? "",
              }).then(async () => {
                pushToast("Runbook started locally.", "success");
                await refreshWorkspace(currentProjectId());
                window.setTimeout(() => {
                  void refreshWorkspace(currentProjectId());
                }, 1500);
              }).catch((error: unknown) => {
                pushToast(error instanceof Error ? error.message : "Could not run runbook.", "danger");
              });
            }}
            onOpenComponent={(id) =>
              navigateToDetail(
                setTab(goSection(route(), "architecture"), "canvas"),
                "architecture:canvas",
                id,
                "arch-node",
              )
            }
            onOpenTerminal={() => applyRoute(goSection(route(), "terminal"))}
            onOpenOperation={(id) => {
              if (id) {
                navigateToDetail(goSection(route(), "operations"), "operations:runs", id, "run");
              } else {
                applyRoute(goSection(route(), "operations"));
              }
            }}
          />
        </Match>

        <Match when={route().section === "integrations"}>
          <IntegrationsView
            projectId={currentProjectId()}
            providers={workspace().providers}
            integrations={workspace().integrations}
            resources={workspace().integrationResources}
            syncRuns={workspace().syncRuns}
            tab={route().tab}
            onTabChange={(tab) => applyRoute(setTab(route(), tab))}
            onOpen={(id) => activateDetail("integrations:connected", id, "integration")}
            setupOpen={route().dialog?.type === "integration-connect"}
            setupProvider={integrationSetupProvider(route())}
            onOpenSetup={(providerId) =>
              applyRoute(
                openDialog(route(), { type: "integration-connect", providerId }),
                "silent",
              )
            }
            onCloseSetup={() => applyRoute(closeDialog(route()), "silent")}
            onConnect={(provider) => {
              void persistIntegrationConnection({
                projectId: currentProjectId(),
                environmentId: route().environmentId,
                provider,
              }).then(async () => {
                await refreshWorkspace(currentProjectId());
              }).catch((error: unknown) => {
                pushToast(error instanceof Error ? error.message : "Could not save connection.", "danger");
              });
            }}
            onSync={(integrationId) => {
              void persistIntegrationSync({
                connectionId: integrationId,
              }).then(async (run) => {
                if (run.result === "success") {
                  pushToast(run.message || "Integration inventory synced.", "success");
                } else {
                  pushToast(run.message || "Integration inventory sync failed.", "danger");
                }
                await refreshWorkspace(currentProjectId());
              }).catch((error: unknown) => {
                pushToast(error instanceof Error ? error.message : "Could not refresh integration metadata.", "danger");
              });
            }}
            onOpenResource={(id) =>
              navigateToDetail(
                setTab(goSection(route(), "resources"), "inventory"),
                "resources:inventory",
                id,
                "resource",
              )
            }
          />
        </Match>

        <Match when={route().section === "settings"}>
          <SettingsView
            tab={route().tab}
            general={workspace().settings}
            environments={workspace().settingsEnvironments}
            onTabChange={(tab) => applyRoute(setTab(route(), tab))}
            onSaveGeneral={(settings) => {
              void persistProjectSettings(
                currentProjectId(),
                settings,
                workspace().settingsEnvironments,
              ).then(async () => {
                pushToast("Project settings saved.", "success");
                await refreshWorkspace(currentProjectId());
              }).catch((error: unknown) => {
                pushToast(error instanceof Error ? error.message : "Could not save settings.", "danger");
              });
            }}
            onCreateEnvironment={(draft) => {
              void persistEnvironment({
                projectId: currentProjectId(),
                name: draft.name,
                kind: draft.kind,
                protection: draft.protection,
                existingCount: workspace().settingsEnvironments.length,
                settings: workspace().settings,
              }).then(async () => {
                pushToast(`Environment "${draft.name}" saved locally.`, "success");
                await refreshWorkspace(currentProjectId());
              }).catch((error: unknown) => {
                pushToast(error instanceof Error ? error.message : "Could not save environment.", "danger");
              });
            }}
            onSetDefaultEnvironment={(environmentId) => {
              const environment = workspace().settingsEnvironments.find((item) => item.id === environmentId);
              void persistEnvironmentDefault(workspace(), currentProjectId(), environmentId).then(async () => {
                pushToast(
                  environment
                    ? `${environment.name} is now the default environment.`
                    : "Default environment updated.",
                  "success",
                );
                await refreshWorkspace(currentProjectId());
              }).catch((error: unknown) => {
                pushToast(error instanceof Error ? error.message : "Could not update default environment.", "danger");
              });
            }}
            onDeleteEnvironment={(environmentId) => {
              const environment = workspace().settingsEnvironments.find((item) => item.id === environmentId);
              void persistDeleteEnvironment(currentProjectId(), environmentId).then(async () => {
                pushToast(
                  environment
                    ? `Environment "${environment.name}" deleted.`
                    : "Environment deleted.",
                  "success",
                );
                await refreshWorkspace(currentProjectId());
              }).catch((error: unknown) => {
                pushToast(error instanceof Error ? error.message : "Could not delete environment.", "danger");
              });
            }}
            onArchiveProject={() => {
              const projectId = currentProjectId();
              const name = workspace().settings.name;
              void persistArchiveProject(projectId).then(async () => {
                pushToast(`Project "${name}" archived.`, "success");
                const listed = await loadProjects();
                const fallback = listed.projects.find((project) => project.projectId !== projectId) ?? listed.projects[0];
                if (fallback) {
                  selectProject(fallback.projectId);
                  await refreshWorkspace(fallback.projectId);
                } else {
                  applyRoute(createDefaultRoute("architecture", PLATFORM_CONTEXT), "replace");
                  setWorkspace(emptyWorkspace());
                }
              }).catch((error: unknown) => {
                pushToast(error instanceof Error ? error.message : "Could not archive project.", "danger");
              });
            }}
            onResetLocalStore={() => {
              void resetLocalStore().then(async () => {
                pushToast("Local store reset to the demonstration project.", "success");
                applyRoute(createDefaultRoute("architecture", PLATFORM_CONTEXT), "replace");
                await refreshWorkspace("proj-polter");
              }).catch((error: unknown) => {
                pushToast(error instanceof Error ? error.message : "Could not reset the local store.", "danger");
              });
            }}
          />
        </Match>
      </Switch>

      <CommandPalette
        open={paletteOpen()}
        platform={PLATFORM_CONTEXT}
        onClose={() => setPaletteOpen(false)}
        onCommand={(id) => {
          setPaletteOpen(false);
          handlePaletteCommand(id);
        }}
      />
    </Shell>
  );
}
