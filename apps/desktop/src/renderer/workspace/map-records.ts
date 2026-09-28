import type { ArchViewCard } from "@polterware/feature-architecture";
import type { Resource, ResourceStatus } from "@polterware/feature-resources";
import type { ProjectEnvironment, ProjectGeneralSettings, ProtectionLevel } from "@polterware/feature-settings";
import type {
  EnvironmentItem,
  EnvironmentKind,
  ProjectSwitcherItem,
  WorkspaceCatalogProvider,
  WorkspaceDocument,
  WorkspaceAdr,
  WorkspaceConstitution,
  WorkspaceEntityLink,
  WorkspacePpmEntity,
  WorkspaceIntegration,
  WorkspaceIntegrationResource,
  WorkspaceIntegrationSyncRun,
  WorkspaceRunbook,
  WorkspaceSource,
  WorkspaceWorkCard,
  WorkStage,
} from "@polterware/feature-shell";
import {
  asBoolean,
  asNumber,
  asObjectArray,
  asRecord,
  asString,
  asStringArray,
  formatInstant,
  parseJsonObject,
} from "./format";

export function mapProjects(
  projects: Record<string, unknown>[],
  environments: EnvironmentItem[],
  currentProjectId?: string,
): ProjectSwitcherItem[] {
  return projects.map((project) => {
    const projectId = asString(project.id);
    const projectEnvs = environments.filter((item) => item.projectId === projectId);
    const defaultEnv = projectEnvs.find((item) => item.kind === "production") ?? projectEnvs[0];
    return {
      projectId,
      name: asString(project.name, projectId),
      directory: asString(project.rootPath),
      environmentSummary: defaultEnv?.name ?? "No environments",
      isCurrent: projectId === currentProjectId,
      status: "active" as const,
      createdAt: formatInstant(asString(project.createdAt)),
    };
  });
}

export function mapEnvironments(items: Record<string, unknown>[]): EnvironmentItem[] {
  return items.flatMap((item) => {
    const kind = asString(item.kind);
    if (!isEnvironmentKind(kind)) return [];
    return [{
      id: asString(item.id),
      name: asString(item.name),
      kind,
      projectId: asString(item.projectId),
    }];
  });
}

export function mapSettingsEnvironments(
  items: Record<string, unknown>[],
  resources: Resource[],
): ProjectEnvironment[] {
  return items.flatMap((item) => {
    const kind = asString(item.kind);
    if (!isEnvironmentKind(kind)) return [];
    const id = asString(item.id);
    return [{
      id,
      name: asString(item.name),
      kind,
      protection: asProtection(asString(item.protection)),
      resourceCount: resources.filter((resource) => resource.environment === asString(item.name)).length,
      nodeCount: 0,
      isDefault: asBoolean(item.isDefault),
      description: asString(item.description),
      protectionRules: protectionRules(asProtection(asString(item.protection))),
      dataSource: "empty" as const,
    }];
  });
}

export function mapSettings(
  settings: Record<string, unknown>,
  project: Record<string, unknown> | undefined,
): ProjectGeneralSettings {
  return {
    name: asString(project?.name, asString(settings.projectId, "Project")),
    description: asString(settings.description),
    defaultEnvironmentId: asString(settings.defaultEnvironmentId),
    projectId: asString(settings.projectId),
    repository: asString(settings.repository),
    dataSource: "empty",
  };
}

export function mapResources(
  items: Record<string, unknown>[],
  environments: EnvironmentItem[],
): Resource[] {
  const names = new Map(environments.map((item) => [item.id, item.name]));
  return items.map((item) => ({
    id: asString(item.id),
    name: asString(item.name),
    type: asString(item.type),
    provider: asString(item.provider) || asString(item.source),
    environment: names.get(asString(item.environmentId)) ?? "All",
    status: asResourceStatus(asString(item.status)),
    region: asString(item.region),
    address: asString(item.address) || undefined,
    tags: asStringArray(item.tags),
    description: asString(item.description),
    dataSource: "core",
  }));
}

export function mapPpmEntities(items: Record<string, unknown>[]): WorkspacePpmEntity[] {
  return items.map((item) => ({
    id: asString(item.id),
    kind: asString(item.kind),
    refKind: asString(item.refKind),
    refId: asString(item.refId),
    title: asString(item.title),
    truthClass: asString(item.truthClass, "declared"),
    source: asString(item.source, "polterware"),
  }));
}

