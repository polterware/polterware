export type CliConnectionState =
  | "cli-unavailable"
  | "ready"
  | "authenticating"
  | "connected"
  | "failed";

export type IntegrationCatalogCategory =
  | "Source Control"
  | "Cloud"
  | "Deployment"
  | "Databases"
  | "Infrastructure";

export type WorkspaceCatalogProvider = {
  id: string;
  name: string;
  description: string;
  category: IntegrationCatalogCategory;
  binary: string;
  authCommands: string[];
  transcript: string[];
  state: CliConnectionState;
};

export type WorkspaceIntegration = {
  id: string;
  providerId: string;
  name: string;
  provider: string;
  status: "connected" | "error" | "disconnected" | "disabled";
  connectionLabel: string;
  environment: string;
  environments?: string[];
  lastSync: string;
  summary: string;
  resourceCount: number;
  resourceIds: string[];
  cliCommand: string;
  syncInterval?: string;
  freshness?: "fresh" | "stale" | "unknown";
  error?: string;
};

export type WorkspaceIntegrationResource = {
  id: string;
  integrationId: string;
  name: string;
  kind: string;
  externalId: string;
  environment: string;
  status: string;
  inventoryResourceId?: string;
};

export type WorkspaceIntegrationSyncRun = {
  id: string;
  integrationId: string;
  startedAt: string;
  finishedAt: string;
  result: "success" | "partial" | "failed" | "running";
  imported: number;
  changed: number;
  message: string;
};

export type WorkspaceEntityLink = {
  id: string;
  entityId: string;
  entityTitle: string;
  entityKind: string;
  entityRefId: string;
  predicate: string;
  truthClass: string;
  source: string;
};

export type WorkspacePpmEntity = {
  id: string;
  kind: string;
  refKind: string;
  refId: string;
  title: string;
  truthClass: string;
  source: string;
};

export type WorkspaceDocument = {
  id: string;
  title: string;
  summary: string;
  bodyMarkdown: string;
  author: string;
  updatedAt: string;
  tags: string[];
  componentIds: string[];
  componentNames: string[];
  environment: string;
  status: "draft" | "published";
  archived?: boolean;
  owner: string;
  entityLinks: WorkspaceEntityLink[];
};

export type WorkspaceAdr = {
  id: string;
  title: string;
  summary: string;
  bodyMarkdown: string;
  updatedAt: string;
  tags: string[];
  status: string;
  owner: string;
  entityLinks: WorkspaceEntityLink[];
};

export type WorkspaceRunbook = {
  id: string;
  title: string;
  objective: string;
  bodyMarkdown: string;
  service: string;
  severity: "low" | "medium" | "high" | "critical";
  prerequisites: string[];
  steps: string[];
  commands: string[];
  rollback: string;
  owner: string;
  lastValidated: string;
  componentId?: string;
  componentName?: string;
  resourceId?: string;
  resourceName?: string;
  nodeId?: string;
  entityLinks: WorkspaceEntityLink[];
};

export type WorkspaceSource = {
  id: string;
  title: string;
  path: string;
  summary: string;
  bodyMarkdown: string;
  updatedAt: string;
  owner: string;
  entityLinks: WorkspaceEntityLink[];
};

export type WorkspaceSpecArtifactKind = "spec" | "plan" | "tasks" | "clarify" | "checklist";

export type WorkspaceSpecArtifact = {
  id: string;
  kind: WorkspaceSpecArtifactKind;
  title: string;
  bodyMarkdown: string;
  status: "draft" | "ready";
  updatedAt: string;
  owner: string;
  featureKey: string;
};

/** Work board column for a feature (Grounded SDD lifecycle). */
export type WorkStage = "shape" | "plan" | "build" | "verify" | "delivered";

export type WorkspaceSpecFeature = {
  featureKey: string;
  title: string;
  status: "draft" | "ready";
  /** Stage persisted on the spec artifact; absent means derive from status and tasks. */
  stage?: WorkStage;
  updatedAt: string;
  owner: string;
  summary: string;
  spec: WorkspaceSpecArtifact;
  plan: WorkspaceSpecArtifact;
  tasks: WorkspaceSpecArtifact;
  clarify: WorkspaceSpecArtifact;
  checklist: WorkspaceSpecArtifact;
};

export type WorkspaceConstitution = {
  id: string;
  title: string;
  bodyMarkdown: string;
  updatedAt: string;
  owner: string;
  status: string;
};
