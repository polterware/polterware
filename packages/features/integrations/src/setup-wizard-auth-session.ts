import type { CliConnectionState } from "@polterware/feature-shell";

export const AUTH_PROBE_INTERVAL_MS = 2_000;
export const AUTH_PROBE_TIMEOUT_MS = 5 * 60_000;
export const AUTH_COMMAND_DELAY_MS = 300;
export const AUTH_DEFAULT_COLS = 100;
export const AUTH_DEFAULT_ROWS = 28;

export type AuthTerminalBridge = {
  create: (payload: Record<string, unknown>) => Promise<Record<string, unknown>>;
  write: (payload: Record<string, unknown>) => Promise<unknown>;
  resize: (payload: Record<string, unknown>) => Promise<unknown>;
  close: (payload: Record<string, unknown>) => Promise<unknown>;
  read: (payload: Record<string, unknown>) => Promise<Record<string, unknown>>;
};

export type AuthIntegrationsBridge = {
  probeProvider: (payload: Record<string, unknown>) => Promise<Record<string, unknown>>;
  beginAuth: (payload: Record<string, unknown>) => Promise<Record<string, unknown>>;
};

export type AuthProbeSnapshot = {
  status: CliConnectionState;
  accountLabel: string;
  message: string;
  binaryPath: string;
};

function bytesToBase64(bytes: Uint8Array): string {
  let binary = "";
  for (const value of bytes) binary += String.fromCharCode(value);
  return btoa(binary);
}

function utf8ToBase64(value: string): string {
  return bytesToBase64(new TextEncoder().encode(value));
}

function base64ToUtf8(value: string): string {
  if (!value) return "";
  const binary = atob(value);
  const bytes = Uint8Array.from(binary, (char) => char.charCodeAt(0));
  return new TextDecoder().decode(bytes);
}

function coerceBase64(data: unknown): string {
  if (typeof data === "string") return data;
  if (ArrayBuffer.isView(data)) {
    return bytesToBase64(new Uint8Array(data.buffer, data.byteOffset, data.byteLength));
  }
  if (data && typeof data === "object") {
    const record = data as Record<string, unknown>;
    if ("data" in record) return coerceBase64(record.data);
    const keys = Object.keys(record);
    if (keys.length > 0 && keys.every((key) => /^\d+$/.test(key))) {
      const bytes = new Uint8Array(keys.length);
      for (const key of keys) bytes[Number(key)] = Number(record[key]) & 0xff;
      return bytesToBase64(bytes);
    }
  }
  return "";
}

function asRecord(value: unknown): Record<string, unknown> {
  return value && typeof value === "object" ? (value as Record<string, unknown>) : {};
}

function normalizeStatus(value: string): CliConnectionState {
  if (
    value === "cli-unavailable" ||
    value === "connected" ||
    value === "failed" ||
    value === "ready" ||
    value === "authenticating"
  ) {
    return value;
  }
  return "ready";
}

export function authTargetId(providerId: string): string {
  return `auth-${providerId}`;
}

export async function readAuthTerminalOutput(
  terminals: AuthTerminalBridge,
  id: string,
  offset: number,
): Promise<{ data: string; nextOffset: number; closed: boolean }> {
  if (!id) {
    return { data: "", nextOffset: offset, closed: true };
  }
  try {
    const response = asRecord(
      await terminals.read({
        id,
        offset,
      }),
    );
    return {
      data: base64ToUtf8(coerceBase64(response.data)),
      nextOffset: Number(response.nextOffset ?? offset),
      closed: Boolean(response.closed),
    };
  } catch (error: unknown) {
    const message = error instanceof Error ? error.message.toLowerCase() : String(error).toLowerCase();
    if (
      message.includes("terminal session not found") ||
      message.includes("not_found") ||
      message.includes("not found")
    ) {
      return { data: "", nextOffset: offset, closed: true };
    }
    throw error;
  }
}

export async function writeAuthTerminal(
  terminals: AuthTerminalBridge,
  id: string,
  data: string,
): Promise<void> {
  if (!id) return;
  try {
    await terminals.write({
      id,
      data: utf8ToBase64(data),
    });
  } catch (error: unknown) {
    const message = error instanceof Error ? error.message.toLowerCase() : String(error).toLowerCase();
    if (
      message.includes("terminal session not found") ||
      message.includes("not_found") ||
      message.includes("not found")
    ) {
      return;
    }
    throw error;
  }
}

