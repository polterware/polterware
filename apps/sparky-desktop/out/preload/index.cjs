"use strict";
const electron = require("electron");
const RPC_METHODS = {
  "memories": [
    "list",
    "get",
    "create",
    "update",
    "delete",
    "setStatus",
    "setPinned",
    "reorder",
    "toggleCheckItem",
    "replaceChecklist",
    "setTags",
    "bulkAction"
  ],
  "minds": [
    "list",
    "get",
    "create",
    "update",
    "delete",
    "reorder"
  ],
  "tags": [
    "list",
    "create",
    "update",
    "delete",
    "merge"
  ],
  "schedule": [
    "set",
    "clear",
    "preview",
    "listOccurrences",
    "completeOccurrence",
    "reopenOccurrence",
    "setFocusRecipe"
  ],
  "places": [
    "set",
    "clear",
    "listInBounds"
  ],
  "attachments": [
    "importFromPath",
    "addLink",
    "list",
    "delete",
    "reorder",
    "openInfo"
  ],
  "focus": [
    "getDefaults",
    "setDefaults",
    "recordSession",
    "listSessions"
  ],
  "review": [
    "get"
  ],
  "search": [
    "search",
    "status",
    "reindex"
  ],
  "reminders": [
    "plan",
    "ack",
    "snooze",
    "armOneOff",
    "cancelOneOff",
    "heartbeat",
    "status"
  ],
  "activity": [
    "list"
  ],
  "settings": [
    "get",
    "set"
  ],
  "agents": [
    "getInfo",
    "rotateToken",
    "setEnabled",
    "setAllowFileAttach"
  ],
  "maintenance": [
    "export",
    "backup",
    "doctor"
  ]
};
const EVENTS = { change: "sparky:change", deepLink: "sparky:deep-link" };
const APP_METHODS = ["openDeepLink", "getDeepLink", "pickFiles", "saveZip", "saveBackup", "revealInFinder", "startMicrophone", "getInfo"];
const bridge = {};
for (const [namespace, methods] of Object.entries(RPC_METHODS)) {
  bridge[namespace] = Object.fromEntries(methods.map((method) => [
    method,
    (input = {}) => electron.ipcRenderer.invoke(`sparky:${namespace}:${method}`, input)
  ]));
}
function subscribe(channel, callback) {
  const listener = (_event, value) => callback(value);
  electron.ipcRenderer.on(channel, listener);
  return () => electron.ipcRenderer.removeListener(channel, listener);
}
bridge.onChange = (callback) => subscribe(EVENTS.change, callback);
bridge.app = {
  name: "Sparky",
  ...Object.fromEntries(APP_METHODS.map((method) => [method, (input) => electron.ipcRenderer.invoke(`sparky:app:${method}`, input)])),
  onDeepLink: (callback) => subscribe(EVENTS.deepLink, callback)
};
electron.contextBridge.exposeInMainWorld("sparky", bridge);
