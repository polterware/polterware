import type {
  EnvironmentKind,
  ProjectEnvironment,
  ProjectGeneralSettings,
  ProtectionLevel,
} from "@polterware/feature-settings";
import type { WorkspaceCatalogProvider, WorkStage } from "@polterware/feature-shell";
import type { ChangeEnvelope } from "@polterware/feature-knowledge";
import { asNumber, asObjectArray, asRecord, asString, parseJsonObject } from "./format";
import type { WorkspaceSnapshot } from "./load";
import {
  checklistHasOpenItems,
  uniqueFeatureKey as nextFeatureKey,
  CHECKLIST_BODY_TEMPLATE,
  CLARIFY_BODY_TEMPLATE,
  CONSTITUTION_BODY_TEMPLATE,
  PLAN_BODY_TEMPLATE,
  SPEC_BODY_TEMPLATE,
  TASKS_BODY_TEMPLATE,
} from "@polterware/feature-knowledge";

function core() {
  return window.polterware.core;
}

function changePayload(change: ChangeEnvelope) {
  return {
    summary: change.summary,
    rationale: change.rationale,
    agentRunId: change.agentRunId ?? "",
  };
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
  change: ChangeEnvelope,
): Promise<void> {
  const current = workspace.rawKnowledge.find((item) => asString(item.id) === itemId);
  if (!current) return;
  await core().knowledge.save({
    item: {
      ...current,
      projectId,
      body,
    },
    change: changePayload(change),
  });
}

export async function persistKnowledgeSourcePath(
  workspace: WorkspaceSnapshot,
  projectId: string,
  itemId: string,
  path: string,
  change: ChangeEnvelope,
): Promise<void> {
  const current = workspace.rawKnowledge.find((item) => asString(item.id) === itemId);
  if (!current) return;
  let extra: Record<string, unknown> = {};
  try {
    extra = JSON.parse(asString(current.extraJson, "{}")) as Record<string, unknown>;
  } catch {
    extra = {};
  }
  await core().knowledge.save({
    item: {
      ...current,
      projectId,
      extraJson: JSON.stringify({ ...extra, path }),
    },
    change: changePayload(change),
  });
}

export async function persistKnowledgeCreate(input: {
  projectId: string;
  kind: "doc" | "runbook" | "adr" | "source";
  title: string;
  body?: string;
  extraJson?: string;
  change: ChangeEnvelope;
}): Promise<string> {
  const id = newId(
    input.kind === "runbook"
      ? "rb"
      : input.kind === "adr"
        ? "adr"
        : input.kind === "source"
          ? "src"
          : "doc",
  );
  const defaultBody =
    input.kind === "adr"
      ? "## Status\n\nProposed\n\n## Context\n\n\n\n## Decision\n\n\n\n## Consequences\n\n"
      : input.kind === "source"
        ? ""
        : "";
  const defaultExtraJson =
    input.kind === "runbook"
      ? JSON.stringify({
          service: "Service",
          severity: "medium",
          prerequisites: [],
          steps: [],
          commands: [],
          rollback: "",
        })
      : input.kind === "source"
        ? JSON.stringify({ path: "" })
        : JSON.stringify({
            author: "Project maintainer",
            environment: "All",
            componentIds: [],
            componentNames: [],
          });
  await core().knowledge.save({
    item: {
      id,
      projectId: input.projectId,
      title: input.title,
      body: input.body ?? defaultBody,
      kind: input.kind,
      tags: input.kind === "adr" ? ["adr"] : input.kind === "source" ? ["source"] : [],
      summary: "",
      owner: "human",
      status: input.kind === "adr" ? "proposed" : input.kind === "source" ? "" : "draft",
      extraJson: input.extraJson ?? defaultExtraJson,
    },
    change: changePayload(input.change),
  });
  return id;
}

export async function persistKnowledgeCreateFeature(input: {
  workspace: WorkspaceSnapshot;
  projectId: string;
  title: string;
  change: ChangeEnvelope;
}): Promise<string> {
  const featureKey = nextFeatureKey(
    input.workspace.specFeatures.map((feature) => feature.featureKey),
    input.title,
  );
  const extraJson = JSON.stringify({ featureKey });
  const artifacts: Array<{
    kind: "spec" | "plan" | "tasks" | "clarify" | "checklist";
    body: string;
    prefix: string;
  }> = [
    { kind: "spec", body: SPEC_BODY_TEMPLATE, prefix: "spec" },
    { kind: "plan", body: PLAN_BODY_TEMPLATE, prefix: "plan" },
    { kind: "tasks", body: TASKS_BODY_TEMPLATE, prefix: "tasks" },
    { kind: "clarify", body: CLARIFY_BODY_TEMPLATE, prefix: "clarify" },
    { kind: "checklist", body: CHECKLIST_BODY_TEMPLATE, prefix: "checklist" },
  ];
  for (const artifact of artifacts) {
    await core().knowledge.save({
      item: {
        id: newId(artifact.prefix),
        projectId: input.projectId,
        title: input.title,
        body: artifact.body,
        kind: artifact.kind,
        tags: [],
        summary: "",
        owner: "human",
        status: "draft",
        extraJson,
      },
      change: changePayload(input.change),
    });
  }
  return featureKey;
}

export async function persistKnowledgeDelete(
  id: string,
  change: ChangeEnvelope,
): Promise<void> {
  await core().knowledge.delete({ id, change: changePayload(change) });
}

