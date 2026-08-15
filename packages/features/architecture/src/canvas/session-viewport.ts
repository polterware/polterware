import type { SelectionState, Viewport } from "@polterware/frontend-canvas";

export type SessionSnapshot = {
  viewport: Viewport;
  selection: SelectionState;
  environmentId?: string | "all";
};

const cache = new Map<string, SessionSnapshot>();

export function setSessionSnapshot(systemId: string, snap: SessionSnapshot): void {
  cache.set(systemId, structuredClone(snap));
}

export function getSessionSnapshot(systemId: string): SessionSnapshot | undefined {
  const s = cache.get(systemId);
  return s ? structuredClone(s) : undefined;
}

export function clearSessionSnapshot(systemId: string): void {
  cache.delete(systemId);
}
