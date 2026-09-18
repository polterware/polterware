import type { JsonObject } from "../../../shared/ipc.js";
import { Code, ConnectError } from "@connectrpc/connect";
import { decodeBase64Bytes, toJsonObject, type CoreClients } from "../client.js";

function payload(value: JsonObject | undefined): Record<string, unknown> {
  return value ?? {};
}

function requestMessage(value: JsonObject | undefined): never {
  return payload(value) as never;
}

function terminalWriteMessage(value: JsonObject | undefined): never {
  const next = payload(value);
  const data = decodeBase64Bytes(next.data);
  if (data) next.data = data;
  return next as never;
}

function isTerminalNotFound(error: unknown): boolean {
  if (error instanceof ConnectError) {
    if (error.code === Code.NotFound) return true;
    const raw = `${error.rawMessage ?? ""} ${error.message}`.toLowerCase();
    if (raw.includes("terminal session not found")) return true;
  }
  const message = error instanceof Error ? error.message.toLowerCase() : String(error).toLowerCase();
  return message.includes("terminal session not found") || message.includes("not_found");
}

export function createCoreDomainControllers(clients: CoreClients) {
  return {
    healthCheck: async () => toJsonObject(await clients.health.check({})),
    getStoreHealth: async () => toJsonObject(await clients.maintenance.getStoreHealth({})),
    resetLocalStore: async () => toJsonObject(await clients.maintenance.resetLocalStore({})),
    listProjects: async (input?: JsonObject) => toJsonObject(await clients.projects.listProjects(requestMessage(input))),
    saveProject: async (input?: JsonObject) => toJsonObject(await clients.projects.saveProject(requestMessage(input))),
    archiveProject: async (input?: JsonObject) => toJsonObject(await clients.projects.archiveProject(requestMessage(input))),
    listEnvironments: async (input?: JsonObject) => toJsonObject(await clients.projects.listEnvironments(requestMessage(input))),
    saveEnvironment: async (input?: JsonObject) => toJsonObject(await clients.projects.saveEnvironment(requestMessage(input))),
    deleteEnvironment: async (input?: JsonObject) => toJsonObject(await clients.projects.deleteEnvironment(requestMessage(input))),
    getProjectSettings: async (input?: JsonObject) => toJsonObject(await clients.projects.getProjectSettings(requestMessage(input))),
    saveProjectSettings: async (input?: JsonObject) => toJsonObject(await clients.projects.saveProjectSettings(requestMessage(input))),
    getLayout: async (input?: JsonObject) => toJsonObject(await clients.architecture.getLayout(requestMessage(input))),
    updateLayout: async (input?: JsonObject) => toJsonObject(await clients.architecture.updateLayout(requestMessage(input))),
    listResources: async (input?: JsonObject) => toJsonObject(await clients.architecture.listResources(requestMessage(input))),
    saveResource: async (input?: JsonObject) => toJsonObject(await clients.architecture.saveResource(requestMessage(input))),
    listViews: async (input?: JsonObject) => toJsonObject(await clients.architecture.listViews(requestMessage(input))),
    saveView: async (input?: JsonObject) => toJsonObject(await clients.architecture.saveView(requestMessage(input))),
    deleteView: async (input?: JsonObject) => toJsonObject(await clients.architecture.deleteView(requestMessage(input))),
    listNodes: async (input?: JsonObject) => toJsonObject(await clients.nodes.listNodes(requestMessage(input))),
    saveNode: async (input?: JsonObject) => toJsonObject(await clients.nodes.saveNode(requestMessage(input))),
    listKnowledge: async (input?: JsonObject) => toJsonObject(await clients.knowledge.listKnowledge(requestMessage(input))),
    saveKnowledge: async (input?: JsonObject) => toJsonObject(await clients.knowledge.saveKnowledge(requestMessage(input))),
    deleteKnowledge: async (input?: JsonObject) => toJsonObject(await clients.knowledge.deleteKnowledge(requestMessage(input))),
    listOperations: async (input?: JsonObject) => toJsonObject(await clients.operations.listOperations(requestMessage(input))),
    requestOperation: async (input?: JsonObject) => toJsonObject(await clients.operations.requestOperation(requestMessage(input))),
    getOperation: async (input?: JsonObject) => toJsonObject(await clients.operations.getOperation(requestMessage(input))),
    cancelOperation: async (input?: JsonObject) => toJsonObject(await clients.operations.cancelOperation(requestMessage(input))),
    listSchedules: async (input?: JsonObject) => toJsonObject(await clients.operations.listSchedules(requestMessage(input))),
    saveSchedule: async (input?: JsonObject) => toJsonObject(await clients.operations.saveSchedule(requestMessage(input))),
    listWorkflows: async (input?: JsonObject) => toJsonObject(await clients.workflows.listWorkflows(requestMessage(input))),
    saveWorkflow: async (input?: JsonObject) => toJsonObject(await clients.workflows.saveWorkflow(requestMessage(input))),
    runWorkflow: async (input?: JsonObject) => toJsonObject(await clients.workflows.runWorkflow(requestMessage(input))),
    listCliProviders: async () => toJsonObject(await clients.integrations.listCliProviders({})),
    listIntegrationConnections: async (input?: JsonObject) => toJsonObject(await clients.integrations.listIntegrationConnections(requestMessage(input))),
    saveIntegrationConnection: async (input?: JsonObject) => toJsonObject(await clients.integrations.saveIntegrationConnection(requestMessage(input))),
    listIntegrationResources: async (input?: JsonObject) => toJsonObject(await clients.integrations.listIntegrationResources(requestMessage(input))),
    listIntegrationSyncRuns: async (input?: JsonObject) => toJsonObject(await clients.integrations.listIntegrationSyncRuns(requestMessage(input))),
    saveIntegrationSyncRun: async (input?: JsonObject) => toJsonObject(await clients.integrations.saveIntegrationSyncRun(requestMessage(input))),
    syncIntegrationConnection: async (input?: JsonObject) => toJsonObject(await clients.integrations.syncIntegrationConnection(requestMessage(input))),
    probeCliProvider: async (input?: JsonObject) => toJsonObject(await clients.integrations.probeCliProvider(requestMessage(input))),
    beginCliAuth: async (input?: JsonObject) => toJsonObject(await clients.integrations.beginCliAuth(requestMessage(input))),
    listProcesses: async (input?: JsonObject) => toJsonObject(await clients.processes.listProcesses(requestMessage(input))),
    startProcess: async (input?: JsonObject) => toJsonObject(await clients.processes.startProcess(requestMessage(input))),
    stopProcess: async (input?: JsonObject) => toJsonObject(await clients.processes.stopProcess(requestMessage(input))),
    getProcess: async (input?: JsonObject) => toJsonObject(await clients.processes.getProcess(requestMessage(input))),
    createTerminalSession: async (input?: JsonObject) => toJsonObject(await clients.terminals.createTerminalSession(requestMessage(input))),
    writeTerminalSession: async (input?: JsonObject) => {
      try {
        return toJsonObject(await clients.terminals.writeTerminalSession(terminalWriteMessage(input)));
      } catch (error: unknown) {
        if (isTerminalNotFound(error)) return {};
        throw error;
      }
    },
    resizeTerminalSession: async (input?: JsonObject) => {
      try {
        return toJsonObject(await clients.terminals.resizeTerminalSession(requestMessage(input)));
      } catch (error: unknown) {
        if (isTerminalNotFound(error)) return {};
        throw error;
      }
    },
    closeTerminalSession: async (input?: JsonObject) => {
      try {
        return toJsonObject(await clients.terminals.closeTerminalSession(requestMessage(input)));
      } catch (error: unknown) {
        if (isTerminalNotFound(error)) return {};
        throw error;
      }
    },
    readTerminalOutput: async (input?: JsonObject) => {
      try {
        return toJsonObject(await clients.terminals.readTerminalOutput(requestMessage(input)));
      } catch (error: unknown) {
        if (isTerminalNotFound(error)) {
          const offset = Number(payload(input).offset ?? 0);
          return { data: "", nextOffset: offset, closed: true };
        }
        throw error;
      }
    },
    listTerminalSessions: async (input?: JsonObject) => toJsonObject(await clients.terminals.listTerminalSessions(requestMessage(input))),
    queryLogs: async (input?: JsonObject) => toJsonObject(await clients.logs.queryLogs(requestMessage(input))),
  };
}

export type CoreDomainControllers = ReturnType<typeof createCoreDomainControllers>;
