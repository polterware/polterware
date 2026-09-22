import { Match, Switch, createEffect, createSignal, onCleanup, onMount } from "solid-js";
import {
  Shell,
  CreateProjectDialog,
  createDefaultRoute,
  goSection,
  enterProject,
  setTab,
  openDialog,
  closeDialog,
  setSearch,
  setFilters,
  setSelectedNode,
  setEnvironment,
  historySnapshot,
  ensureRouteAvailable,
  routesEqual,
  type ClientRoute,
  type EnvironmentItem,
} from "@polterware/feature-shell";
import { ArchitectureView } from "@polterware/feature-architecture";
import { ResourcesView } from "@polterware/feature-resources";
import { KnowledgeView } from "@polterware/feature-knowledge";
import { IntegrationsView } from "@polterware/feature-integrations";
import { SettingsView, AgentsPage, hydrateShortcutPreferences, hydrateAppearancePreferences } from "@polterware/feature-settings";
import { CommandPalette } from "@polterware/feature-commander";
import { pushToast } from "@polterware/frontend-design-system";
import { detailPanelStore, matchesShortcut } from "@polterware/frontend-state";
import { bootstrapPlatform } from "./platform-bootstrap";
import { emptyWorkspace, loadProjects, loadWorkspace, type WorkspaceSnapshot } from "./workspace/load";
import { shouldOpenCreateProject } from "./workspace/first-project";
import {
  persistCreatedProject,
  persistDeleteEnvironment,
  persistDeleteView,
  persistArchiveProject,
  persistEnvironment,
  persistEnvironmentDefault,
  persistIntegrationConnection,
  persistIntegrationSync,
  persistImportProjectSources,
  persistKnowledgeBody,
  persistKnowledgeCreate,
  persistKnowledgeCreateFeature,
  persistKnowledgeArchiveFeature,
  persistKnowledgeDelete,
  persistKnowledgeDuplicate,
  persistKnowledgeSourcePath,
  persistKnowledgeSetFeatureStatus,
  persistConstitutionEnsure,
  persistConstitutionArchive,
  persistLayout,
  persistProjectSettings,
  persistResource,
  persistView,
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
  const [createProjectOpen, setCreateProjectOpen] = createSignal(false);
  const [importSourcesBusy, setImportSourcesBusy] = createSignal(false);
  const [workspace, setWorkspace] = createSignal<WorkspaceSnapshot>(emptyWorkspace());
  const [loading, setLoading] = createSignal(true);

  async function refreshWorkspace(projectId = route().projectId ?? "") {
    if (!projectId) {
      setLoading(false);
      return;
    }
    try {
      const snapshot = await loadWorkspace(projectId);
      setWorkspace(snapshot);
      if (shouldOpenCreateProject({
        coreAvailable: snapshot.available,
        projectCount: snapshot.projects.length,
      })) {
        loadedProjectId = "";
        setCreateProjectOpen(true);
        return;
      }
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
  const [bootstrapped, setBootstrapped] = createSignal(false);

  async function bootstrapProjects() {
    try {
      const listed = await loadProjects();
      if (shouldOpenCreateProject({
        coreAvailable: true,
        projectCount: listed.projects.length,
      })) {
        setCreateProjectOpen(true);
        setWorkspace({ ...emptyWorkspace(), available: true, projects: [] });
        setLoading(false);
        setBootstrapped(true);
        return;
      }
      const preferredId =
        (route().projectId && listed.projects.some((project) => project.projectId === route().projectId)
          ? route().projectId
          : undefined)
        ?? listed.projects[0]?.projectId;
      if (!preferredId) {
        setLoading(false);
        setBootstrapped(true);
        return;
      }
      if (route().projectId !== preferredId) {
        const environment = listed.environments.find((item) => item.projectId === preferredId);
        applyRoute(
          enterProject(route(), preferredId, { environmentId: environment?.id }),
          "replace",
        );
      }
      loadedProjectId = preferredId;
      await refreshWorkspace(preferredId);
      setBootstrapped(true);
    } catch (error) {
      setWorkspace({
        ...emptyWorkspace(),
        healthMessage: error instanceof Error ? error.message : "Core unavailable",
      });
      setLoading(false);
      setBootstrapped(true);
    }
  }

  createEffect(() => {
    if (!bootstrapped()) return;
    if (createProjectOpen() && workspace().projects.length === 0) return;
    const projectId = route().projectId;
    if (!projectId || projectId === loadedProjectId) return;
    loadedProjectId = projectId;
    void refreshWorkspace(projectId);
  });

  onMount(() => {
    void bootstrapProjects();
    void hydrateShortcutPreferences();
    void hydrateAppearancePreferences();
    const onKeyDown = (event: KeyboardEvent) => {
      if (!matchesShortcut("shell.toggle_commander", event)) return;
      event.preventDefault();
      setPaletteOpen((open) => !open);
    };
    window.addEventListener("keydown", onKeyDown);
    const unsubscribeExecute = window.polterware.commander.onExecuteCommand((id: string) => {
      handlePaletteCommand(id);
    });
    onCleanup(() => {
      window.removeEventListener("keydown", onKeyDown);
      unsubscribeExecute();
    });
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

  return (
    <Shell
      route={route}
      projects={workspace().projects}
      environments={workspace().environments}
      onNavigate={navigateSection}
      onProjectSelect={selectProject}
      onCreateProject={() => setCreateProjectOpen(true)}
      onSelectEnvironment={(environmentId) =>
        applyRoute(setEnvironment(route(), environmentId))
      }
      onOpenCommander={() => setPaletteOpen(true)}
      onOpenSettings={() => applyRoute(setTab(goSection(route(), "settings"), "general"))}
      canGoBack={historyIndex() > 0}
      canGoForward={historyIndex() < history().length - 1}
      onBack={goBack}
      onForward={goForward}
      status={{ dataMode: dataMode() }}
      platform={PLATFORM_CONTEXT}
    >
      <Switch>
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
            inspector={{
              knowledge: (() => {
                const nodeId = route().selectedNodeId;
                if (!nodeId) return [];
                const entity = workspace().ppmEntities.find(
                  (item) => item.kind === "component" && item.refId === nodeId,
                );
                if (!entity) return [];
                const linked = [
                  ...workspace().documents,
                  ...workspace().adrs,
                  ...workspace().runbooks,
                ].flatMap((item) =>
                  item.entityLinks.some((link) => link.entityId === entity.id)
                    ? [{ id: item.id, title: item.title, body: "summary" in item ? item.summary : "" }]
                    : [],
                );
                return linked;
              })(),
              ops: [],
              logs: [],
            }}
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
              setWorkspace((current) => {
                if (
                  current.layoutBody === bodyJson &&
                  current.layoutRevision === result.revision
                ) {
                  return current;
                }
                return {
                  ...current,
                  layoutBody: bodyJson,
                  layoutRevision: result.revision,
                };
              });
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
            dataSource="empty"
            search={route().search}
            filters={route().filters}
            environmentId={route().environmentId}
            dialogOpen={route().dialog?.type === "resource-create"}
            onSearch={(search) => applyRoute(setSearch(route(), search), "replace")}
            onFiltersChange={(filters) =>
              applyRoute(setFilters(route(), filters), "replace")
            }
            onOpen={(id) => activateDetail("resources:inventory", id, "resource")}
            onOpenArchitecture={(nodeId) =>
              navigateToDetail(
                setSelectedNode(setTab(goSection(route(), "architecture"), "canvas"), nodeId),
                "architecture:canvas",
                nodeId,
                "arch-node",
              )
            }
            onCreate={() => applyRoute(openDialog(route(), { type: "resource-create" }), "silent")}
            onCloseDialog={() => applyRoute(closeDialog(route()), "silent")}
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
          />
        </Match>

        <Match when={route().section === "knowledge"}>
          <KnowledgeView
            documents={workspace().documents}
            adrs={workspace().adrs}
            runbooks={workspace().runbooks}
            sources={workspace().sources}
            specFeatures={workspace().specFeatures}
            constitution={workspace().constitution}
            entities={workspace().ppmEntities}
            tab={route().tab}
            onTabChange={(tab) => applyRoute(setTab(route(), tab))}
            onOpenDoc={(id) => activateDetail("knowledge:library", id, "document")}
            onOpenAdr={(id) => activateDetail("knowledge:library", id, "adr")}
            onOpenRunbook={(id) => activateDetail("knowledge:library", id, "runbook")}
            onOpenSource={(id) => activateDetail("knowledge:sources", id, "source")}
            onOpenFeature={(featureKey) => activateDetail("knowledge:specs", featureKey, "feature")}
            onOpenConstitution={(id) => activateDetail("knowledge:specs", id, "constitution")}
            onOpenEntity={(_entityId, refId, kind) => {
              if (kind === "component") {
                applyRoute(setTab(goSection(route(), "architecture"), "canvas"));
                activateDetail("architecture:canvas", refId, "arch-node");
              } else if (kind === "resource") {
                applyRoute(setTab(goSection(route(), "resources"), "inventory"));
                activateDetail("resources:inventory", refId, "resource");
              }
            }}
            onLinkEntity={(knowledgeId, entityId, kind) => {
              const predicate = kind === "resource" || kind === "component"
                ? (workspace().runbooks.some((item) => item.id === knowledgeId) ? "OPERATES" : "DOCUMENTS")
                : "RELATED_TO";
              void window.polterware.core.knowledge.saveLink({
                link: {
                  projectId: currentProjectId(),
                  knowledgeId,
                  entityId,
                  predicate,
                  truthClass: "declared",
                  source: "desktop",
                  actor: "human",
                },
              }).then(async () => {
                await refreshWorkspace(currentProjectId());
                pushToast("Entity linked.", "success");
              }).catch((error: unknown) => {
                pushToast(error instanceof Error ? error.message : "Could not link entity.", "danger");
              });
            }}
            onUnlinkEntity={(linkId) => {
              void window.polterware.core.knowledge.deleteLink({ id: linkId }).then(async () => {
                await refreshWorkspace(currentProjectId());
                pushToast("Entity unlinked.", "success");
              }).catch((error: unknown) => {
                pushToast(error instanceof Error ? error.message : "Could not unlink entity.", "danger");
              });
            }}
            onSaveBody={(id, body) => {
              // Persist without refreshWorkspace — full reload per keystroke freezes Knowledge + TipTap.
              void persistKnowledgeBody(workspace(), currentProjectId(), id, body).catch((error: unknown) => {
                pushToast(error instanceof Error ? error.message : "Could not save knowledge.", "danger");
              });
            }}
            onSaveSourcePath={(id, path) => {
              void persistKnowledgeSourcePath(workspace(), currentProjectId(), id, path).catch((error: unknown) => {
                pushToast(error instanceof Error ? error.message : "Could not save source path.", "danger");
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
                  activateDetail("knowledge:library", id, "runbook");
                  pushToast("Runbook saved locally.", "success");
                } else if (kind === "adr") {
                  activateDetail("knowledge:library", id, "adr");
                  pushToast("ADR saved locally.", "success");
                } else {
                  activateDetail("knowledge:library", id, "document");
                  pushToast("Document saved locally.", "success");
                }
              }).catch((error: unknown) => {
                pushToast(error instanceof Error ? error.message : "Could not create knowledge.", "danger");
              });
            }}
            onCreateSource={({ title, path }) => {
              void persistKnowledgeCreate({
                projectId: currentProjectId(),
                kind: "source",
                title,
                extraJson: JSON.stringify({ path }),
              }).then(async (id) => {
                await refreshWorkspace(currentProjectId());
                activateDetail("knowledge:sources", id, "source");
                pushToast("Source saved locally.", "success");
              }).catch((error: unknown) => {
                pushToast(error instanceof Error ? error.message : "Could not create source.", "danger");
              });
            }}
            onImportProjectSources={() => {
              if (importSourcesBusy()) return;
              setImportSourcesBusy(true);
              void persistImportProjectSources({ projectId: currentProjectId() })
                .then(async (result) => {
                  await refreshWorkspace(currentProjectId());
                  pushToast(result.message, "success");
                })
                .catch((error: unknown) => {
                  pushToast(
                    error instanceof Error ? error.message : "Could not import project sources.",
                    "danger",
                  );
                })
                .finally(() => setImportSourcesBusy(false));
            }}
            importBusy={importSourcesBusy()}
            onCreateFromTemplate={(template) => {
              if (template.kind === "feature") {
                void persistKnowledgeCreateFeature({
                  workspace: workspace(),
                  projectId: currentProjectId(),
                  title: template.title,
                }).then(async (featureKey) => {
                  await refreshWorkspace(currentProjectId());
                  activateDetail("knowledge:specs", featureKey, "feature");
                  pushToast("Feature saved locally.", "success");
                }).catch((error: unknown) => {
                  pushToast(error instanceof Error ? error.message : "Could not create feature.", "danger");
                });
                return;
              }
              void persistKnowledgeCreate({
                projectId: currentProjectId(),
                kind: template.kind,
                title: template.title,
                body: template.body,
                extraJson: template.extraJson,
              }).then(async (id) => {
                await refreshWorkspace(currentProjectId());
                if (template.kind === "runbook") {
                  activateDetail("knowledge:library", id, "runbook");
                  pushToast("Runbook saved locally.", "success");
                } else if (template.kind === "adr") {
                  activateDetail("knowledge:library", id, "adr");
                  pushToast("ADR saved locally.", "success");
                } else if (template.kind === "source") {
                  activateDetail("knowledge:sources", id, "source");
                  pushToast("Source saved locally.", "success");
                } else {
                  activateDetail("knowledge:library", id, "document");
                  pushToast("Document saved locally.", "success");
                }
              }).catch((error: unknown) => {
                pushToast(error instanceof Error ? error.message : "Could not create knowledge.", "danger");
              });
            }}
            onCreateFeature={(title) => {
              void persistKnowledgeCreateFeature({
                workspace: workspace(),
                projectId: currentProjectId(),
                title,
              }).then(async (featureKey) => {
                await refreshWorkspace(currentProjectId());
                activateDetail("knowledge:specs", featureKey, "feature");
                pushToast("Feature saved locally.", "success");
              }).catch((error: unknown) => {
                pushToast(error instanceof Error ? error.message : "Could not create feature.", "danger");
              });
            }}
            onEnsureConstitution={() => {
              void persistConstitutionEnsure({
                workspace: workspace(),
                projectId: currentProjectId(),
              }).then(async (id) => {
                await refreshWorkspace(currentProjectId());
                activateDetail("knowledge:specs", id, "constitution");
                pushToast("Constitution saved locally.", "success");
              }).catch((error: unknown) => {
                pushToast(error instanceof Error ? error.message : "Could not create constitution.", "danger");
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
            onArchiveFeature={(featureKey) => {
              void persistKnowledgeArchiveFeature(workspace(), featureKey).then(async () => {
                pushToast("Feature archived.", "success");
                await refreshWorkspace(currentProjectId());
              }).catch((error: unknown) => {
                pushToast(error instanceof Error ? error.message : "Could not archive feature.", "danger");
              });
            }}
            onArchiveConstitution={() => {
              void persistConstitutionArchive(workspace()).then(async () => {
                pushToast("Constitution archived.", "success");
                await refreshWorkspace(currentProjectId());
              }).catch((error: unknown) => {
                pushToast(error instanceof Error ? error.message : "Could not archive constitution.", "danger");
              });
            }}
            onSetFeatureStatus={(featureKey, status) => {
              void persistKnowledgeSetFeatureStatus(
                workspace(),
                currentProjectId(),
                featureKey,
                status,
              ).then(async () => {
                await refreshWorkspace(currentProjectId());
                pushToast(status === "ready" ? "Feature marked ready." : "Feature marked draft.", "success");
              }).catch((error: unknown) => {
                pushToast(error instanceof Error ? error.message : "Could not update feature status.", "danger");
              });
            }}
            onDuplicate={(id) => {
              void persistKnowledgeDuplicate(workspace(), currentProjectId(), id).then(async (nextId) => {
                await refreshWorkspace(currentProjectId());
                if (nextId) activateDetail("knowledge:library", nextId, "runbook");
                pushToast("Runbook duplicated.", "success");
              }).catch((error: unknown) => {
                pushToast(error instanceof Error ? error.message : "Could not duplicate runbook.", "danger");
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
          />
        </Match>

        <Match when={route().section === "agents"}>
          <AgentsPage
            projectId={currentProjectId()}
            tab={route().tab}
            onTabChange={(tab) => applyRoute(setTab(route(), tab))}
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
            projectId={currentProjectId()}
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
                  loadedProjectId = "";
                  setCreateProjectOpen(true);
                  setWorkspace({ ...emptyWorkspace(), available: true, projects: [] });
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

      <CreateProjectDialog
        open={createProjectOpen()}
        canCancel={workspace().projects.length > 0}
        onClose={() => {
          if (workspace().projects.length === 0) return;
          setCreateProjectOpen(false);
        }}
        onPickDirectory={async () => {
          const result = await window.polterware.app.selectDirectory();
          return result.path;
        }}
        onCreate={(input) => {
          void (async () => {
            const kinds = input.environments.filter(isEnvironmentKind);
            const created = await persistCreatedProject({
              ...input,
              environments: kinds.length > 0 ? kinds : ["development"],
            });
            setCreateProjectOpen(false);
            applyRoute(
              enterProject(route(), created.projectId, {
                environmentId: created.environmentId,
              }),
            );
            pushToast(`Created ${input.name}.`, "success");
            loadedProjectId = created.projectId;
            await refreshWorkspace(created.projectId);
          })().catch((error: unknown) => {
            pushToast(error instanceof Error ? error.message : "Could not create project.", "danger");
          });
        }}
      />

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
