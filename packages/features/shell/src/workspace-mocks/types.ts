/** Operator-facing fixture session lifecycle. No PTY is created. */
export type TerminalSessionStatus = "active" | "idle" | "closed";

export type TerminalSessionMock = {
  id: string;
  name: string;
  nodeId: string;
  nodeName: string;
  cwd: string;
  shell: string;
  status: TerminalSessionStatus;
  startedAt: string;
  duration: string;
  environment: string;
  correlationId: string;
  dataSource?: "fixture" | "mock";
  lines: string[];
};

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
  operationId?: string;
};