export async function resizeAuthTerminal(
  terminals: AuthTerminalBridge,
  id: string,
  cols: number,
  rows: number,
): Promise<void> {
  if (!id) return;
  try {
    await terminals.resize({ id, cols, rows });
  } catch (error: unknown) {
    const message = error instanceof Error ? error.message.toLowerCase() : String(error).toLowerCase();
    if (
      message.includes("terminal session not found") ||
      message.includes("not_found") ||
      message.includes("not found")
    ) {
      return;
    }
    throw error;
  }
}

export async function closeAuthTerminal(
  terminals: AuthTerminalBridge,
  id: string | null | undefined,
): Promise<void> {
  if (!id) return;
  try {
    await terminals.close({ id });
  } catch {
    // Session may already be closed.
  }
}

export async function createAuthTerminalSession(input: {
  terminals: AuthTerminalBridge;
  projectId: string;
  providerId: string;
  cols?: number;
  rows?: number;
}): Promise<string> {
  const response = asRecord(
    await input.terminals.create({
      projectId: input.projectId,
      targetId: authTargetId(input.providerId),
      cols: input.cols ?? AUTH_DEFAULT_COLS,
      rows: input.rows ?? AUTH_DEFAULT_ROWS,
    }),
  );
  const session = asRecord(response.session);
  const id = String(session.id ?? "");
  if (!id) throw new Error("Terminal session id missing.");
  return id;
}

export async function beginAuthPlan(
  integrations: AuthIntegrationsBridge,
  providerId: string,
): Promise<{ authCommand: string; docsUrl: string; message: string }> {
  const plan = await integrations.beginAuth({ providerId });
  return {
    authCommand: String(plan.authCommand ?? ""),
    docsUrl: String(plan.docsUrl ?? ""),
    message: String(plan.message ?? ""),
  };
}

export async function probeAuthStatus(
  integrations: AuthIntegrationsBridge,
  providerId: string,
): Promise<AuthProbeSnapshot> {
  const result = await integrations.probeProvider({ providerId });
  return {
    status: normalizeStatus(String(result.status ?? "ready")),
    accountLabel: String(result.accountLabel ?? ""),
    message: String(result.message ?? ""),
    binaryPath: String(result.binaryPath ?? ""),
  };
}

export type AuthProbeLoopHandlers = {
  onSnapshot: (snapshot: AuthProbeSnapshot) => void;
  onConnected: (snapshot: AuthProbeSnapshot) => void;
  onUnavailable: (snapshot: AuthProbeSnapshot) => void;
  onTimeout: () => void;
};

export function startAuthProbeLoop(input: {
  integrations: AuthIntegrationsBridge;
  providerId: string;
  intervalMs?: number;
  timeoutMs?: number;
  handlers: AuthProbeLoopHandlers;
}): () => void {
  const intervalMs = input.intervalMs ?? AUTH_PROBE_INTERVAL_MS;
  const timeoutMs = input.timeoutMs ?? AUTH_PROBE_TIMEOUT_MS;
  const startedAt = Date.now();
  let timer: ReturnType<typeof setTimeout> | undefined;
  let stopped = false;

  const stop = () => {
    stopped = true;
    if (timer !== undefined) clearTimeout(timer);
  };

  const tick = async () => {
    if (stopped) return;
    if (Date.now() - startedAt >= timeoutMs) {
      stop();
      input.handlers.onTimeout();
      return;
    }
    try {
      const snapshot = await probeAuthStatus(input.integrations, input.providerId);
      if (stopped) return;
      input.handlers.onSnapshot(snapshot);
      if (snapshot.status === "connected") {
        stop();
        input.handlers.onConnected(snapshot);
        return;
      }
      if (snapshot.status === "cli-unavailable") {
        stop();
        input.handlers.onUnavailable(snapshot);
        return;
      }
    } catch {
      // Keep polling through transient probe errors.
    }
    if (stopped) return;
    timer = setTimeout(() => {
      void tick();
    }, intervalMs);
  };

  void tick();
  return stop;
}

export async function startAuthCommand(input: {
  terminals: AuthTerminalBridge;
  sessionId: string;
  authCommand: string;
  delayMs?: number;
}): Promise<void> {
  const delay = input.delayMs ?? AUTH_COMMAND_DELAY_MS;
  if (delay > 0) {
    await new Promise((resolve) => setTimeout(resolve, delay));
  }
  const command = input.authCommand.trim();
  if (!command) return;
  await writeAuthTerminal(input.terminals, input.sessionId, `${command}\n`);
}
