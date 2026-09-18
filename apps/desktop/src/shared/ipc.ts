import type { AppIconOption, AppIconSelection } from "./app-icons.js";

export interface AppInfo {
  name: string;
  version: string;
}

export type JsonValue = string | number | boolean | null | JsonObject | JsonValue[];
export interface JsonObject {
  [key: string]: JsonValue;
}

export const IPC_CHANNELS = {
  app: {
    getInfo: "app:get-info",
    listIcons: "app:list-icons",
    getIcon: "app:get-icon",
    setIcon: "app:set-icon",
  },
  commander: {
    hideOverlay: "commander:hide-overlay",
    showMainWindow: "commander:show-main-window",
  },
  core: {
    healthCheck: "core:health-check",
    getStoreHealth: "core:get-store-health",
    resetLocalStore: "core:reset-local-store",
    listProjects: "core:list-projects",
    saveProject: "core:save-project",
    listEnvironments: "core:list-environments",
    saveEnvironment: "core:save-environment",
    getProjectSettings: "core:get-project-settings",
    saveProjectSettings: "core:save-project-settings",
    getLayout: "core:get-layout",
    updateLayout: "core:update-layout",
    listResources: "core:list-resources",
    saveResource: "core:save-resource",
    listViews: "core:list-views",
    saveView: "core:save-view",
    deleteView: "core:delete-view",
    listNodes: "core:list-nodes",
    saveNode: "core:save-node",
    listKnowledge: "core:list-knowledge",
    saveKnowledge: "core:save-knowledge",
    deleteKnowledge: "core:delete-knowledge",
    listOperations: "core:list-operations",
    requestOperation: "core:request-operation",
    getOperation: "core:get-operation",
    cancelOperation: "core:cancel-operation",
    listSchedules: "core:list-schedules",
    saveSchedule: "core:save-schedule",
    listWorkflows: "core:list-workflows",
    saveWorkflow: "core:save-workflow",
    runWorkflow: "core:run-workflow",
    listCliProviders: "core:list-cli-providers",
    listIntegrationConnections: "core:list-integration-connections",
    saveIntegrationConnection: "core:save-integration-connection",
    listIntegrationResources: "core:list-integration-resources",
    listIntegrationSyncRuns: "core:list-integration-sync-runs",
    saveIntegrationSyncRun: "core:save-integration-sync-run",
    listProcesses: "core:list-processes",
    startProcess: "core:start-process",
    stopProcess: "core:stop-process",
    getProcess: "core:get-process",
    queryLogs: "core:query-logs",
  },
} as const;

export type IpcChannelTree = typeof IPC_CHANNELS;

export const IPC_EVENTS = {
  commander: {
    focusSearch: "commander:focus-search",
  },
} as const;

export type IpcEventTree = typeof IPC_EVENTS;

export type CoreIpcChannel = (typeof IPC_CHANNELS.core)[keyof typeof IPC_CHANNELS.core];

export type AppIpcRequests = {
  [IPC_CHANNELS.app.getInfo]: undefined;
  [IPC_CHANNELS.app.listIcons]: undefined;
  [IPC_CHANNELS.app.getIcon]: undefined;
  [IPC_CHANNELS.app.setIcon]: JsonValue;
  [IPC_CHANNELS.commander.hideOverlay]: undefined;
  [IPC_CHANNELS.commander.showMainWindow]: undefined;
} & Record<CoreIpcChannel, JsonObject | undefined>;

export type AppIpcResponses = {
  [IPC_CHANNELS.app.getInfo]: AppInfo;
  [IPC_CHANNELS.app.listIcons]: AppIconOption[];
  [IPC_CHANNELS.app.getIcon]: AppIconSelection;
  [IPC_CHANNELS.app.setIcon]: AppIconSelection;
  [IPC_CHANNELS.commander.hideOverlay]: void;
  [IPC_CHANNELS.commander.showMainWindow]: void;
} & Record<CoreIpcChannel, JsonObject>;
