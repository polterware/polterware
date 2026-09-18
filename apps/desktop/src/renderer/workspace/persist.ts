import type {
  EnvironmentKind,
  ProjectEnvironment,
  ProjectGeneralSettings,
  ProtectionLevel,
} from "@polterware/feature-settings";
import type { WorkspaceCatalogProvider } from "@polterware/feature-shell";
import { asNumber, asObjectArray, asRecord, asString } from "./format";
import type { WorkspaceSnapshot } from "./load";

function core() {
  return window.polterware.core;
}

function newId(prefix: string): string {
  return `${prefix}-${crypto.randomUUID().slice(0, 8)}`;
}

function protectionForKind(kind: string): ProtectionLevel {
  if (kind === "production") return "strict";
  if (kind === "staging") return "standard";
  return "relaxed";
}

async function syncEnvironmentDefaultFlags(
  projectId: string,
  environments: readonly ProjectEnvironment[],
  defaultEnvironmentId: string,
): Promise<void> {
  if (!defaultEnvironmentId || environments.length === 0) return;
  for (const environment of environments) {
    const isDefault = environment.id === defaultEnvironmentId;
    if (environment.isDefault === isDefault) continue;
    await core().projects.saveEnvironment({
      environment: {
        id: environment.id,
        projectId,
        name: environment.name,
        kind: environment.kind,
        protection: environment.protection,
        isDefault,
        description: environment.description ?? "",
      },
    });
  }
}

export async function persistLayout(
  projectId: string,
  bodyJson: string,
  expectedRevision: number,
): Promise<{ revision: number }> {
  const response = asRecord(await core().architecture.updateLayout({
    projectId,
    expectedRevision,
    bodyJson,
  }));
  return { revision: asNumber(asRecord(response.layout).revision) };
}

export async function persistView(input: {
  projectId: string;
  id?: string;
  name: string;
  description?: string;
  owner?: string;
  purpose?: string;
  notes?: string;
  filterSummary?: string;
  environments?: string[];
  tags?: string[];
  members: Array<{ nodeId: string; name: string; type?: string; layer?: string }>;
}): Promise<void> {
  await core().architecture.saveView({
    view: {
      id: input.id ?? newId("view"),
      projectId: input.projectId,
      name: input.name,
      description: input.description ?? "",
      owner: input.owner ?? "",
      purpose: input.purpose ?? "",
      notes: input.notes ?? "",
      filterSummary: input.filterSummary ?? "",
      environments: input.environments ?? [],
      tags: input.tags ?? [],
      members: input.members.map((member) => ({
        nodeId: member.nodeId,
        name: member.name,
        type: member.type ?? "",
        layer: member.layer ?? "",
      })),
    },
  });
}

export async function persistDeleteView(id: string): Promise<void> {
  await core().architecture.deleteView({ id });
}

export async function persistKnowledgeBody(
  workspace: WorkspaceSnapshot,
  projectId: string,
  itemId: string,
  body: string,
): Promise<void> {
  const current = workspace.rawKnowledge.find((item) => asString(item.id) === itemId);
  if (!current) return;
  await core().knowledge.save({
    item: {
      ...current,
      projectId,
      body,
    },
  });
}

export async function persistKnowledgeCreate(input: {
  projectId: string;
  kind: "doc" | "runbook";
  title: string;
}): Promise<string> {
  const id = newId(input.kind === "runbook" ? "rb" : "doc");
  await core().knowledge.save({
    item: {
      id,
      projectId: input.projectId,
      title: input.title,
      body: "",
      kind: input.kind,
      tags: [],
      summary: "",
      owner: "Platform",
      status: "draft",
      extraJson: input.kind === "runbook"
        ? JSON.stringify({
            service: "Service",
            severity: "medium",
            prerequisites: [],
            steps: [],
            commands: [],
            rollback: "",
          })
        : JSON.stringify({
            author: "Project maintainer",
            environment: "All",
            componentIds: [],
            componentNames: [],
          }),
    },
  });
  return id;
}

export async function persistKnowledgeDelete(id: string): Promise<void> {
  await core().knowledge.delete({ id });
}

export async function persistKnowledgeDuplicate(
  workspace: WorkspaceSnapshot,
  projectId: string,
  itemId: string,
): Promise<string | undefined> {
  const current = workspace.rawKnowledge.find((item) => asString(item.id) === itemId);
  if (!current) return undefined;
  const id = newId(asString(current.kind) === "runbook" ? "rb" : "doc");
  const title = `${asString(current.title)} (copy)`;
  await core().knowledge.save({
    item: {
      ...current,
      id,
      projectId,
      title,
    },
  });
  return id;
}

