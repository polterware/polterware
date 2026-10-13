import { afterEach, describe, expect, it, vi } from "vitest";
import { DEFAULT_SETTINGS } from "../shared/settings.js";
import type { VitalsSettings } from "../shared/settings.js";

const mock = vi.hoisted(() => ({
  settings: {} as VitalsSettings,
  onStatus: null as null | ((status: string) => void),
  connect: vi.fn(),
  reset: vi.fn(),
}));
vi.mock("./settings.js", () => ({ getSettings: () => mock.settings }));
vi.mock("./storage.js", () => ({
  deleteServer: vi.fn(), getServerById: vi.fn(), getServerSecrets: vi.fn(),
  listServersPublic: vi.fn(), upsertServer: vi.fn(),
}));
vi.mock("./agent-session.js", () => ({
  AgentSession: class {
    constructor(options: { onStatus: (status: string) => void }) { mock.onStatus = options.onStatus; }
    connect = mock.connect;
    resetClient = mock.reset;
    dispose = vi.fn();
  },
}));
import { SessionManager } from "./session-manager.js";

afterEach(() => { vi.useRealTimers(); vi.clearAllMocks(); });

function start(settings: Partial<VitalsSettings>) {
  vi.useFakeTimers();
  mock.settings = { ...DEFAULT_SETTINGS, ...settings };
  const sessions = new SessionManager();
  const send = vi.fn();
  sessions.setWindow({ webContents: { send } } as never);
  const { sessionId } = sessions.connect({ host: "host", port: 22, username: "user", authMode: "password" });
  return { sessions, sessionId, send };
}

describe("connection settings", () => {
  it("uses the configured delay and checks auto-reconnect again before retrying", () => {
    start({ reconnectDelaySeconds: 5 });
    mock.onStatus!("disconnected");
    vi.advanceTimersByTime(4999);
    expect(mock.connect).toHaveBeenCalledTimes(1);
    vi.advanceTimersByTime(1);
    expect(mock.connect).toHaveBeenCalledTimes(2);
    mock.onStatus!("disconnected");
    mock.settings.autoReconnect = false;
    vi.advanceTimersByTime(5000);
    expect(mock.connect).toHaveBeenCalledTimes(2);
  });
  it("does not schedule reconnects when disabled or after explicit disconnect", () => {
    const { sessions, sessionId } = start({ autoReconnect: false });
    mock.onStatus!("disconnected");
    expect(vi.getTimerCount()).toBe(0);
    mock.settings.autoReconnect = true;
    mock.onStatus!("disconnected");
    expect(vi.getTimerCount()).toBe(1);
    sessions.disconnect(sessionId);
    expect(vi.getTimerCount()).toBe(0);
    vi.advanceTimersByTime(3000);
    expect(mock.connect).toHaveBeenCalledTimes(1);
  });
});
