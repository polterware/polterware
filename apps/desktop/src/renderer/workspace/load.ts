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

type ProjectMetadata = {
  rawProjects: Record<string, unknown>[];
  rawEnvironments: Record<string, unknown>[];
  environments: EnvironmentItem[];
};

export type ProjectCatalog = ProjectMetadata & {
  projects: ProjectSwitcherItem[];
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

async function loadProjectMetadata(api = core()): Promise<ProjectMetadata> {
  const listed = asRecord(await api.projects.list());
  const rawProjects = asObjectArray(listed.projects);
  const environmentResults = await Promise.all(rawProjects.map(async (project) => {
    const listedEnvs = asRecord(await api.projects.listEnvironments({ projectId: asString(project.id) }));
    return asObjectArray(listedEnvs.environments);
  }));
  const rawEnvironments = environmentResults.flat();
  const environments = mapEnvironments(rawEnvironments);
  return { rawProjects, rawEnvironments, environments };
}

export async function loadProjects(currentProjectId?: string): Promise<ProjectCatalog> {
  const metadata = await loadProjectMetadata();
  return {
    ...metadata,
    projects: mapProjects(metadata.rawProjects, metadata.environments, currentProjectId),
  };
}

export async function loadWorkspace(
  projectId: string,
  catalog?: ProjectCatalog,
): Promise<WorkspaceSnapshot> {
  const api = core();
  const metadataPromise: Promise<ProjectMetadata> = catalog
    ? Promise.resolve(catalog)
    : loadProjectMetadata(api);
  const [
    healthRes,
    settingsRes,
    layoutRes,
    resourcesRes,
    knowledgeRes,
    entitiesRes,
    linksRes,
    providersRes,
    connectionsRes,
    integrationResourcesRes,
    syncRunsRes,
    viewsRes,
  ] = await Promise.all([
    api.health.getStoreHealth(),
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
  ] as const);

  const metadata = await metadataPromise;
  const health = asRecord(healthRes);
  const { rawProjects, rawEnvironments, environments } = metadata;
  const projectEnvironments = environments.filter((item) => item.projectId === projectId);
  const currentEnvRecords = rawEnvironments.filter((item) => asString(item.projectId) === projectId);
  // Architecture + project shell first — needed for the default route paint.
  const resources = mapResources(asObjectArray(asRecord(resourcesRes).resources), projectEnvironments);
  const layout = asRecord(asRecord(layoutRes).layout);
  const projects = mapProjects(rawProjects, environments, projectId);
  const settings = mapSettings(
    asRecord(asRecord(settingsRes).settings),
    rawProjects.find((item) => asString(item.id) === projectId),
  );
  const settingsEnvironments = mapSettingsEnvironments(currentEnvRecords, resources);
  const views = mapViews(asObjectArray(asRecord(viewsRes).views));

  // Yield between map groups so the post-Promise.all burst is not one long task.
  await Promise.resolve();

  const knowledgeItems = asObjectArray(asRecord(knowledgeRes).items);
  const ppmEntities = mapPpmEntities(asObjectArray(asRecord(entitiesRes).entities));
  const linksByKnowledge = mapKnowledgeEntityLinks(asObjectArray(asRecord(linksRes).links), ppmEntities);
  const documents = mapDocuments(knowledgeItems, linksByKnowledge);
  const adrs = mapAdrs(knowledgeItems, linksByKnowledge);
  const runbooks = mapRunbooks(knowledgeItems, linksByKnowledge);
  const sources = mapSources(knowledgeItems, linksByKnowledge);
  const specFeatures = mapSpecFeatures(knowledgeItems);
  const constitution = mapConstitution(knowledgeItems);

  await Promise.resolve();

  const integrationResources = mapIntegrationResources(
    asObjectArray(asRecord(integrationResourcesRes).resources),
    projectEnvironments,
  );
  const providersRaw = asObjectArray(asRecord(providersRes).providers);
  const integrations = mapConnections(
    asObjectArray(asRecord(connectionsRes).connections),
    providersRaw,
    projectEnvironments,
    integrationResources,
  );
  const providers = mapProviders(providersRaw, integrations);
  const syncRuns = mapSyncRuns(asObjectArray(asRecord(syncRunsRes).runs));

  return {
    available: true,
    quarantined: health.quarantined === true,
    healthMessage: asString(health.message),
    projects,
    environments,
    settingsEnvironments,
    settings,
    resources,
    documents,
    adrs,
    runbooks,
    sources,
    specFeatures,
    constitution,
    ppmEntities,
    providers,
    integrations,
    integrationResources,
    syncRuns,
    views,
    layoutBody: asString(layout.bodyJson),
    layoutRevision: asNumber(layout.revision),
    rawKnowledge: knowledgeItems,
  };
}
