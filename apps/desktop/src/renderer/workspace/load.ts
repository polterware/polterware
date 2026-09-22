import type { ArchViewCard } from "@polterware/feature-architecture";
import type { Resource } from "@polterware/feature-resources";
import type { ProjectEnvironment, ProjectGeneralSettings } from "@polterware/feature-settings";
import type {
  EnvironmentItem,
  ProjectSwitcherItem,
  WorkspaceCatalogProvider,
  WorkspaceDocument,
  WorkspaceAdr,
  WorkspacePpmEntity,
  WorkspaceIntegration,
  WorkspaceIntegrationResource,
  WorkspaceIntegrationSyncRun,
  WorkspaceRunbook,
  WorkspaceSource,
  WorkspaceSpecFeature,
  WorkspaceConstitution,
} from "@polterware/feature-shell";
import { asNumber, asObjectArray, asRecord, asString } from "./format";
import {
  mapAdrs,
  mapConnections,
  mapDocuments,
  mapEnvironments,
  mapIntegrationResources,
  mapKnowledgeEntityLinks,
  mapPpmEntities,
  mapProjects,
  mapProviders,
  mapResources,
  mapRunbooks,
  mapSettings,
  mapSettingsEnvironments,
  mapConstitution,
  mapSources,
  mapSpecFeatures,
  mapSyncRuns,
  mapViews,
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
  documents: WorkspaceDocument[];
  adrs: WorkspaceAdr[];
  runbooks: WorkspaceRunbook[];
  sources: WorkspaceSource[];
  specFeatures: WorkspaceSpecFeature[];
  constitution: WorkspaceConstitution | null;
  ppmEntities: WorkspacePpmEntity[];
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
    documents: [],
    adrs: [],
    runbooks: [],
    sources: [],
    specFeatures: [],
    constitution: null,
    ppmEntities: [],
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

  const [settingsRes, layoutRes, resourcesRes, knowledgeRes, entitiesRes, linksRes, providersRes, connectionsRes, integrationResourcesRes, syncRunsRes, viewsRes] = await Promise.all([
    api.projects.getSettings({ projectId }),
    api.architecture.getLayout({ projectId }).catch(() => ({})),
    api.architecture.listResources({ projectId }),
    api.knowledge.list({ projectId }),
    api.ppm.listEntities({ projectId }).catch(() => ({ entities: [] })),
    api.knowledge.listLinks({ projectId }).catch(() => ({ links: [] })),
    api.integrations.listProviders(),
    api.integrations.listConnections({ projectId }),
    api.integrations.listResources({ projectId }),
    api.integrations.listSyncRuns({ projectId }),
    api.architecture.listViews({ projectId }),
  ]);

  const resources = mapResources(asObjectArray(asRecord(resourcesRes).resources), projectEnvironments);
  const knowledgeItems = asObjectArray(asRecord(knowledgeRes).items);
  const ppmEntities = mapPpmEntities(asObjectArray(asRecord(entitiesRes).entities));
  const linksByKnowledge = mapKnowledgeEntityLinks(asObjectArray(asRecord(linksRes).links), ppmEntities);
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
    settingsEnvironments: mapSettingsEnvironments(currentEnvRecords, resources),
    settings: mapSettings(asRecord(asRecord(settingsRes).settings), rawProjects.find((item) => asString(item.id) === projectId)),
    resources,
    documents: mapDocuments(knowledgeItems, linksByKnowledge),
    adrs: mapAdrs(knowledgeItems, linksByKnowledge),
    runbooks: mapRunbooks(knowledgeItems, linksByKnowledge),
    sources: mapSources(knowledgeItems, linksByKnowledge),
    specFeatures: mapSpecFeatures(knowledgeItems),
    constitution: mapConstitution(knowledgeItems),
    ppmEntities,
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