export async function persistProjectSettings(
  projectId: string,
  settings: ProjectGeneralSettings,
  environments: readonly ProjectEnvironment[] = [],
): Promise<void> {
  await core().projects.save({
    project: {
      id: projectId,
      name: settings.name,
    },
  });
  await core().projects.saveSettings({
    settings: {
      projectId,
      description: settings.description,
      defaultEnvironmentId: settings.defaultEnvironmentId,
      repository: settings.repository,
    },
  });
  await syncEnvironmentDefaultFlags(projectId, environments, settings.defaultEnvironmentId);
}

export async function persistEnvironment(input: {
  projectId: string;
  name: string;
  kind: EnvironmentKind;
  protection?: ProtectionLevel;
  description?: string;
  isDefault?: boolean;
  existingCount: number;
  settings: ProjectGeneralSettings;
}): Promise<string> {
  const id = newId("env");
  const isDefault = input.isDefault ?? input.existingCount === 0;
  const protection = input.protection ?? protectionForKind(input.kind);
  await core().projects.saveEnvironment({
    environment: {
      id,
      projectId: input.projectId,
      name: input.name,
      kind: input.kind,
      protection,
      isDefault,
      description: input.description ?? `${input.kind} environment`,
    },
  });
  if (isDefault) {
    await core().projects.saveSettings({
      settings: {
        projectId: input.projectId,
        description: input.settings.description,
        defaultEnvironmentId: id,
        repository: input.settings.repository,
      },
    });
  }
  return id;
}

export async function persistEnvironmentDefault(
  workspace: WorkspaceSnapshot,
  projectId: string,
  environmentId: string,
): Promise<void> {
  const environments = workspace.settingsEnvironments;
  if (!environments.some((item) => item.id === environmentId)) return;

  for (const environment of environments) {
    const isDefault = environment.id === environmentId;
    if (environment.isDefault === isDefault) continue;
    await core().projects.saveEnvironment({
      environment: {
        id: environment.id,
        projectId,
        name: environment.name,
        kind: environment.kind,
        protection: environment.protection,
        isDefault,
        description: environment.description ?? "",
      },
    });
  }

  await core().projects.saveSettings({
    settings: {
      projectId,
      description: workspace.settings.description,
      defaultEnvironmentId: environmentId,
      repository: workspace.settings.repository,
    },
  });
}

export async function persistDeleteEnvironment(projectId: string, environmentId: string): Promise<void> {
  await core().projects.deleteEnvironment({ id: environmentId, projectId });
}

export async function persistArchiveProject(projectId: string): Promise<void> {
  await core().projects.archive({ id: projectId });
}

export async function persistCreatedProject(input: {
  name: string;
  directory: string;
  environments: string[];
}): Promise<{ projectId: string; environmentId?: string }> {
  const projectId = newId("proj");
  await core().projects.save({
    project: {
      id: projectId,
      name: input.name,
      rootPath: input.directory,
    },
  });
  let firstEnvironmentId: string | undefined;
  for (const [index, kind] of input.environments.entries()) {
    const environmentId = `${projectId}-${kind}`;
    await core().projects.saveEnvironment({
      environment: {
        id: environmentId,
        projectId,
        name: kind === "production" ? "main" : kind,
        kind,
        protection: kind === "production" ? "strict" : kind === "staging" ? "standard" : "relaxed",
        isDefault: index === 0,
        description: `${kind} environment`,
      },
    });
    firstEnvironmentId ??= environmentId;
  }
  await core().projects.saveSettings({
    settings: {
      projectId,
      description: "",
      defaultEnvironmentId: firstEnvironmentId ?? "",
      repository: "",
    },
  });
  return { projectId, environmentId: firstEnvironmentId };
}

export async function persistResource(input: {
  projectId: string;
  environmentId?: string;
  name: string;
  type: string;
}): Promise<void> {
  await core().architecture.saveResource({
    resource: {
      id: newId("res"),
      projectId: input.projectId,
      environmentId: input.environmentId ?? "",
      name: input.name,
      type: input.type,
      source: "manual",
      provider: "local",
      status: "unknown",
      region: "local",
      tags: [],
      description: "",
    },
  });
}

export async function persistNode(input: {
  projectId: string;
  environmentId?: string;
  name: string;
  platform?: string;
  shell?: string;
}): Promise<void> {
  await core().nodes.save({
    node: {
      id: newId("node"),
      projectId: input.projectId,
      environmentId: input.environmentId ?? "",
      name: input.name,
      platform: input.platform ?? "local",
      shell: input.shell ?? "bash",
      status: "healthy",
      address: "",
      cpu: "",
      memory: "",
      lastSeenAt: new Date().toISOString(),
    },
  });
}

