import type { ArchViewCard } from "@polterware/feature-architecture";
import type { OperationRun, OperationSchedule, OperationWorkflow } from "@polterware/feature-operations";
import type { NodeFixture, ProcessFixture, Resource } from "@polterware/feature-resources";
import type { ProjectEnvironment, ProjectGeneralSettings } from "@polterware/feature-settings";
import type {
  EnvironmentItem,
  ProjectSwitcherItem,
  TerminalTargetItem,
  WorkspaceCatalogProvider,
  WorkspaceDocument,
  WorkspaceIntegration,
  WorkspaceIntegrationResource,
  WorkspaceIntegrationSyncRun,
  WorkspaceRunbook,
} from "@polterware/feature-shell";
import { asNumber, asObjectArray, asRecord, asString } from "./format";
import {
  mapConnections,
  mapDocuments,
  mapEnvironments,
  mapIntegrationResources,
  mapNodes,
  mapProcesses,
  mapProjects,
  mapProviders,
  mapResources,
  mapRunbooks,
  mapRuns,
  mapSchedules,
  mapSettings,
  mapSettingsEnvironments,
  mapSyncRuns,
  mapTerminalTargets,
  mapViews,
  mapWorkflows,
} from "./map-records";

export type WorkspaceSnapshot = {
  available: boolean;
  quarantined: boolean;
  healthMessage: string;
  projects: ProjectSwitcherItem[];
  environments: EnvironmentItem[];
  settingsEnvironments: ProjectEnvironment[];
  settings: ProjectGeneralSettings;
  resources: Resource[];
  nodes: NodeFixture[];
  processes: ProcessFixture[];
  terminalTargets: TerminalTargetItem[];
  documents: WorkspaceDocument[];
  runbooks: WorkspaceRunbook[];
  runs: OperationRun[];
  workflows: OperationWorkflow[];
  schedules: OperationSchedule[];
  providers: WorkspaceCatalogProvider[];
  integrations: WorkspaceIntegration[];
  integrationResources: WorkspaceIntegrationResource[];
  syncRuns: WorkspaceIntegrationSyncRun[];
  views: ArchViewCard[];
  layoutBody: string;
  layoutRevision: number;
  rawKnowledge: Record<string, unknown>[];
};

const emptySettings: ProjectGeneralSettings = {
  name: "",
  description: "",
  defaultEnvironmentId: "",
  projectId: "",
  repository: "",
  dataSource: "unavailable",
};

export function emptyWorkspace(): WorkspaceSnapshot {
  return {
    available: false,
    quarantined: false,
    healthMessage: "",
    projects: [],
    environments: [],
    settingsEnvironments: [],
    settings: emptySettings,
    resources: [],
    nodes: [],
    processes: [],
    terminalTargets: [],
    documents: [],
    runbooks: [],
    runs: [],
    workflows: [],
    schedules: [],
    providers: [],
    integrations: [],
    integrationResources: [],
    syncRuns: [],
    views: [],
    layoutBody: "",
    layoutRevision: 0,
    rawKnowledge: [],
  };
}

function core() {
  return window.polterware.core;
}

export async function loadProjects(currentProjectId?: string): Promise<{
  projects: ProjectSwitcherItem[];
  rawProjects: Record<string, unknown>[];
  environments: EnvironmentItem[];
}> {
  const listed = asRecord(await core().projects.list());
  const rawProjects = asObjectArray(listed.projects);
  const environments: EnvironmentItem[] = [];
  for (const project of rawProjects) {
    const listedEnvs = asRecord(await core().projects.listEnvironments({ projectId: asString(project.id) }));
    environments.push(...mapEnvironments(asObjectArray(listedEnvs.environments)));
  }
  return {
    projects: mapProjects(rawProjects, environments, currentProjectId),
    rawProjects,
    environments,
  };
}

