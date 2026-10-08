import { randomUUID } from "node:crypto";
import type { BrowserWindow } from "electron";
import type {
  ConnectPayload,
  ConnectionStateEvent,
  HostKeyPrompt,
  MetricsEvent,
  ShellDataEvent,
  ShellExitEvent,
} from "../shared/bridge.js";
import { IPC_EVENTS } from "../shared/channels.js";
import {
  deleteServer,
  getServerById,
  getServerSecrets,
  listServersPublic,
  upsertServer,
} from "./storage.js";
import { SshSession, type SessionAuth } from "./ssh-session.js";

interface ManagedSession {
  id: string;
  label: string;
  host: string;
  port: number;
  username: string;
  auth: SessionAuth;
  ssh: SshSession;
  reconnectTimer: ReturnType<typeof setTimeout> | null;
  pendingFingerprint: string | null;
  allowReconnect: boolean;
}

export class SessionManager {
  private sessions = new Map<string, ManagedSession>();
  private window: BrowserWindow | null = null;

  setWindow(win: BrowserWindow): void {
    this.window = win;
  }

  listServers() {
    return listServersPublic();
  }

  deleteServer(id: string) {
    deleteServer(id);
  }

  private emit(channel: string, payload: unknown): void {
    this.window?.webContents.send(channel, payload);
  }

  connect(payload: ConnectPayload): { sessionId: string } {
    const sessionId = randomUUID();
    let auth: SessionAuth = {
      password: payload.password,
      privateKeyPath: payload.privateKeyPath,
      privateKeyPassphrase: payload.privateKeyPassphrase,
    };

    if (payload.serverId) {
      const pub = getServerById(payload.serverId);
      const secrets = getServerSecrets(payload.serverId);
      if (pub && secrets) {
        auth = {
          password: secrets.password,
          privateKeyPath: pub.privateKeyPath,
          privateKeyPassphrase: secrets.privateKeyPassphrase,
        };
      }
    }

    const label = payload.label || payload.host || `${payload.username}@${payload.host}`;

    const sendState = (status: ConnectionStateEvent["status"], message?: string) => {
      this.emit(IPC_EVENTS.connection.state, {
        sessionId,
        status,
        message,
        label,
        host: payload.host,
      } satisfies ConnectionStateEvent);
    };

    const ssh = new SshSession({
      sessionId,
      host: payload.host,
      port: payload.port,
      username: payload.username,
      auth,
      onMetrics: (snapshot, collectedAt) => {
        this.emit(IPC_EVENTS.metrics.snapshot, {
          sessionId,
          snapshot,
          collectedAt,
        } satisfies MetricsEvent);
      },
      onStatus: (status, message) => {
        const mapped =
          status === "connected"
            ? "connected"
            : status === "connecting"
              ? "connecting"
              : status === "disconnected"
                ? "disconnected"
                : status === "error"
                  ? "error"
                  : "connecting";
        sendState(mapped as ConnectionStateEvent["status"], message);

        const managed = this.sessions.get(sessionId);
        if (mapped === "disconnected" && managed && managed.allowReconnect && !managed.reconnectTimer) {
          sendState("reconnecting", "Reconnecting in 3s…");
          managed.reconnectTimer = setTimeout(() => {
            managed.reconnectTimer = null;
            if (!this.sessions.has(sessionId)) return;
            managed.ssh.resetClient();
            sendState("reconnecting");
            managed.ssh.connect();
          }, 3000);
        }
      },
      onHostKeyPrompt: (fingerprint) => {
        const managed = this.sessions.get(sessionId);
        if (managed) managed.pendingFingerprint = fingerprint;
        this.emit(IPC_EVENTS.hostKey.prompt, {
          sessionId,
          host: payload.host,
          port: payload.port,
          fingerprint,
        } satisfies HostKeyPrompt);
      },
    });

    this.sessions.set(sessionId, {
      id: sessionId,
      label,
      host: payload.host,
      port: payload.port,
      username: payload.username,
      auth,
      ssh,
      reconnectTimer: null,
      pendingFingerprint: null,
      allowReconnect: true,
    });

    sendState("connecting");
    ssh.connect();

    if (payload.save) {
      upsertServer(
        {
          id: payload.serverId,
          label,
          host: payload.host,
          port: payload.port,
          username: payload.username,
          authMode: payload.authMode,
          privateKeyPath: payload.privateKeyPath,
        },
        {
          password: payload.password,
          privateKeyPassphrase: payload.privateKeyPassphrase,
        },
      );
    }

    return { sessionId };
  }

  disconnect(sessionId: string): void {
    const managed = this.sessions.get(sessionId);
    if (!managed) return;
    managed.allowReconnect = false;
    if (managed.reconnectTimer) clearTimeout(managed.reconnectTimer);
    managed.ssh.dispose();
    this.sessions.delete(sessionId);
    this.emit(IPC_EVENTS.connection.state, {
      sessionId,
      status: "disconnected",
      label: managed.label,
      host: managed.host,
    } satisfies ConnectionStateEvent);
  }

  acceptHostKey(sessionId: string): void {
    const managed = this.sessions.get(sessionId);
    if (!managed || !managed.pendingFingerprint) return;
    managed.ssh.approveHostKey(managed.pendingFingerprint);
    managed.pendingFingerprint = null;
    managed.ssh.resetClient();
    managed.ssh.connect();
  }

  rejectHostKey(sessionId: string): void {
    this.disconnect(sessionId);
  }

  async shellOpen(sessionId: string, cols: number, rows: number): Promise<void> {
    const managed = this.sessions.get(sessionId);
    if (!managed) throw new Error("Session not found");
    await managed.ssh.openShell(
      cols,
      rows,
      (data) => this.emit(IPC_EVENTS.shell.data, { sessionId, data } satisfies ShellDataEvent),
      () => this.emit(IPC_EVENTS.shell.exit, { sessionId } satisfies ShellExitEvent),
    );
  }

  shellWrite(sessionId: string, data: string): void {
    this.sessions.get(sessionId)?.ssh.writeShell(data);
  }

  shellResize(sessionId: string, cols: number, rows: number): void {
    this.sessions.get(sessionId)?.ssh.resizeShell(cols, rows);
  }

  shellClose(sessionId: string): void {
    this.sessions.get(sessionId)?.ssh.closeShell();
  }

  async killProcess(sessionId: string, pid: number, signal: "TERM" | "KILL"): Promise<void> {
    const managed = this.sessions.get(sessionId);
    if (!managed) throw new Error("Session not found");
    await managed.ssh.killProcess(pid, signal);
  }
}