export async function persistKnowledgeArchiveFeature(
  workspace: WorkspaceSnapshot,
  featureKey: string,
  change: ChangeEnvelope,
): Promise<void> {
  const feature = workspace.specFeatures.find((item) => item.featureKey === featureKey);
  if (!feature) return;
  await Promise.all([
    persistKnowledgeDelete(feature.spec.id, change),
    persistKnowledgeDelete(feature.plan.id, change),
    persistKnowledgeDelete(feature.tasks.id, change),
    persistKnowledgeDelete(feature.clarify.id, change),
    persistKnowledgeDelete(feature.checklist.id, change),
  ]);
}

export class ChecklistIncompleteError extends Error {
  constructor() {
    super("Complete the checklist before marking ready.");
    this.name = "ChecklistIncompleteError";
  }
}

export async function persistKnowledgeSetFeatureStatus(
  workspace: WorkspaceSnapshot,
  projectId: string,
  featureKey: string,
  status: "draft" | "ready",
  change: ChangeEnvelope,
): Promise<void> {
  const feature = workspace.specFeatures.find((item) => item.featureKey === featureKey);
  if (!feature) return;
  if (status === "ready" && checklistHasOpenItems(feature.checklist.bodyMarkdown)) {
    throw new ChecklistIncompleteError();
  }
  for (const artifact of [
    feature.spec,
    feature.plan,
    feature.tasks,
    feature.clarify,
    feature.checklist,
  ]) {
    const current = workspace.rawKnowledge.find((item) => asString(item.id) === artifact.id);
    if (!current) continue;
    await core().knowledge.save({
      item: {
        ...current,
        projectId,
        status,
      },
      change: changePayload(change),
    });
  }
}

/** Move a feature to another Work board stage; stored on the spec artifact's extra_json. */
export async function persistKnowledgeSetFeatureStage(
  workspace: WorkspaceSnapshot,
  projectId: string,
  featureKey: string,
  stage: WorkStage,
  change: ChangeEnvelope,
): Promise<void> {
  const feature = workspace.specFeatures.find((item) => item.featureKey === featureKey);
  if (!feature) return;
  const current = workspace.rawKnowledge.find((item) => asString(item.id) === feature.spec.id);
  if (!current) return;
  const extra = parseJsonObject(asString(current.extraJson));
  await core().knowledge.save({
    item: {
      ...current,
      projectId,
      extraJson: JSON.stringify({ ...extra, featureKey, stage }),
    },
    change: changePayload(change),
  });
}

export async function persistConstitutionEnsure(input: {
  workspace: WorkspaceSnapshot;
  projectId: string;
  change: ChangeEnvelope;
}): Promise<string> {
  const existing = input.workspace.constitution;
  if (existing) return existing.id;
  const id = newId("constitution");
  await core().knowledge.save({
    item: {
      id,
      projectId: input.projectId,
      title: "Project constitution",
      body: CONSTITUTION_BODY_TEMPLATE,
      kind: "constitution",
      tags: [],
      summary: "",
      owner: "human",
      status: "active",
      extraJson: "{}",
    },
    change: changePayload(input.change),
  });
  return id;
}

export async function persistConstitutionArchive(
  workspace: WorkspaceSnapshot,
  change: ChangeEnvelope,
): Promise<void> {
  const constitution = workspace.constitution;
  if (!constitution) return;
  await persistKnowledgeDelete(constitution.id, change);
}

export async function persistKnowledgeDuplicate(
  workspace: WorkspaceSnapshot,
  projectId: string,
  itemId: string,
  change: ChangeEnvelope,
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
    change: changePayload(change),
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

export async function persistImportProjectSources(input: {
  projectId: string;
}): Promise<{
  factCount: number;
  evidenceCount: number;
  sourceCount: number;
  skippedCount: number;
  message: string;
}> {
  const response = asRecord(
    await core().ppm.importSources({ projectId: input.projectId }),
  );
  return {
    factCount: asNumber(response.factCount, 0),
    evidenceCount: asNumber(response.evidenceCount, 0),
    sourceCount: asNumber(response.sourceCount, 0),
    skippedCount: asNumber(response.skippedCount, 0),
    message: asString(response.message, "Import finished."),
  };
}

export async function persistPpmFact(input: {
  fact: {
    id?: string;
    projectId: string;
    statement: string;
    truthClass: string;
    entityId?: string;
  };
  change: ChangeEnvelope;
}): Promise<void> {
  await core().ppm.upsertFact({
    fact: {
      id: input.fact.id ?? "",
      projectId: input.fact.projectId,
      statement: input.fact.statement,
      truthClass: input.fact.truthClass,
      entityId: input.fact.entityId ?? "",
      source: "",
      actor: "",
      observedAt: "",
      createdAt: "",
      updatedAt: "",
    },
    change: changePayload(input.change),
  });
}

export async function persistPpmEvidence(input: {
  evidence: {
    id?: string;
    projectId: string;
    kind: string;
    summary: string;
    path?: string;
  };
  change: ChangeEnvelope;
}): Promise<{ id: string }> {
  const id = input.evidence.id?.trim() || newId("ev");
  await core().ppm.upsertEvidence({
    evidence: {
      id,
      projectId: input.evidence.projectId,
      kind: input.evidence.kind,
      summary: input.evidence.summary,
      path: input.evidence.path ?? "",
      source: "",
      actor: "",
      observedAt: "",
      createdAt: "",
      updatedAt: "",
    },
    change: changePayload(input.change),
  });
  return { id };
}

export async function persistLinkEvidenceToFact(input: {
  projectId: string;
  factId: string;
  evidenceId: string;
  change: ChangeEnvelope;
}): Promise<void> {
  await core().ppm.linkEvidenceToFact({
    projectId: input.projectId,
    factId: input.factId,
    evidenceId: input.evidenceId,
    change: changePayload(input.change),
  });
}

export async function resetLocalStore(): Promise<void> {
  await core().health.resetLocalStore({});
}
