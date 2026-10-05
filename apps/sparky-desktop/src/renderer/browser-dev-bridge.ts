import { RPC_METHODS } from "../shared/channels";
import type { SparkyBridge } from "../shared/bridge";

/** Protobuf-JSON responses with the repeated fields the shell iterates; everything else resolves to `{}`. */
const SEEDED: Record<string, unknown> = {
  "memories.list": { memories: [], nextCursor: "", total: 0 },
  "minds.list": { minds: [] },
  "tags.list": { tags: [] },
  "settings.get": { settings: {} },
};

function createBrowserDevBridge(): SparkyBridge {
  const bridge: Record<string, unknown> = {};
  for (const [namespace, methods] of Object.entries(RPC_METHODS)) {
    bridge[namespace] = Object.fromEntries(methods.map(method =>
      [method, async () => structuredClone(SEEDED[`${namespace}.${method}`] ?? {})]));
  }
  bridge.onChange = () => () => {};
  bridge.app = {
    name: "Sparky",
    openDeepLink: async () => {},
    onDeepLink: () => () => {},
    getDeepLink: async () => null,
    pickFiles: async () => [],
    pathsForDroppedFiles: async () => [],
    writeTempFile: async () => "",
    saveZip: async () => null,
    saveBackup: async () => null,
    revealInFinder: async () => {},
    startMicrophone: async () => false,
    getInfo: async () => ({ name: "Sparky", version: "0.0.0-browser", updates: "disabled" as const }),
    requestNotificationAuthorization: async () => false,
    openLog: async () => {},
    openDataFolder: async () => {},
    getReduceTransparency: async () => true,
    setReduceTransparency: async (enabled: boolean) => enabled,
    testMcpConnection: async () => ({ ok: false, error: "Unavailable in the browser preview" }),
  };
  return bridge as unknown as SparkyBridge;
}

/** Installs the in-memory preview bridge when Electron preload did not. Returns false if one already exists. */
export function installBrowserDevBridge(): boolean {
  if (window.sparky) return false;
  window.sparky = createBrowserDevBridge();
  return true;
}