export function mapKnowledgeEntityLinks(
  links: Record<string, unknown>[],
  entities: WorkspacePpmEntity[],
): Map<string, WorkspaceEntityLink[]> {
  const byId = new Map(entities.map((entity) => [entity.id, entity]));
  const grouped = new Map<string, WorkspaceEntityLink[]>();
  for (const item of links) {
    const knowledgeId = asString(item.knowledgeId);
    const entityId = asString(item.entityId);
    const entity = byId.get(entityId);
    const link: WorkspaceEntityLink = {
      id: asString(item.id),
      entityId,
      entityTitle: entity?.title ?? entityId,
      entityKind: entity?.kind ?? "",
      entityRefId: entity?.refId ?? "",
      predicate: asString(item.predicate),
      truthClass: asString(item.truthClass, "declared"),
      source: asString(item.source, "polterware"),
    };
    const current = grouped.get(knowledgeId) ?? [];
    current.push(link);
    grouped.set(knowledgeId, current);
  }
  return grouped;
}

export function mapDocuments(
  items: Record<string, unknown>[],
  linksByKnowledge: Map<string, WorkspaceEntityLink[]> = new Map(),
): WorkspaceDocument[] {
  return items.filter((item) => asString(item.kind) === "doc").map((item) => {
    const extra = parseJsonObject(asString(item.extraJson));
    const id = asString(item.id);
    const entityLinks = linksByKnowledge.get(id) ?? [];
    const componentIds = entityLinks
      .filter((link) => link.entityKind === "component")
      .map((link) => link.entityRefId);
    const componentNames = entityLinks
      .filter((link) => link.entityKind === "component")
      .map((link) => link.entityTitle);
    return {
      id,
      title: asString(item.title),
      summary: asString(item.summary),
      bodyMarkdown: asString(item.body),
      author: asString(extra.author, asString(item.owner, "Project maintainer")),
      updatedAt: formatInstant(asString(item.updatedAt)),
      tags: asStringArray(item.tags),
      componentIds: componentIds.length > 0 ? componentIds : asStringArray(extra.componentIds),
      componentNames: componentNames.length > 0 ? componentNames : asStringArray(extra.componentNames),
      environment: asString(extra.environment, "All"),
      status: asString(item.status) === "draft" ? "draft" : "published",
      owner: asString(item.owner, "Platform"),
      entityLinks,
    };
  });
}

export function mapAdrs(
  items: Record<string, unknown>[],
  linksByKnowledge: Map<string, WorkspaceEntityLink[]> = new Map(),
): WorkspaceAdr[] {
  return items.filter((item) => asString(item.kind) === "adr").map((item) => {
    const id = asString(item.id);
    return {
      id,
      title: asString(item.title),
      summary: asString(item.summary),
      bodyMarkdown: asString(item.body),
      updatedAt: formatInstant(asString(item.updatedAt)),
      tags: asStringArray(item.tags),
      status: asString(item.status, "accepted"),
      owner: asString(item.owner, "Platform"),
      entityLinks: linksByKnowledge.get(id) ?? [],
    };
  });
}

export function mapRunbooks(
  items: Record<string, unknown>[],
  linksByKnowledge: Map<string, WorkspaceEntityLink[]> = new Map(),
): WorkspaceRunbook[] {
  return items.filter((item) => asString(item.kind) === "runbook").map((item) => {
    const extra = parseJsonObject(asString(item.extraJson));
    const id = asString(item.id);
    const entityLinks = linksByKnowledge.get(id) ?? [];
    const componentLink = entityLinks.find((link) => link.entityKind === "component");
    const resourceLink = entityLinks.find((link) => link.entityKind === "resource");
    return {
      id,
      title: asString(item.title),
      objective: asString(item.summary),
      bodyMarkdown: asString(item.body),
      service: asString(extra.service, "Service"),
      severity: asSeverity(asString(extra.severity)),
      prerequisites: asStringArray(extra.prerequisites),
      steps: asStringArray(extra.steps),
      commands: asStringArray(extra.commands),
      rollback: asString(extra.rollback),
      owner: asString(item.owner, "Platform"),
      lastValidated: formatInstant(asString(extra.lastValidated) || asString(item.updatedAt)),
      componentId: componentLink?.entityRefId || asString(extra.componentId) || undefined,
      componentName: componentLink?.entityTitle || asString(extra.componentName) || undefined,
      resourceId: resourceLink?.entityRefId || asString(extra.resourceId) || undefined,
      resourceName: resourceLink?.entityTitle || asString(extra.resourceName) || undefined,
      entityLinks,
    };
  });
}