export async function loadWorkspace(projectId: string): Promise<WorkspaceSnapshot> {
  const api = core();
  const health = asRecord(await api.health.getStoreHealth());
  const listed = asRecord(await api.projects.list());
  const rawProjects = asObjectArray(listed.projects);
  const environments: EnvironmentItem[] = [];
  for (const project of rawProjects) {
    const listedEnvs = asRecord(await api.projects.listEnvironments({ projectId: asString(project.id) }));
    environments.push(...mapEnvironments(asObjectArray(listedEnvs.environments)));
  }
  const projectEnvironments = environments.filter((item) => item.projectId === projectId);
  const currentEnvRecords = (await Promise.all(rawProjects.map(async (project) => {
    if (asString(project.id) !== projectId) return [];
    const listedEnvs = asRecord(await api.projects.listEnvironments({ projectId }));
    return asObjectArray(listedEnvs.environments);
  }))).flat();

  const [settingsRes, layoutRes, resourcesRes, nodesRes, processesRes, knowledgeRes, operationsRes, workflowsRes, schedulesRes, providersRes, connectionsRes, integrationResourcesRes, syncRunsRes, viewsRes] = await Promise.all([
    api.projects.getSettings({ projectId }),
    api.architecture.getLayout({ projectId }).catch(() => ({})),
    api.architecture.listResources({ projectId }),
    api.nodes.list({ projectId }),
    api.processes.list({ projectId }),
    api.knowledge.list({ projectId }),
    api.operations.list({ projectId, pageSize: 50 }),
    api.workflows.list({ projectId }),
    api.operations.listSchedules({ projectId }),
    api.integrations.listProviders(),
    api.integrations.listConnections({ projectId }),
    api.integrations.listResources({ projectId }),
    api.integrations.listSyncRuns({ projectId }),
    api.architecture.listViews({ projectId }),
  ]);

  const resources = mapResources(asObjectArray(asRecord(resourcesRes).resources), projectEnvironments);
  const rawNodes = asObjectArray(asRecord(nodesRes).nodes);
  const nodes = mapNodes(rawNodes, projectEnvironments);
  const processes = mapProcesses(asObjectArray(asRecord(processesRes).sessions), nodes);
  const knowledgeItems = asObjectArray(asRecord(knowledgeRes).items);
  const operations = asObjectArray(asRecord(operationsRes).operations);
  const logsByOperation: Record<string, string[]> = {};
  await Promise.all(operations.map(async (operation) => {
    const id = asString(operation.id);
    const logs = asRecord(await api.logs.query({ operationId: id, limit: 50 }));
    logsByOperation[id] = asObjectArray(logs.chunks).map((chunk) => asString(chunk.body));
  }));
  const workflows = mapWorkflows(asObjectArray(asRecord(workflowsRes).workflows));
  const integrationResources = mapIntegrationResources(asObjectArray(asRecord(integrationResourcesRes).resources), projectEnvironments);
  const providersRaw = asObjectArray(asRecord(providersRes).providers);
  const integrations = mapConnections(asObjectArray(asRecord(connectionsRes).connections), providersRaw, projectEnvironments, integrationResources);
  const layout = asRecord(asRecord(layoutRes).layout);

  return {
    available: true,
    quarantined: health.quarantined === true,
    healthMessage: asString(health.message),
    projects: mapProjects(rawProjects, environments, projectId),
    environments,
    settingsEnvironments: mapSettingsEnvironments(currentEnvRecords, resources, nodes),
    settings: mapSettings(asRecord(asRecord(settingsRes).settings), rawProjects.find((item) => asString(item.id) === projectId)),
    resources,
    nodes,
    processes,
    terminalTargets: mapTerminalTargets(rawNodes),
    documents: mapDocuments(knowledgeItems),
    runbooks: mapRunbooks(knowledgeItems),
    runs: mapRuns(operations, projectEnvironments, logsByOperation),
    workflows,
    schedules: mapSchedules(asObjectArray(asRecord(schedulesRes).schedules), workflows, projectEnvironments),
    providers: mapProviders(providersRaw, integrations),
    integrations,
    integrationResources,
    syncRuns: mapSyncRuns(asObjectArray(asRecord(syncRunsRes).runs)),
    views: mapViews(asObjectArray(asRecord(viewsRes).views)),
    layoutBody: asString(layout.bodyJson),
    layoutRevision: asNumber(layout.revision),
    rawKnowledge: knowledgeItems,
  };
}
