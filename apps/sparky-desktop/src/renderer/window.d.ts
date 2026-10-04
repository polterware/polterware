export interface SparkyBridge {
  app: {
    name: string;
  };
}

declare global {
  interface Window {
    sparky: SparkyBridge;
  }
}

export {};
