import type { SparkyBridge } from "../shared/bridge.js";

declare global {
  interface Window {
    sparky: SparkyBridge;
  }
}

export {};
