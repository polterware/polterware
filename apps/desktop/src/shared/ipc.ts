import type { AppIconOption, AppIconSelection } from "./app-icons.js";
import type { McpServerStatus } from "./mcp-status.js";
import type { StoredAppearancePreferences } from "./appearance-preferences.js";
import type { StoredShortcutPreferences } from "./shortcut-preferences.js";

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
    getShortcutPreferences: "app:get-shortcut-preferences",
    setShortcutPreferences: "app:set-shortcut-preferences",
    getAppearancePreferences: "app:get-appearance-preferences",
    setAppearancePreferences: "app:set-appearance-preferences",
    getMcpStatus: "app:get-mcp-status",
    startMcp: "app:start-mcp",
    stopMcp: "app:stop-mcp",
    openExternal: "app:open-external",
    selectDirectory: "app:select-directory",
  },
  commander: {
    hideOverlay: "commander:hide-overlay",
    showMainWindow: "commander:show-main-window",
    runCommand: "commander:run-command",
  },
  core: {
    healthCheck: "core:health-check",
    getStoreHealth: "core:get-store-health",
    resetLocalStore: "core:reset-local-store",
    listProjects: "core:list-projects",
    saveProject: "core:save-project",
    archiveProject: "core:archive-project",
    listEnvironments: "core:list-environments",
    saveEnvironment: "core:save-environment",
    deleteEnvironment: "core:delete-environment",
    getProjectSettings: "core:get-project-settings",
    saveProjectSettings: "core:save-project-settings",
    getLayout: "core:get-layout",
    updateLayout: "core:update-layout",
    listResources: "core:list-resources",
    saveResource: "core:save-resource",
    listViews: "core:list-views",
    saveView: "core:save-view",
    deleteView: "core:delete-view",
    listKnowledge: "core:list-knowledge",
    saveKnowledge: "core:save-knowledge",
    deleteKnowledge: "core:delete-knowledge",
    searchKnowledge: "core:search-knowledge",
    listKnowledgeLinks: "core:list-knowledge-links",
    saveKnowledgeLink: "core:save-knowledge-link",
    deleteKnowledgeLink: "core:delete-knowledge-link",
    listPpmEntities: "core:list-ppm-entities",
    syncProjectModel: "core:sync-project-model",
    importProjectSources: "core:import-project-sources",
    compileContext: "core:compile-context",
    listAgentToolCalls: "core:list-agent-tool-calls",
    listAgentRuns: "core:list-agent-runs",
    getAgentRun: "core:get-agent-run",
    listCliProviders: "core:list-cli-providers",
    listIntegrationConnections: "core:list-integration-connections",
    saveIntegrationConnection: "core:save-integration-connection",
    listIntegrationResources: "core:list-integration-resources",
    listIntegrationSyncRuns: "core:list-integration-sync-runs",
    saveIntegrationSyncRun: "core:save-integration-sync-run",
    syncIntegrationConnection: "core:sync-integration-connection",
    probeCliProvider: "core:probe-cli-provider",
    beginCliAuth: "core:begin-cli-auth",
    createTerminalSession: "core:create-terminal-session",
    writeTerminalSession: "core:write-terminal-session",
    resizeTerminalSession: "core:resize-terminal-session",
    closeTerminalSession: "core:close-terminal-session",
    readTerminalOutput: "core:read-terminal-output",
    listTerminalSessions: "core:list-terminal-sessions",
  },
} as const;

export type IpcChannelTree = typeof IPC_CHANNELS;

export const IPC_EVENTS = {
  commander: {
    focusSearch: "commander:focus-search",
    executeCommand: "commander:execute-command",
  },
} as const;

export type IpcEventTree = typeof IPC_EVENTS;

export type CoreIpcChannel = (typeof IPC_CHANNELS.core)[keyof typeof IPC_CHANNELS.core];

export type AppIpcRequests = {
  [IPC_CHANNELS.app.getInfo]: undefined;
  [IPC_CHANNELS.app.listIcons]: undefined;
  [IPC_CHANNELS.app.getIcon]: undefined;
  [IPC_CHANNELS.app.setIcon]: JsonValue;
  [IPC_CHANNELS.app.getShortcutPreferences]: undefined;
  [IPC_CHANNELS.app.setShortcutPreferences]: JsonValue;
  [IPC_CHANNELS.app.getAppearancePreferences]: undefined;
  [IPC_CHANNELS.app.setAppearancePreferences]: JsonValue;
  [IPC_CHANNELS.app.getMcpStatus]: undefined;
  [IPC_CHANNELS.app.startMcp]: undefined;
  [IPC_CHANNELS.app.stopMcp]: undefined;
  [IPC_CHANNELS.app.openExternal]: { url: string };
  [IPC_CHANNELS.app.selectDirectory]: undefined;
  [IPC_CHANNELS.commander.hideOverlay]: undefined;
  [IPC_CHANNELS.commander.showMainWindow]: undefined;
  [IPC_CHANNELS.commander.runCommand]: { id: string };
} & Record<CoreIpcChannel, JsonObject | undefined>;

export type AppIpcResponses = {
  [IPC_CHANNELS.app.getInfo]: AppInfo;
  [IPC_CHANNELS.app.listIcons]: AppIconOption[];
  [IPC_CHANNELS.app.getIcon]: AppIconSelection;
  [IPC_CHANNELS.app.setIcon]: AppIconSelection;
  [IPC_CHANNELS.app.getShortcutPreferences]: StoredShortcutPreferences;
  [IPC_CHANNELS.app.setShortcutPreferences]: StoredShortcutPreferences;
  [IPC_CHANNELS.app.getAppearancePreferences]: StoredAppearancePreferences;
  [IPC_CHANNELS.app.setAppearancePreferences]: StoredAppearancePreferences;
  [IPC_CHANNELS.app.getMcpStatus]: McpServerStatus;
  [IPC_CHANNELS.app.startMcp]: McpServerStatus;
  [IPC_CHANNELS.app.stopMcp]: McpServerStatus;
  [IPC_CHANNELS.app.openExternal]: { ok: boolean };
  [IPC_CHANNELS.app.selectDirectory]: { path: string | null };
  [IPC_CHANNELS.commander.hideOverlay]: void;
  [IPC_CHANNELS.commander.showMainWindow]: void;
  [IPC_CHANNELS.commander.runCommand]: void;
} & Record<CoreIpcChannel, JsonObject>;
