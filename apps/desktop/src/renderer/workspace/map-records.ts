import type { ArchViewCard } from "@polterware/feature-architecture";
import type { OperationRun, OperationSchedule, OperationWorkflow, RunStatus, RunStep } from "@polterware/feature-operations";
import type { NodeFixture, ProcessFixture, Resource, ResourceStatus } from "@polterware/feature-resources";
import type { ProjectEnvironment, ProjectGeneralSettings, ProtectionLevel } from "@polterware/feature-settings";
import type {
  EnvironmentItem,
  EnvironmentKind,
  ProjectSwitcherItem,
  TerminalTargetItem,
  TerminalTargetStatus,
  WorkspaceCatalogProvider,
  WorkspaceDocument,
  WorkspaceIntegration,
  WorkspaceIntegrationResource,
  WorkspaceIntegrationSyncRun,
  WorkspaceRunbook,
} from "@polterware/feature-shell";
import {
  asBoolean,
  asNumber,
  asObjectArray,
  asRecord,
  asString,
  asStringArray,
  formatDuration,
  formatInstant,
  parseJsonArray,
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
  nodes: NodeFixture[],
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
      nodeCount: nodes.filter((node) => node.environment === asString(item.name)).length,
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

export function mapNodes(
  items: Record<string, unknown>[],
  environments: EnvironmentItem[],
): NodeFixture[] {
  const names = new Map(environments.map((item) => [item.id, item.name]));
  return items.map((item) => ({
    id: asString(item.id),
    name: asString(item.name),
    platform: asString(item.platform),
    environment: names.get(asString(item.environmentId)) ?? "All",
    status: asResourceStatus(asString(item.status)),
    address: asString(item.address),
    cpu: asString(item.cpu),
    memory: asString(item.memory),
    lastSeen: formatInstant(asString(item.lastSeenAt)),
  }));
}

export function mapTerminalTargets(nodes: Record<string, unknown>[]): TerminalTargetItem[] {
  return nodes.map((item) => ({
    id: asString(item.id),
    name: asString(item.name),
    shell: asString(item.shell, "bash"),
    status: asTerminalStatus(asString(item.status)),
    environmentId: asString(item.environmentId) || undefined,
  }));
}

export function mapProcesses(
  items: Record<string, unknown>[],
  nodes: NodeFixture[],
): ProcessFixture[] {
  const names = new Map(nodes.map((item) => [item.id, item.name]));
  return items.map((item) => ({
    id: asString(item.id),
    name: asString(item.name) || asString(item.command),
    nodeId: asString(item.nodeId),
    nodeName: names.get(asString(item.nodeId)) ?? (asString(item.nodeId) || "—"),
    status: asProcessStatus(asString(item.status)),
    pid: asNumber(item.pid) || undefined,
    command: asString(item.command),
    startedAt: formatInstant(asString(item.startedAt)),
  }));
}

export function mapDocuments(items: Record<string, unknown>[]): WorkspaceDocument[] {
  return items.filter((item) => asString(item.kind) === "doc").map((item) => {
    const extra = parseJsonObject(asString(item.extraJson));
    return {
      id: asString(item.id),
      title: asString(item.title),
      summary: asString(item.summary),
      bodyMarkdown: asString(item.body),
      author: asString(extra.author, asString(item.owner, "Project maintainer")),
      updatedAt: formatInstant(asString(item.updatedAt)),
      tags: asStringArray(item.tags),
      componentIds: asStringArray(extra.componentIds),
      componentNames: asStringArray(extra.componentNames),
      environment: asString(extra.environment, "All"),
      status: asString(item.status) === "draft" ? "draft" : "published",
      owner: asString(item.owner, "Platform"),
    };
  });
}

export function mapRunbooks(items: Record<string, unknown>[]): WorkspaceRunbook[] {
  return items.filter((item) => asString(item.kind) === "runbook").map((item) => {
    const extra = parseJsonObject(asString(item.extraJson));
    return {
      id: asString(item.id),
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
      componentId: asString(extra.componentId) || undefined,
      componentName: asString(extra.componentName) || undefined,
      resourceId: asString(extra.resourceId) || undefined,
      resourceName: asString(extra.resourceName) || undefined,
      operationId: asString(extra.operationId) || undefined,
    };
  });
}

export function mapRuns(
  items: Record<string, unknown>[],
  environments: EnvironmentItem[],
  logsByOperation: Record<string, string[]>,
): OperationRun[] {
  const names = new Map(environments.map((item) => [item.id, item.name]));
  return items.map((item) => {
    const id = asString(item.id);
    const startedAt = asString(item.startedAt) || asString(item.requestedAt);
    const status = mapRunStatus(asString(item.status));
    return {
      id,
      title: asString(item.title) || asString(item.type, "Operation"),
      target: asString(item.target) || "local",
      status,
      startedAt: formatInstant(startedAt),
      duration: formatDuration(startedAt, asString(item.finishedAt)),
      environment: names.get(asString(item.environmentId)) ?? "All",
      command: asString(item.command),
      steps: parseJsonArray(asString(item.stepsJson)).map((step) => ({
        id: asString(step.id),
        name: asString(step.name),
        status: asStepStatus(asString(step.status)),
        duration: asString(step.duration) || undefined,
      } satisfies RunStep)),
      outputLines: logsByOperation[id] ?? [],
      result: asString(item.resultSummary),
      dataSource: "core",
    };
  });
}

export function mapWorkflows(items: Record<string, unknown>[]): OperationWorkflow[] {
  return items.map((item) => ({
    id: asString(item.id),
    name: asString(item.name),
    description: asString(item.description),
    status: asWorkflowStatus(asString(item.status)),
    trigger: asString(item.trigger) === "schedule" ? "schedule" : "manual",
    steps: asObjectArray(item.steps).map((step) => ({
      id: asString(step.id),
      label: asString(step.name),
      type: asWorkflowStepType(asString(step.operationType)),
      command: asString(asRecord(step.params).command) || undefined,
    })),
    lastRun: formatInstant(asString(item.lastRunAt)),
    dataSource: "core",
  }));
}

export function mapSchedules(
  items: Record<string, unknown>[],
  workflows: OperationWorkflow[],
  environments: EnvironmentItem[],
): OperationSchedule[] {
  const workflowNames = new Map(workflows.map((item) => [item.id, item.name]));
  const envNames = new Map(environments.map((item) => [item.id, item.name]));
  return items.map((item) => ({
    id: asString(item.id),
    name: asString(item.name),
    description: asString(item.description),
    status: asScheduleStatus(asString(item.status)),
    workflowId: asString(item.workflowId),
    workflowName: workflowNames.get(asString(item.workflowId)) ?? asString(item.workflowId),
    expression: asString(item.expression),
    timezone: asString(item.timezone, "UTC"),
    nextRun: formatInstant(asString(item.nextRunAt)),
    lastRun: formatInstant(asString(item.lastRunAt)),
    environment: envNames.get(asString(item.environmentId)) ?? "All",
    dataSource: "core",
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

function asTerminalStatus(value: string): TerminalTargetStatus {
  if (value === "healthy" || value === "online") return "online";
  if (value === "degraded") return "degraded";
  if (value === "offline") return "offline";
  if (value === "revoked") return "revoked";
  return "unknown";
}

function asProcessStatus(value: string): ProcessFixture["status"] {
  if (value === "running" || value === "failed") return value;
  return "stopped";
}

function mapRunStatus(value: string): RunStatus {
  if (value === "succeeded" || value === "success") return "success";
  if (value === "running" || value === "failed" || value === "cancelled" || value === "queued") return value;
  return "queued";
}

function asStepStatus(value: string): RunStep["status"] {
  if (value === "running" || value === "success" || value === "failed" || value === "skipped") return value;
  return "queued";
}

function asWorkflowStatus(value: string): OperationWorkflow["status"] {
  if (value === "draft" || value === "disabled") return value;
  return "ready";
}

function asWorkflowStepType(value: string): OperationWorkflow["steps"][number]["type"] {
  if (value === "validate" || value === "build" || value === "deploy" || value === "health_check" || value === "notify") {
    return value;
  }
  return "command";
}

function asScheduleStatus(value: string): OperationSchedule["status"] {
  if (value === "paused" || value === "disabled") return value;
  return "active";
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
