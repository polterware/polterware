import { createClient, type Client, type Interceptor, type Transport } from "@connectrpc/connect";
import { createConnectTransport } from "@connectrpc/connect-node";
import {
  ArchitectureService,
  HealthService,
  IntegrationService,
  KnowledgeService,
  LogService,
  MaintenanceService,
  NodeService,
  OperationService,
  ProcessService,
  ProjectService,
  WorkflowService,
} from "@polterware/contracts";

const authInterceptor = (token: string): Interceptor => (next) => async (request) => {
  request.header.set("Authorization", `Bearer ${token}`);
  return next(request);
};

export type CoreClients = {
  health: Client<typeof HealthService>;
  maintenance: Client<typeof MaintenanceService>;
  projects: Client<typeof ProjectService>;
  architecture: Client<typeof ArchitectureService>;
  nodes: Client<typeof NodeService>;
  knowledge: Client<typeof KnowledgeService>;
  operations: Client<typeof OperationService>;
  workflows: Client<typeof WorkflowService>;
  integrations: Client<typeof IntegrationService>;
  processes: Client<typeof ProcessService>;
  logs: Client<typeof LogService>;
};

export function createCoreTransport(endpoint: string, token: string): Transport {
  return createConnectTransport({
    baseUrl: endpoint,
    httpVersion: "1.1",
    interceptors: [authInterceptor(token)],
  });
}

export function createCoreClients(endpoint: string, token: string): CoreClients {
  const transport = createCoreTransport(endpoint, token);
  return {
    health: createClient(HealthService, transport),
    maintenance: createClient(MaintenanceService, transport),
    projects: createClient(ProjectService, transport),
    architecture: createClient(ArchitectureService, transport),
    nodes: createClient(NodeService, transport),
    knowledge: createClient(KnowledgeService, transport),
    operations: createClient(OperationService, transport),
    workflows: createClient(WorkflowService, transport),
    integrations: createClient(IntegrationService, transport),
    processes: createClient(ProcessService, transport),
    logs: createClient(LogService, transport),
  };
}

export async function waitForHealth(clients: CoreClients): Promise<void> {
  let lastError: unknown;
  for (let attempt = 0; attempt < 20; attempt += 1) {
    try {
      await clients.health.check({});
      return;
    } catch (error) {
      lastError = error;
      await new Promise((resolve) => setTimeout(resolve, 150));
    }
  }
  throw lastError instanceof Error ? lastError : new Error("Polterware core health check failed.");
}

export function toJsonObject(value: unknown): Record<string, unknown> {
  return JSON.parse(JSON.stringify(value ?? {}, (_key, current) => (
    typeof current === "bigint" ? Number(current) : current
  ))) as Record<string, unknown>;
}
