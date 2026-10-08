import type { IpcMain, IpcMainInvokeEvent } from "electron";
import type { ConnectPayload, KillProcessPayload } from "../shared/bridge.js";
import { IPC_CHANNELS } from "../shared/channels.js";
import type { SessionManager } from "./session-manager.js";
import { assertSessionId, parseShellAck, parseShellOpen, parseShellResize, parseShellWrite } from "./shell-session.js";
import { readTheme, writeTheme } from "./theme.js";

type IpcHandle = (event: IpcMainInvokeEvent, payload?: unknown) => unknown | Promise<unknown>;

export type VitalsSessionService = Pick<
  SessionManager,
  | "connect"
  | "disconnect"
  | "listServers"
  | "deleteServer"
  | "acceptHostKey"
  | "rejectHostKey"
  | "killProcess"
  | "shellOpen"
  | "shellWrite"
  | "shellResize"
  | "shellClose"
  | "shellAck"
>;

export function createVitalsIpcHandlers(deps: {
  sessions: VitalsSessionService;
}): Record<string, IpcHandle> {
  const { sessions } = deps;
  const c = IPC_CHANNELS;
  return {
    [c.session.connect]: (_e, payload) => sessions.connect(payload as ConnectPayload),
    [c.session.disconnect]: (_e, sessionId) => {
      sessions.disconnect(sessionId as string);
    },
    [c.session.listServers]: () => sessions.listServers(),
    [c.session.deleteServer]: (_e, id) => {
      sessions.deleteServer(id as string);
    },
    [c.session.hostKeyAccept]: (_e, sessionId) => {
      sessions.acceptHostKey(sessionId as string);
    },
    [c.session.hostKeyReject]: (_e, sessionId) => {
      sessions.rejectHostKey(sessionId as string);
    },
    [c.session.killProcess]: (_e, payload) => {
      const p = payload as KillProcessPayload;
      return sessions.killProcess(p.sessionId, p.pid, p.signal ?? "TERM");
    },
    [c.shell.open]: (_e, payload) => {
      const p = parseShellOpen(payload);
      return sessions.shellOpen(p.sessionId, p.cols, p.rows);
    },
    [c.shell.write]: (_e, payload) => {
      const p = parseShellWrite(payload);
      sessions.shellWrite(p.sessionId, p.data);
    },
    [c.shell.resize]: (_e, payload) => {
      const p = parseShellResize(payload);
      sessions.shellResize(p.sessionId, p.cols, p.rows);
    },
    [c.shell.close]: (_e, sessionId) => {
      sessions.shellClose(assertSessionId(sessionId));
    },
    [c.shell.ack]: (_e, payload) => {
      const p = parseShellAck(payload);
      sessions.shellAck(p.sessionId, p.bytes);
    },
    [c.theme.get]: () => readTheme(),
    [c.theme.set]: (_e, theme) => {
      writeTheme(theme as "dark" | "light");
    },
  };
}

export function registerVitalsIpcHandlers(
  deps: { sessions: VitalsSessionService },
  ipcMain: Pick<IpcMain, "handle">,
): void {
  const handlers = createVitalsIpcHandlers(deps);
  for (const [channel, handler] of Object.entries(handlers)) {
    ipcMain.handle(channel, handler);
  }
}
