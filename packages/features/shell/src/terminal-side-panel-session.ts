import type { LiveTerminalSession } from "./live-terminal-session";
import type { TerminalTargetItem } from "./types";
import { canOpenTerminalSessionOnTarget } from "./terminal-session-model";

export type { LiveTerminalSession } from "./live-terminal-session";

export const DRAWER_TERMINAL_COLS = 80;
export const DRAWER_TERMINAL_ROWS = 24;

export type CreateDrawerTerminalSession = (input: {
  projectId: string;
  targetId: string;
  cols: number;
  rows: number;
}) => Promise<LiveTerminalSession>;

export type DrawerTerminalCallbacks = {
  projectId?: string;
  onCreateSession?: CreateDrawerTerminalSession;
  onWriteSession?: (id: string, data: string) => Promise<void>;
  onResizeSession?: (id: string, cols: number, rows: number) => Promise<void>;
  onCloseSession?: (id: string) => Promise<void>;
  onReadSession?: (id: string, offset: number) => Promise<{
    data: string;
    nextOffset: number;
    closed: boolean;
  }>;
};

export function hasLiveTerminalBridge(callbacks: DrawerTerminalCallbacks): boolean {
  return Boolean(
    callbacks.projectId &&
      callbacks.onCreateSession &&
      callbacks.onWriteSession &&
      callbacks.onReadSession,
  );
}

export function resolveDrawerTarget(
  targets: readonly TerminalTargetItem[],
  targetId: string | null,
): TerminalTargetItem | null {
  if (!targetId) return null;
  return targets.find((target) => target.id === targetId) ?? null;
}

export async function openDrawerTerminalSession(input: {
  callbacks: DrawerTerminalCallbacks;
  target: TerminalTargetItem;
  cols?: number;
  rows?: number;
}): Promise<LiveTerminalSession> {
  const { callbacks, target } = input;
  if (!hasLiveTerminalBridge(callbacks) || !callbacks.onCreateSession || !callbacks.projectId) {
    throw new Error("Local terminal is unavailable.");
  }
  if (!canOpenTerminalSessionOnTarget(target.status)) {
    throw new Error(`Target ${target.name} is not usable for a terminal session.`);
  }
  return callbacks.onCreateSession({
    projectId: callbacks.projectId,
    targetId: target.id,
    cols: input.cols ?? DRAWER_TERMINAL_COLS,
    rows: input.rows ?? DRAWER_TERMINAL_ROWS,
  });
}

export async function closeDrawerTerminalSession(
  callbacks: DrawerTerminalCallbacks,
  sessionId: string | null | undefined,
): Promise<void> {
  if (!sessionId || !callbacks.onCloseSession) return;
  try {
    await callbacks.onCloseSession(sessionId);
  } catch {
    // Session may already be closed.
  }
}