export function mapSources(
  items: Record<string, unknown>[],
  linksByKnowledge: Map<string, WorkspaceEntityLink[]> = new Map(),
): WorkspaceSource[] {
  return items.filter((item) => asString(item.kind) === "source").map((item) => {
    const extra = parseJsonObject(asString(item.extraJson));
    const id = asString(item.id);
    return {
      id,
      title: asString(item.title),
      path: asString(extra.path),
      summary: asString(item.summary),
      bodyMarkdown: asString(item.body),
      updatedAt: formatInstant(asString(item.updatedAt)),
      owner: asString(item.owner, "Platform"),
      entityLinks: linksByKnowledge.get(id) ?? [],
    };
  });
}

export function mapConstitution(items: Record<string, unknown>[]): WorkspaceConstitution | null {
  const item = items.find((entry) => asString(entry.kind) === "constitution");
  if (!item) return null;
  return {
    id: asString(item.id),
    title: asString(item.title, "Project constitution"),
    bodyMarkdown: asString(item.body),
    updatedAt: formatInstant(asString(item.updatedAt)),
    owner: asString(item.owner, "Platform"),
    status: asString(item.status, "active"),
  };
}

function asWorkStage(value: string): WorkStage {
  return value === "plan"
    || value === "build"
    || value === "verify"
    || value === "delivered"
    ? value
    : "shape";
}

/** Active Work cards from WorkService.ListWorkCards, in board position order. */
export function mapWorkCards(items: Record<string, unknown>[]): WorkspaceWorkCard[] {
  return items.map((item) => ({
    id: asString(item.id),
    slug: asString(item.slug),
    title: asString(item.title, asString(item.slug)),
    intent: asString(item.intent),
    stage: asWorkStage(asString(item.stage)),
    owner: asString(item.owner),
    under: asString(item.under),
    decisions: asStringArray(item.decisions),
    updatedAt: formatInstant(asString(item.updatedAt)),
    tasks: asObjectArray(item.tasks).map((task) => {
      const state = asString(task.state);
      return {
        id: asString(task.id),
        number: asNumber(task.number),
        title: asString(task.title),
        state: state === "done" || state === "dropped" ? state : "open",
        scope: asString(task.scope),
      };
    }),
    questions: asObjectArray(item.questions).map((question) => ({
      id: asString(question.id),
      text: asString(question.text),
      blocking: asBoolean(question.blocking),
      options: asStringArray(question.options),
      state: asString(question.state) === "answered" ? "answered" : "open",
      answer: asString(question.answer),
    })),
    invariants: asObjectArray(item.invariants).map((invariant) => ({
      id: asString(invariant.id),
      text: asString(invariant.text),
    })),
  }));
}

export function mapProviders(
  items: Record<string, unknown>[],
  connections: WorkspaceIntegration[],
): WorkspaceCatalogProvider[] {
  const connected = new Set(connections.map((item) => item.providerId));
  return items.map((item) => {
    const id = asString(item.id);
    const authCommands = asStringArray(item.authCommands);
    return {
      id,
      name: asString(item.name),
      description: asString(item.description),
      category: asCategory(asString(item.category)),
      binary: asString(item.binary),
      authCommands,
      transcript: authCommands.map((command) => `$ ${command}`),
      state: connected.has(id) ? "connected" : "ready",
    };
  });
}

export function mapConnections(
  items: Record<string, unknown>[],
  providers: Record<string, unknown>[],
  environments: EnvironmentItem[],
  resources: WorkspaceIntegrationResource[],
): WorkspaceIntegration[] {
  const providerNames = new Map(providers.map((item) => [asString(item.id), asString(item.name)]));
  const envNames = new Map(environments.map((item) => [item.id, item.name]));
  return items.map((item) => {
    const id = asString(item.id);
    const linked = resources.filter((resource) => resource.integrationId === id);
    return {
      id,
      providerId: asString(item.providerId),
      name: providerNames.get(asString(item.providerId)) ?? asString(item.providerId),
      provider: providerNames.get(asString(item.providerId)) ?? asString(item.providerId),
      status: asIntegrationStatus(asString(item.status)),
      connectionLabel: asString(item.label),
      environment: envNames.get(asString(item.environmentId)) ?? "All",
      lastSync: formatInstant(asString(item.lastSyncAt)),
      summary: asString(item.summary),
      resourceCount: linked.length,
      resourceIds: linked.flatMap((resource) => resource.inventoryResourceId ? [resource.inventoryResourceId] : []),
      cliCommand: asString(item.cliCommand),
      syncInterval: asString(item.syncInterval) || undefined,
      freshness: asFreshness(asString(item.freshness)),
      error: asString(item.errorMessage) || undefined,
    };
  });
}

