import { contextBridge, ipcRenderer } from "electron";
import { RPC_METHODS } from "../shared/channels.js";
import { APP_METHODS, EVENTS, type SparkyBridge } from "../shared/bridge.js";

const bridge: Record<string, unknown> = {};
for (const [namespace, methods] of Object.entries(RPC_METHODS)) {
  bridge[namespace] = Object.fromEntries(methods.map(method => [method,
    (input: unknown = {}) => ipcRenderer.invoke(`sparky:${namespace}:${method}`, input)]));
}
function subscribe<T>(channel: string, callback: (event: T) => void): () => void {
  const listener = (_event: unknown, value: T) => callback(value);
  ipcRenderer.on(channel, listener);
  return () => ipcRenderer.removeListener(channel, listener);
}
bridge.onChange = (callback: Parameters<SparkyBridge["onChange"]>[0]) => subscribe(EVENTS.change, callback);
bridge.app = {
  name: "Sparky",
  ...Object.fromEntries(APP_METHODS.map(method => [method, (input?: unknown) => ipcRenderer.invoke(`sparky:app:${method}`, input)])),
  onDeepLink: (callback: Parameters<SparkyBridge["app"]["onDeepLink"]>[0]) => {
    const seen = new Set<string>();
    const deliver = (link: { url: string; route: string }) => {
      if (seen.has(link.url)) return;
      seen.add(link.url);
      callback(link);
    };
    const unsubscribe = subscribe(EVENTS.deepLink, deliver);
    void ipcRenderer.invoke("sparky:app:getDeepLink").then((link: { url: string; route: string } | null) => {
      if (link) deliver(link);
    });
    return unsubscribe;
  },
};
contextBridge.exposeInMainWorld("sparky", bridge as unknown as SparkyBridge);
