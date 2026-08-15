import { contextBridge, ipcRenderer } from "electron";
import { IPC_CHANNELS, IPC_EVENTS } from "../shared/ipc.js";

type BridgeListener = (...args: unknown[]) => void;
type BridgeIpc = {
  invoke(channel: string, payload?: unknown): Promise<unknown>;
  on?(channel: string, listener: BridgeListener): unknown;
  removeListener?(channel: string, listener: BridgeListener): unknown;
};

/** Minimal typed bridge — no raw ipcRenderer, no skills channels */
export function createPolterwareBridge(ipc: BridgeIpc) {
  return {
    app: {
      getInfo: (): Promise<{ name: string; version: string }> =>
        ipc.invoke(IPC_CHANNELS.app.getInfo) as Promise<{ name: string; version: string }>,
      listIcons: (): Promise<unknown> => ipc.invoke(IPC_CHANNELS.app.listIcons),
      getIcon: (): Promise<unknown> => ipc.invoke(IPC_CHANNELS.app.getIcon),
      setIcon: (id: string): Promise<unknown> =>
        ipc.invoke(IPC_CHANNELS.app.setIcon, { id }),
    },
    commander: {
      hideOverlay: (): Promise<void> =>
        ipc.invoke(IPC_CHANNELS.commander.hideOverlay) as Promise<void>,
      showMainWindow: (): Promise<void> =>
        ipc.invoke(IPC_CHANNELS.commander.showMainWindow) as Promise<void>,
      onFocusSearch: (callback: () => void) => {
        const listener = () => callback();
        ipc.on?.(IPC_EVENTS.commander.focusSearch, listener);
        return () => ipc.removeListener?.(IPC_EVENTS.commander.focusSearch, listener);
      },
    },
  } as const;
}

const polterware = createPolterwareBridge(ipcRenderer);

contextBridge.exposeInMainWorld("polterware", polterware);

export type PolterwareBridge = typeof polterware;
