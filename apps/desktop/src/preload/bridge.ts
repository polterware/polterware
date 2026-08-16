import { contextBridge, ipcRenderer } from "electron";
import { IPC_CHANNELS, IPC_EVENTS, type AppIpcRequests, type AppIpcResponses } from "../shared/ipc.js";

type BridgeListener = () => void;
type BridgeIpc = {
  invoke<K extends keyof AppIpcRequests>(channel: K, payload?: AppIpcRequests[K]): Promise<AppIpcResponses[K]>;
  on?(channel: string, listener: BridgeListener): void;
  removeListener?(channel: string, listener: BridgeListener): void;
};

/** Minimal typed bridge — no raw ipcRenderer, no skills channels */
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
