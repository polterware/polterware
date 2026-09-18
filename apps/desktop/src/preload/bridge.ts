import { contextBridge, ipcRenderer } from "electron";
import { IPC_CHANNELS, IPC_EVENTS, type AppIpcRequests, type AppIpcResponses, type JsonObject } from "../shared/ipc.js";

type BridgeListener = () => void;
type BridgeIpc = {
  invoke<K extends keyof AppIpcRequests>(channel: K, payload?: AppIpcRequests[K]): Promise<AppIpcResponses[K]>;
  on?(channel: string, listener: BridgeListener): void;
  removeListener?(channel: string, listener: BridgeListener): void;
};

function coreCall(ipc: BridgeIpc, channel: (typeof IPC_CHANNELS.core)[keyof typeof IPC_CHANNELS.core], payload?: JsonObject) {
  return ipc.invoke(channel, payload);
}

export function createPolterwareBridge(ipc: BridgeIpc) {
  return {
    app: {
      getInfo: () => ipc.invoke(IPC_CHANNELS.app.getInfo),
      listIcons: () => ipc.invoke(IPC_CHANNELS.app.listIcons),
      getIcon: () => ipc.invoke(IPC_CHANNELS.app.getIcon),
      setIcon: (id: string) => ipc.invoke(IPC_CHANNELS.app.setIcon, { id }),
    },
    commander: {
      hideOverlay: () => ipc.invoke(IPC_CHANNELS.commander.hideOverlay),
      showMainWindow: () => ipc.invoke(IPC_CHANNELS.commander.showMainWindow),
      onFocusSearch: (callback: () => void) => {
        const listener = () => callback();
        ipc.on?.(IPC_EVENTS.commander.focusSearch, listener);
        return () => ipc.removeListener?.(IPC_EVENTS.commander.focusSearch, listener);
      },
    },
    core: {
      health: {
        check: () => coreCall(ipc, IPC_CHANNELS.core.healthCheck),
        getStoreHealth: () => coreCall(ipc, IPC_CHANNELS.core.getStoreHealth),
        resetLocalStore: () => coreCall(ipc, IPC_CHANNELS.core.resetLocalStore),
      },
      projects: {
        list: () => coreCall(ipc, IPC_CHANNELS.core.listProjects),
        save: (payload: JsonObject) => coreCall(ipc, IPC_CHANNELS.core.saveProject, payload),
        listEnvironments: (payload: JsonObject) => coreCall(ipc, IPC_CHANNELS.core.listEnvironments, payload),
        saveEnvironment: (payload: JsonObject) => coreCall(ipc, IPC_CHANNELS.core.saveEnvironment, payload),
        getSettings: (payload: JsonObject) => coreCall(ipc, IPC_CHANNELS.core.getProjectSettings, payload),
        saveSettings: (payload: JsonObject) => coreCall(ipc, IPC_CHANNELS.core.saveProjectSettings, payload),
      },
      architecture: {
        getLayout: (payload: JsonObject) => coreCall(ipc, IPC_CHANNELS.core.getLayout, payload),
        updateLayout: (payload: JsonObject) => coreCall(ipc, IPC_CHANNELS.core.updateLayout, payload),
        listResources: (payload: JsonObject) => coreCall(ipc, IPC_CHANNELS.core.listResources, payload),
        saveResource: (payload: JsonObject) => coreCall(ipc, IPC_CHANNELS.core.saveResource, payload),
        listViews: (payload: JsonObject) => coreCall(ipc, IPC_CHANNELS.core.listViews, payload),
        saveView: (payload: JsonObject) => coreCall(ipc, IPC_CHANNELS.core.saveView, payload),
        deleteView: (payload: JsonObject) => coreCall(ipc, IPC_CHANNELS.core.deleteView, payload),
      },
      nodes: {
        list: (payload: JsonObject) => coreCall(ipc, IPC_CHANNELS.core.listNodes, payload),
        save: (payload: JsonObject) => coreCall(ipc, IPC_CHANNELS.core.saveNode, payload),
      },
      knowledge: {
        list: (payload: JsonObject) => coreCall(ipc, IPC_CHANNELS.core.listKnowledge, payload),
        save: (payload: JsonObject) => coreCall(ipc, IPC_CHANNELS.core.saveKnowledge, payload),
        delete: (payload: JsonObject) => coreCall(ipc, IPC_CHANNELS.core.deleteKnowledge, payload),
      },
      operations: {
        list: (payload: JsonObject) => coreCall(ipc, IPC_CHANNELS.core.listOperations, payload),
        request: (payload: JsonObject) => coreCall(ipc, IPC_CHANNELS.core.requestOperation, payload),
        get: (payload: JsonObject) => coreCall(ipc, IPC_CHANNELS.core.getOperation, payload),
        cancel: (payload: JsonObject) => coreCall(ipc, IPC_CHANNELS.core.cancelOperation, payload),
        listSchedules: (payload: JsonObject) => coreCall(ipc, IPC_CHANNELS.core.listSchedules, payload),
        saveSchedule: (payload: JsonObject) => coreCall(ipc, IPC_CHANNELS.core.saveSchedule, payload),
      },
      workflows: {
        list: (payload: JsonObject) => coreCall(ipc, IPC_CHANNELS.core.listWorkflows, payload),
        save: (payload: JsonObject) => coreCall(ipc, IPC_CHANNELS.core.saveWorkflow, payload),
        run: (payload: JsonObject) => coreCall(ipc, IPC_CHANNELS.core.runWorkflow, payload),
      },
      integrations: {
        listProviders: () => coreCall(ipc, IPC_CHANNELS.core.listCliProviders),
        listConnections: (payload: JsonObject) => coreCall(ipc, IPC_CHANNELS.core.listIntegrationConnections, payload),
        saveConnection: (payload: JsonObject) => coreCall(ipc, IPC_CHANNELS.core.saveIntegrationConnection, payload),
        listResources: (payload: JsonObject) => coreCall(ipc, IPC_CHANNELS.core.listIntegrationResources, payload),
        listSyncRuns: (payload: JsonObject) => coreCall(ipc, IPC_CHANNELS.core.listIntegrationSyncRuns, payload),
      },
      processes: {
        list: (payload: JsonObject) => coreCall(ipc, IPC_CHANNELS.core.listProcesses, payload),
        start: (payload: JsonObject) => coreCall(ipc, IPC_CHANNELS.core.startProcess, payload),
        stop: (payload: JsonObject) => coreCall(ipc, IPC_CHANNELS.core.stopProcess, payload),
        get: (payload: JsonObject) => coreCall(ipc, IPC_CHANNELS.core.getProcess, payload),
      },
      logs: {
        query: (payload: JsonObject) => coreCall(ipc, IPC_CHANNELS.core.queryLogs, payload),
      },
    },
  } as const;
}

const ipcWrapper: BridgeIpc = {
  invoke: (channel, payload) => ipcRenderer.invoke(channel, payload),
  on: (channel, listener) => {
    ipcRenderer.on(channel, listener);
  },
  removeListener: (channel, listener) => {
    ipcRenderer.removeListener(channel, listener);
  },
};

const polterware = createPolterwareBridge(ipcWrapper);

contextBridge.exposeInMainWorld("polterware", polterware);

export type PolterwareBridge = typeof polterware;
