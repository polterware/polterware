import { Match, Switch, createSignal, onCleanup, onMount } from "solid-js";
import {
  Shell,
  CreateProjectPage,
  TerminalPage,
  ENVIRONMENT_FIXTURES,
  PROJECT_SWITCHER_FIXTURES,
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
  type ProjectSwitcherItem,
} from "@polterware/feature-shell";
import { ArchitectureView } from "@polterware/feature-architecture";
import {
  ResourcesView,
  RESOURCE_FIXTURES,
} from "@polterware/feature-resources";
import { OperationsView } from "@polterware/feature-operations";
import { KnowledgeView } from "@polterware/feature-knowledge";
import { IntegrationsView } from "@polterware/feature-integrations";
import { SettingsView } from "@polterware/feature-settings";
import { CommandPalette } from "@polterware/feature-commander";
import { pushToast } from "@polterware/frontend-design-system";
import { detailPanelStore } from "@polterware/frontend-state";
import { bootstrapPlatform } from "./platform-bootstrap";

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
    || kind === "preview"
    || kind === "test"
    || kind === "local";
}

export function App() {
  const initialRoute = createDefaultRoute("architecture", PLATFORM_CONTEXT);
  const [route, setRoute] = createSignal<ClientRoute>(initialRoute);
  const [history, setHistory] = createSignal<ClientRoute[]>([historySnapshot(initialRoute)]);
  const [historyIndex, setHistoryIndex] = createSignal(0);
  const [paletteOpen, setPaletteOpen] = createSignal(false);
  const [projects, setProjects] = createSignal<ProjectSwitcherItem[]>(PROJECT_SWITCHER_FIXTURES);
  const [environments, setEnvironments] = createSignal<EnvironmentItem[]>(ENVIRONMENT_FIXTURES);

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
      environments().find((item) => item.projectId === projectId) ?? environments()[0];
    applyRoute(
      enterProject(route(), projectId, {
        environmentId: environment?.id,
      }),
    );
  }

  return (
    <Shell
      route={route}
      projects={projects()}
      environments={environments()}
      onNavigate={navigateSection}
      onProjectSelect={selectProject}
      onCreateProject={() => applyRoute(goCreateProject(route()))}
      onSelectEnvironment={(environmentId) =>
        applyRoute(setEnvironment(route(), environmentId))
      }
      onOpenCommander={() => setPaletteOpen(true)}
      onExpandTerminal={() => applyRoute(goSection(route(), "terminal"))}
      onOpenSettings={() => applyRoute(setTab(goSection(route(), "settings"), "general"))}
      canGoBack={historyIndex() > 0}
      canGoForward={historyIndex() < history().length - 1}
      onBack={goBack}
      onForward={goForward}
      status={{ dataMode: "fixture" }}
      platform={PLATFORM_CONTEXT}
    >
      <Switch>
        <Match when={route().section === "create-project"}>
          <CreateProjectPage
            onCancel={() => applyRoute(goSection(route(), "architecture"))}
            onCreate={(input) => {
              const suffix = Date.now().toString(36);
              const projectId = `proj-${suffix}`;
              const createdEnvironments = input.environments.map((kind, index) => {
                const validKind = isEnvironmentKind(kind) ? kind : "development";

                return {
                  id: `env-${suffix}-${index}`,
                  name: kind === "production" ? "main" : kind,
                  kind: validKind,
                  projectId,
                };
              });
              setProjects((items) => [
                ...items.map((item) => ({ ...item, isCurrent: false })),
                {
                  projectId,
                  name: input.name,
                  directory: input.directory,
                  environmentSummary: input.environments.join(", "),
                  isCurrent: true,
                  status: "active",
                  refId: projectId,
                },
              ]);
              setEnvironments((items) => [...items, ...createdEnvironments]);
              applyRoute(
                enterProject(route(), projectId, {
                  environmentId: createdEnvironments[0]?.id,
                }),
              );
              pushToast(`Created ${input.name} for this preview session.`, "success");
            }}
          />
        </Match>

        <Match when={route().section === "architecture"}>
          <ArchitectureView
            systems={[{ id: SYSTEM_ID, name: "Polter" }]}
            resources={RESOURCE_FIXTURES.map((resource) => ({
              id: resource.id,
              name: resource.name,
              type: resource.type,
              source: resource.provider,
              systemId: SYSTEM_ID,
            }))}
            dataSource="fixture"
            fixtureMode="force"
            inspector={{ knowledge: [], ops: [], logs: [] }}
            tab={route().tab}
            search={route().search}
            selectedNodeId={route().selectedNodeId}
            createDialogOpen={route().dialog?.type === "arch-node-create"}
            onSelect={(id) => applyRoute(setSelectedNode(route(), id), "silent")}
            onOpenNode={(id) => activateDetail("architecture:canvas", id, "arch-node")}
            onTabChange={(tab) => applyRoute(setTab(route(), tab))}
            onCreateNode={() =>
              applyRoute(openDialog(route(), { type: "arch-node-create" }), "silent")
            }
            onCloseCreateNode={() => applyRoute(closeDialog(route()), "silent")}
            onSubmitCreateNode={() => {
              applyRoute(closeDialog(route()), "silent");
              pushToast("Architecture node added for this preview session.", "success");
            }}
          />
        </Match>

        <Match when={route().section === "resources"}>
          <ResourcesView
            catalog={RESOURCE_FIXTURES}
            dataSource="fixture"
            tab={route().tab}
            search={route().search}
            filters={[...(route().filters?.status ?? [])]}
            environmentId={route().environmentId}
            dialogOpen={route().dialog?.type === "resource-create"}
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
            onCloseDialog={() => applyRoute(closeDialog(route()), "silent")}
            onSubmitDialog={() => {
              applyRoute(closeDialog(route()), "silent");
              pushToast("Resource added for this preview session.", "success");
            }}
            onTerminal={() => applyRoute(goSection(route(), "terminal"))}
            onRunCommand={() =>
              applyRoute(openDialog(goSection(route(), "operations"), { type: "run-command" }))
            }
          />
        </Match>

        <Match when={route().section === "operations"}>
          <OperationsView
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
            onRunCommand={() => applyRoute(openDialog(route(), { type: "run-command" }), "silent")}
            onCloseRunCommand={() => applyRoute(closeDialog(route()), "silent")}
            onSubmitRunCommand={(input) => {
              applyRoute(closeDialog(route()), "silent");
              pushToast(`Previewed command: ${input.command}`, "success");
            }}
          />
        </Match>

        <Match when={route().section === "terminal"}>
          <TerminalPage
            sessionId={route().sessionId}
            onSelectSession={(id) => applyRoute(setSessionId(route(), id), "silent")}
          />
        </Match>

        <Match when={route().section === "knowledge"}>
          <KnowledgeView
            tab={route().tab}
            onTabChange={(tab) => applyRoute(setTab(route(), tab))}
            onOpenDoc={(id) => activateDetail("knowledge:docs", id, "document")}
            onOpenRunbook={(id) => activateDetail("knowledge:runbooks", id, "runbook")}
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
            onTabChange={(tab) => applyRoute(setTab(route(), tab))}
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
