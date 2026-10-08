// Frozen vitals-v1 IPC allowlist.
export const IPC_CHANNELS = {
  session: {
    connect: "vitals:connect",
    disconnect: "vitals:disconnect",
    listServers: "vitals:list-servers",
    deleteServer: "vitals:delete-server",
    hostKeyAccept: "vitals:host-key-accept",
    hostKeyReject: "vitals:host-key-reject",
    killProcess: "vitals:kill-process",
  },
  theme: {
    get: "vitals:get-theme",
    set: "vitals:set-theme",
  },
} as const;

export const IPC_EVENTS = {
  connection: {
    state: "vitals:connection-state",
  },
  metrics: {
    snapshot: "vitals:metrics",
  },
  hostKey: {
    prompt: "vitals:host-key-prompt",
  },
} as const;
