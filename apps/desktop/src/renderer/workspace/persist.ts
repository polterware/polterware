import type { ProjectGeneralSettings } from "@polterware/feature-settings";
import type { WorkspaceCatalogProvider } from "@polterware/feature-shell";
import { asNumber, asRecord, asString } from "./format";
import type { WorkspaceSnapshot } from "./load";

function core() {
  return window.polterware.core;
}

function newId(prefix: string): string {
  return `${prefix}-${crypto.randomUUID().slice(0, 8)}`;
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

export async function persistProjectSettings(
  projectId: string,
  settings: ProjectGeneralSettings,
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
    params: { environment: input.environment },
  });
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

export async function resetLocalStore(): Promise<void> {
  await core().health.resetLocalStore({});
}
