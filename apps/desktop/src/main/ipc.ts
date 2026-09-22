import { ipcMain, type IpcMain, type IpcMainInvokeEvent } from "electron";
import type { AppIconOption, AppIconSelection } from "../shared/app-icons.js";
import {
  IPC_CHANNELS,
  type AppInfo,
  type AppIpcRequests,
  type AppIpcResponses,
  type JsonObject,
  type JsonValue,
} from "../shared/ipc.js";
import type {
  StoredShortcutOverrides,
  StoredShortcutPreferences,
} from "../shared/shortcut-preferences.js";
import type { McpServerStatus } from "../shared/mcp-status.js";
import type { CoreDomainControllers } from "./core/controllers/domain.js";

type Handler<K extends keyof AppIpcRequests> = (
  event?: IpcMainInvokeEvent,
  payload?: AppIpcRequests[K],
) => AppIpcResponses[K] | Promise<AppIpcResponses[K]>;

export interface PolterwareIpcControllers {
  appInfo?: {
    getInfo(): AppInfo;
  };
  appIcons?: {
    getIcon(): AppIconSelection;
    listIcons(): AppIconOption[];
    setIcon(id: string): AppIconSelection;
  };
  shortcutPreferences?: {
    getPreferences(): StoredShortcutPreferences;
    setPreferences(overrides: StoredShortcutOverrides): StoredShortcutPreferences;
  };
  mcp?: {
    getStatus(): McpServerStatus;
    start(): Promise<McpServerStatus> | McpServerStatus;
    stop(): Promise<McpServerStatus> | McpServerStatus;
  };
  appShell?: {
    openExternal(url: string): Promise<{ ok: boolean }> | { ok: boolean };
  };
  commander?: {
    hideOverlay(): void;
    showMainWindow(): void;
  };
  core?: CoreDomainControllers;
}

function missing(channel: string): never {
  throw new Error(`Polterware IPC channel is not connected: ${channel}`);
}

function isJsonObject(value: JsonValue): value is JsonObject {
  return value !== null && !Array.isArray(value) && Object(value) === value;
}

function parseAppIconSelectionPayload(payload: JsonValue | undefined): string {
  if (!payload || !isJsonObject(payload)) {
    throw new Error("App icon id is required.");
  }
  const id = payload.id;
  if (Object.prototype.toString.call(id) !== "[object String]") {
    throw new Error("App icon id is required.");
  }
  const parsedId = String(id);
  if (!parsedId.trim()) {
    throw new Error("App icon id is required.");
  }
  return parsedId;
}

function parseShortcutPreferencesPayload(
  payload: JsonValue | undefined,
): StoredShortcutOverrides {
  if (!payload || !isJsonObject(payload)) {
    throw new Error("Shortcut preferences are required.");
  }
  const overridesValue = payload.overrides;
  if (
    overridesValue === null
    || overridesValue === undefined
    || Array.isArray(overridesValue)
    || Object(overridesValue) !== overridesValue
  ) {
    throw new Error("Shortcut overrides are required.");
  }

  const overrides: StoredShortcutOverrides = {};
  for (const [id, chord] of Object.entries(overridesValue as JsonObject)) {
    if (Object.prototype.toString.call(chord) !== "[object String]") {
      throw new Error(`Shortcut override for ${id} must be a string.`);
    }
    const trimmed = String(chord).trim();
    if (!trimmed) {
      throw new Error(`Shortcut override for ${id} is empty.`);
    }
    overrides[id] = trimmed.toLowerCase();
  }
  return overrides;
}

function coreHandler(
  controllers: PolterwareIpcControllers,
  key: keyof CoreDomainControllers,
): Handler<keyof AppIpcRequests> {
  return async (_event, payload) => {
    const core = controllers.core;
    if (!core) missing(key);
    return (await core[key](payload as JsonObject | undefined)) as JsonObject;
  };
}

