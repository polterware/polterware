import type { AppIconOption, AppIconSelection } from "./app-icons.js";

export interface AppInfo {
  name: string;
  version: string;
}

export type JsonValue = string | number | boolean | null | JsonObject | JsonValue[];
export interface JsonObject {
  [key: string]: JsonValue;
}

/** Minimal Desktop IPC catalog for local visual chrome. */
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
} as const;

export type IpcChannelTree = typeof IPC_CHANNELS;

export const IPC_EVENTS = {
  commander: {
    focusSearch: "commander:focus-search",
  },
} as const;

export type IpcEventTree = typeof IPC_EVENTS;

export interface AppIpcRequests {
  [IPC_CHANNELS.app.getInfo]: undefined;
  [IPC_CHANNELS.app.listIcons]: undefined;
  [IPC_CHANNELS.app.getIcon]: undefined;
  [IPC_CHANNELS.app.setIcon]: JsonValue;
  [IPC_CHANNELS.commander.hideOverlay]: undefined;
  [IPC_CHANNELS.commander.showMainWindow]: undefined;
}

export interface AppIpcResponses {
  [IPC_CHANNELS.app.getInfo]: AppInfo;
  [IPC_CHANNELS.app.listIcons]: AppIconOption[];
  [IPC_CHANNELS.app.getIcon]: AppIconSelection;
  [IPC_CHANNELS.app.setIcon]: AppIconSelection;
  [IPC_CHANNELS.commander.hideOverlay]: void;
  [IPC_CHANNELS.commander.showMainWindow]: void;
}
