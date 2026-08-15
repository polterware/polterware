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