export async function persistStartProcess(input: {
  projectId: string;
  environmentId?: string;
  name?: string;
  command: string;
  args?: string[];
  cwd?: string;
  nodeId?: string;
}): Promise<void> {
  const parts = input.command.trim().split(/\s+/).filter(Boolean);
  const command = parts[0] ?? "";
  const args = input.args ?? parts.slice(1);
  await core().processes.start({
    projectId: input.projectId,
    environmentId: input.environmentId ?? "",
    name: input.name ?? "",
    command,
    args,
    cwd: input.cwd ?? "",
    nodeId: input.nodeId ?? "",
  });
}

export async function persistStopProcess(id: string): Promise<void> {
  await core().processes.stop({ id });
}

export async function persistOperationPreview(input: {
  projectId: string;
  environmentId?: string;
  target: string;
  command: string;
  cwd: string;
  environment: string;
}): Promise<void> {
  await core().operations.request({
    type: "command",
    projectId: input.projectId,
    environmentId: input.environmentId ?? "",
    cwd: input.cwd,
    title: input.command,
    target: input.target,
    command: input.command,
    params: { environment: input.environment, cwd: input.cwd },
  });
}

export async function persistWorkflowRun(input: {
  workflowId: string;
  environmentId?: string;
}): Promise<void> {
  await core().workflows.run({
    id: input.workflowId,
    environmentId: input.environmentId ?? "",
  });
}

export async function persistRunbookRun(input: {
  projectId: string;
  environmentId?: string;
  runbookId: string;
  title: string;
  body: string;
}): Promise<void> {
  const command = extractRunbookCommand(input.body) ?? `echo "Runbook ${input.title}"`;
  await core().operations.request({
    type: "runbook",
    projectId: input.projectId,
    environmentId: input.environmentId ?? "",
    title: `Runbook: ${input.title}`,
    target: input.runbookId,
    command,
    params: { runbookId: input.runbookId },
  });
}

function extractRunbookCommand(body: string): string | undefined {
  const fence = body.match(/```(?:bash|sh|shell)?\n([\s\S]*?)```/);
  if (!fence) return undefined;
  const lines = fence[1]
    .split("\n")
    .map((line) => line.trim())
    .filter((line) => line && !line.startsWith("#"));
  return lines[0];
}

export async function persistWorkflow(input: {
  projectId: string;
  name: string;
  description?: string;
}): Promise<string> {
  const id = newId("wf");
  await core().workflows.save({
    workflow: {
      id,
      projectId: input.projectId,
      name: input.name,
      description: input.description ?? "",
      status: "draft",
      trigger: "manual",
      steps: [],
      lastRunAt: "",
    },
  });
  return id;
}

export async function persistSchedule(input: {
  projectId: string;
  environmentId?: string;
  name: string;
  workflowId: string;
  expression: string;
  timezone?: string;
}): Promise<string> {
  const id = newId("sch");
  await core().operations.saveSchedule({
    schedule: {
      id,
      projectId: input.projectId,
      environmentId: input.environmentId ?? "",
      name: input.name,
      description: "",
      status: "active",
      workflowId: input.workflowId,
      expression: input.expression,
      timezone: input.timezone ?? "UTC",
      nextRunAt: "",
      lastRunAt: "",
    },
  });
  return id;
}

export async function persistIntegrationConnection(input: {
  projectId: string;
  environmentId?: string;
  provider: WorkspaceCatalogProvider;
}): Promise<void> {
  await core().integrations.saveConnection({
    connection: {
      id: newId("int"),
      projectId: input.projectId,
      providerId: input.provider.id,
      status: "connected",
      label: input.provider.name,
      environmentId: input.environmentId ?? "",
      summary: `Local ${input.provider.name} connection`,
      cliCommand: input.provider.binary,
      freshness: "unknown",
    },
  });
}

export async function persistIntegrationSync(input: {
  connectionId: string;
}): Promise<{ result: string; message: string }> {
  const response = asRecord(await core().integrations.syncConnection({
    connectionId: input.connectionId,
  }));
  const run = asRecord(response.run);
  return {
    result: asString(run.result, "failed"),
    message: asString(run.message, "CLI inventory sync finished."),
  };
}

export async function resetLocalStore(): Promise<void> {
  await core().health.resetLocalStore({});
}