export function createPolterwareIpcHandlers(controllers: PolterwareIpcControllers = {}) {
  const handlers = {
    [IPC_CHANNELS.app.getInfo]: async () => controllers.appInfo?.getInfo() ?? missing(IPC_CHANNELS.app.getInfo),
    [IPC_CHANNELS.app.listIcons]: async () => controllers.appIcons?.listIcons() ?? missing(IPC_CHANNELS.app.listIcons),
    [IPC_CHANNELS.app.getIcon]: async () => controllers.appIcons?.getIcon() ?? missing(IPC_CHANNELS.app.getIcon),
    [IPC_CHANNELS.app.setIcon]: async (_event, payload) => {
      if (!controllers.appIcons) missing(IPC_CHANNELS.app.setIcon);
      return controllers.appIcons.setIcon(parseAppIconSelectionPayload(payload));
    },
    [IPC_CHANNELS.app.getShortcutPreferences]: async () =>
      controllers.shortcutPreferences?.getPreferences()
      ?? missing(IPC_CHANNELS.app.getShortcutPreferences),
    [IPC_CHANNELS.app.setShortcutPreferences]: async (_event, payload) => {
      if (!controllers.shortcutPreferences) {
        missing(IPC_CHANNELS.app.setShortcutPreferences);
      }
      return controllers.shortcutPreferences.setPreferences(
        parseShortcutPreferencesPayload(payload),
      );
    },
    [IPC_CHANNELS.app.getMcpStatus]: async () =>
      controllers.mcp?.getStatus() ?? missing(IPC_CHANNELS.app.getMcpStatus),
    [IPC_CHANNELS.app.startMcp]: async () => {
      if (!controllers.mcp) missing(IPC_CHANNELS.app.startMcp);
      return controllers.mcp.start();
    },
    [IPC_CHANNELS.app.stopMcp]: async () => {
      if (!controllers.mcp) missing(IPC_CHANNELS.app.stopMcp);
      return controllers.mcp.stop();
    },
    [IPC_CHANNELS.app.openExternal]: async (_event, payload) => {
      if (!controllers.appShell) missing(IPC_CHANNELS.app.openExternal);
      const url = payload && isJsonObject(payload as JsonValue) ? String((payload as JsonObject).url ?? "") : "";
      if (!url.trim()) throw new Error("External URL is required.");
      return controllers.appShell.openExternal(url);
    },
    [IPC_CHANNELS.commander.hideOverlay]: async () => {
      if (!controllers.commander) missing(IPC_CHANNELS.commander.hideOverlay);
      controllers.commander.hideOverlay();
    },
    [IPC_CHANNELS.commander.showMainWindow]: async () => {
      if (!controllers.commander) missing(IPC_CHANNELS.commander.showMainWindow);
      controllers.commander.showMainWindow();
    },
    [IPC_CHANNELS.core.healthCheck]: coreHandler(controllers, "healthCheck"),
    [IPC_CHANNELS.core.getStoreHealth]: coreHandler(controllers, "getStoreHealth"),
    [IPC_CHANNELS.core.resetLocalStore]: coreHandler(controllers, "resetLocalStore"),
    [IPC_CHANNELS.core.listProjects]: coreHandler(controllers, "listProjects"),
    [IPC_CHANNELS.core.saveProject]: coreHandler(controllers, "saveProject"),
    [IPC_CHANNELS.core.archiveProject]: coreHandler(controllers, "archiveProject"),
    [IPC_CHANNELS.core.listEnvironments]: coreHandler(controllers, "listEnvironments"),
    [IPC_CHANNELS.core.saveEnvironment]: coreHandler(controllers, "saveEnvironment"),
    [IPC_CHANNELS.core.deleteEnvironment]: coreHandler(controllers, "deleteEnvironment"),
    [IPC_CHANNELS.core.getProjectSettings]: coreHandler(controllers, "getProjectSettings"),
    [IPC_CHANNELS.core.saveProjectSettings]: coreHandler(controllers, "saveProjectSettings"),
    [IPC_CHANNELS.core.getLayout]: coreHandler(controllers, "getLayout"),
    [IPC_CHANNELS.core.updateLayout]: coreHandler(controllers, "updateLayout"),
    [IPC_CHANNELS.core.listResources]: coreHandler(controllers, "listResources"),
    [IPC_CHANNELS.core.saveResource]: coreHandler(controllers, "saveResource"),
    [IPC_CHANNELS.core.listViews]: coreHandler(controllers, "listViews"),
    [IPC_CHANNELS.core.saveView]: coreHandler(controllers, "saveView"),
    [IPC_CHANNELS.core.deleteView]: coreHandler(controllers, "deleteView"),
    [IPC_CHANNELS.core.listNodes]: coreHandler(controllers, "listNodes"),
    [IPC_CHANNELS.core.saveNode]: coreHandler(controllers, "saveNode"),
    [IPC_CHANNELS.core.listKnowledge]: coreHandler(controllers, "listKnowledge"),
    [IPC_CHANNELS.core.saveKnowledge]: coreHandler(controllers, "saveKnowledge"),
    [IPC_CHANNELS.core.deleteKnowledge]: coreHandler(controllers, "deleteKnowledge"),
    [IPC_CHANNELS.core.searchKnowledge]: coreHandler(controllers, "searchKnowledge"),
    [IPC_CHANNELS.core.listKnowledgeLinks]: coreHandler(controllers, "listKnowledgeLinks"),
    [IPC_CHANNELS.core.saveKnowledgeLink]: coreHandler(controllers, "saveKnowledgeLink"),
    [IPC_CHANNELS.core.deleteKnowledgeLink]: coreHandler(controllers, "deleteKnowledgeLink"),
    [IPC_CHANNELS.core.listPpmEntities]: coreHandler(controllers, "listPpmEntities"),
    [IPC_CHANNELS.core.syncProjectModel]: coreHandler(controllers, "syncProjectModel"),
    [IPC_CHANNELS.core.compileContext]: coreHandler(controllers, "compileContext"),
    [IPC_CHANNELS.core.listAgentToolCalls]: coreHandler(controllers, "listAgentToolCalls"),
    [IPC_CHANNELS.core.listOperations]: coreHandler(controllers, "listOperations"),
    [IPC_CHANNELS.core.requestOperation]: coreHandler(controllers, "requestOperation"),
    [IPC_CHANNELS.core.getOperation]: coreHandler(controllers, "getOperation"),
    [IPC_CHANNELS.core.cancelOperation]: coreHandler(controllers, "cancelOperation"),
    [IPC_CHANNELS.core.listSchedules]: coreHandler(controllers, "listSchedules"),
    [IPC_CHANNELS.core.saveSchedule]: coreHandler(controllers, "saveSchedule"),
    [IPC_CHANNELS.core.listWorkflows]: coreHandler(controllers, "listWorkflows"),
    [IPC_CHANNELS.core.saveWorkflow]: coreHandler(controllers, "saveWorkflow"),
    [IPC_CHANNELS.core.runWorkflow]: coreHandler(controllers, "runWorkflow"),
    [IPC_CHANNELS.core.listCliProviders]: coreHandler(controllers, "listCliProviders"),
    [IPC_CHANNELS.core.listIntegrationConnections]: coreHandler(controllers, "listIntegrationConnections"),
    [IPC_CHANNELS.core.saveIntegrationConnection]: coreHandler(controllers, "saveIntegrationConnection"),
    [IPC_CHANNELS.core.listIntegrationResources]: coreHandler(controllers, "listIntegrationResources"),
    [IPC_CHANNELS.core.listIntegrationSyncRuns]: coreHandler(controllers, "listIntegrationSyncRuns"),
    [IPC_CHANNELS.core.saveIntegrationSyncRun]: coreHandler(controllers, "saveIntegrationSyncRun"),
    [IPC_CHANNELS.core.syncIntegrationConnection]: coreHandler(controllers, "syncIntegrationConnection"),
    [IPC_CHANNELS.core.probeCliProvider]: coreHandler(controllers, "probeCliProvider"),
    [IPC_CHANNELS.core.beginCliAuth]: coreHandler(controllers, "beginCliAuth"),
    [IPC_CHANNELS.core.listProcesses]: coreHandler(controllers, "listProcesses"),
    [IPC_CHANNELS.core.startProcess]: coreHandler(controllers, "startProcess"),
    [IPC_CHANNELS.core.stopProcess]: coreHandler(controllers, "stopProcess"),
    [IPC_CHANNELS.core.getProcess]: coreHandler(controllers, "getProcess"),
    [IPC_CHANNELS.core.createTerminalSession]: coreHandler(controllers, "createTerminalSession"),
    [IPC_CHANNELS.core.writeTerminalSession]: coreHandler(controllers, "writeTerminalSession"),
    [IPC_CHANNELS.core.resizeTerminalSession]: coreHandler(controllers, "resizeTerminalSession"),
    [IPC_CHANNELS.core.closeTerminalSession]: coreHandler(controllers, "closeTerminalSession"),
    [IPC_CHANNELS.core.readTerminalOutput]: coreHandler(controllers, "readTerminalOutput"),
    [IPC_CHANNELS.core.listTerminalSessions]: coreHandler(controllers, "listTerminalSessions"),
    [IPC_CHANNELS.core.queryLogs]: coreHandler(controllers, "queryLogs"),
  } as { [K in keyof AppIpcRequests]: Handler<K> };

  return handlers;
}

export function registerPolterwareIpcHandlers(
  controllers: PolterwareIpcControllers = {},
  target: Pick<IpcMain, "handle"> = ipcMain,
) {
  const handlers = createPolterwareIpcHandlers(controllers);
  for (const [channel, handler] of Object.entries(handlers)) {
    target.handle(channel, (event, payload) =>
      (handler as Handler<keyof AppIpcRequests>)(event, payload as never),
    );
  }
  return handlers;
}