export function mapIntegrationResources(
  items: Record<string, unknown>[],
  environments: EnvironmentItem[],
): WorkspaceIntegrationResource[] {
  const envNames = new Map(environments.map((item) => [item.id, item.name]));
  return items.map((item) => ({
    id: asString(item.id),
    integrationId: asString(item.connectionId),
    name: asString(item.name),
    kind: asString(item.kind),
    externalId: asString(item.externalId),
    environment: envNames.get(asString(item.environmentId)) ?? "All",
    status: asString(item.status),
    inventoryResourceId: asString(item.inventoryResourceId) || undefined,
  }));
}

export function mapSyncRuns(items: Record<string, unknown>[]): WorkspaceIntegrationSyncRun[] {
  return items.map((item) => ({
    id: asString(item.id),
    integrationId: asString(item.connectionId),
    startedAt: formatInstant(asString(item.startedAt)),
    finishedAt: formatInstant(asString(item.finishedAt)),
    result: asSyncResult(asString(item.result)),
    imported: asNumber(item.imported),
    changed: asNumber(item.changed),
    message: asString(item.message),
  }));
}

export function mapViews(items: Record<string, unknown>[]): ArchViewCard[] {
  return items.map((item) => {
    const members = asObjectArray(item.members).map((member) => ({
      id: asString(member.nodeId),
      name: asString(member.name),
      type: asString(member.type) || undefined,
      layer: asString(member.layer) || undefined,
    }));
    return {
      id: asString(item.id),
      name: asString(item.name),
      description: asString(item.description),
      nodeCount: members.length,
      updated: formatInstant(asString(item.updatedAt)),
      owner: asString(item.owner) || undefined,
      environments: asStringArray(item.environments),
      tags: asStringArray(item.tags),
      purpose: asString(item.purpose) || undefined,
      members: members.length > 0 ? members : undefined,
      notes: asString(item.notes) || undefined,
      filterSummary: asString(item.filterSummary) || undefined,
    };
  });
}

function isEnvironmentKind(value: string): value is EnvironmentKind {
  return value === "production" || value === "staging" || value === "development" || value === "local";
}

function asProtection(value: string): ProtectionLevel {
  if (value === "strict" || value === "relaxed") return value;
  return "standard";
}

function protectionRules(level: ProtectionLevel): string[] {
  if (level === "strict") {
    return ["Changes persist locally", "Freeze window: Fri 18:00 – Mon 08:00 UTC", "Destructive commands are not executed"];
  }
  if (level === "relaxed") {
    return ["Local changes are allowed", "No remote commands are executed"];
  }
  return ["Changes persist locally", "Validation results stay in the local store"];
}

function asResourceStatus(value: string): ResourceStatus {
  if (value === "healthy" || value === "degraded" || value === "offline") return value;
  return "unknown";
}

function asCategory(value: string): WorkspaceCatalogProvider["category"] {
  if (value === "Cloud" || value === "Deployment" || value === "Databases" || value === "Infrastructure") return value;
  return "Source Control";
}

function asIntegrationStatus(value: string): WorkspaceIntegration["status"] {
  if (value === "error" || value === "disconnected" || value === "disabled") return value;
  return "connected";
}

function asFreshness(value: string): WorkspaceIntegration["freshness"] {
  if (value === "stale" || value === "unknown") return value;
  return "fresh";
}

function asSyncResult(value: string): WorkspaceIntegrationSyncRun["result"] {
  if (value === "partial" || value === "failed" || value === "running") return value;
  return "success";
}

function asSeverity(value: string): WorkspaceRunbook["severity"] {
  if (value === "low" || value === "medium" || value === "critical") return value;
  return "high";
}
