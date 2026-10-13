import type { LoreBridge } from "../../preload/bridge.js";

declare global {
  interface Window {
    lore: LoreBridge;
  }
}

export {};
