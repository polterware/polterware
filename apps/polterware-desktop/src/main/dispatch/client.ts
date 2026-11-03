import { spawn } from "node:child_process";
import { existsSync, readFileSync } from "node:fs";
import { homedir } from "node:os";
import { delimiter, join } from "node:path";
import { createClient, type Client, type Interceptor } from "@connectrpc/connect";
import { createConnectTransport } from "@connectrpc/connect-node";
import { DispatchService, HealthService } from "@polterware/contracts/dispatch";
import type { JsonObject } from "../../shared/ipc.js";
import { toJsonObject } from "../core/client.js";

// Polterware Dispatch is its own app with its own core (`dispatch serve`).
// Polterware reads a few of its answers (a card's runs, Activity's dispatch
// rows, the Route pill's sources) and starts runs from Commander; everything
// else happens in the Dispatch app, opened through its dispatch:// scheme.

type DispatchCore = { pid: number; endpoint: string; token: string };

/** Where Polterware Dispatch keeps its database: PWDISPATCH_DATA_DIR, else its Application Support folder. */
export function dispatchDataDir(env: NodeJS.ProcessEnv = process.env): string {
  return env.PWDISPATCH_DATA_DIR?.trim() || join(homedir(), "Library", "Application Support", "Polterware Dispatch");
}

/** The dispatch binary: PWDISPATCH_BIN, else the installed app's, else one on PATH. */
export function findDispatchBinary(env: NodeJS.ProcessEnv = process.env): string | null {
  const candidates = [
    env.PWDISPATCH_BIN?.trim(),
    "/Applications/Dispatch.app/Contents/Resources/dispatch",
    join(homedir(), "Applications", "Dispatch.app", "Contents", "Resources", "dispatch"),
    // Installs from before the app was named Dispatch.
    "/Applications/Polterware Dispatch.app/Contents/Resources/dispatch",
    join(homedir(), "Applications", "Polterware Dispatch.app", "Contents", "Resources", "dispatch"),
    ...(env.PATH ?? "").split(delimiter).filter(Boolean).map((dir) => join(dir, "dispatch")),
  ];
  return candidates.find((path): path is string => Boolean(path) && existsSync(path!)) ?? null;
}

function readCore(dataDir: string): DispatchCore | null {
  try {
    const core = JSON.parse(readFileSync(join(dataDir, "run", "core.json"), "utf8")) as Partial<DispatchCore>;
    if (!core.endpoint || !core.token || !core.pid) return null;
    process.kill(core.pid, 0);
    return { pid: core.pid, endpoint: core.endpoint, token: core.token };
  } catch {
    return null;
  }
}

const bearer = (token: string): Interceptor => (next) => async (request) => {
  request.header.set("Authorization", `Bearer ${token}`);
  request.header.set("Polterware-Client", "polterware");
  return next(request);
};

export type DispatchClientOptions = {
  env?: NodeJS.ProcessEnv;
  openExternal?: (url: string) => Promise<unknown>;
};

export function createDispatchClient(options: DispatchClientOptions = {}) {
  const env = options.env ?? process.env;
  const dataDir = dispatchDataDir(env);
  let cached: { core: DispatchCore; client: Client<typeof DispatchService> } | null = null;
  let starting: Promise<DispatchCore | null> | null = null;

  async function healthy(core: DispatchCore): Promise<boolean> {
    try {
      const transport = createConnectTransport({ baseUrl: core.endpoint, httpVersion: "1.1" });
      await createClient(HealthService, transport).check({}, { timeoutMs: 2000 });
      return true;
    } catch {
      return false;
    }
  }

  // start launches the Dispatch core once, detached, when the app is installed.
  async function start(): Promise<DispatchCore | null> {
    const binary = findDispatchBinary(env);
    if (!binary) return null;
    const child = spawn(binary, ["-db", join(dataDir, "dispatch.db"), "serve"], { detached: true, stdio: "ignore", env });
    child.unref();
    for (let attempt = 0; attempt < 50; attempt += 1) {
      await new Promise((resolve) => setTimeout(resolve, 100));
      const core = readCore(dataDir);
      if (core && (await healthy(core))) return core;
    }
    return null;
  }

  async function client(): Promise<Client<typeof DispatchService> | null> {
    const running = readCore(dataDir);
    if (cached && running && running.pid === cached.core.pid && running.token === cached.core.token) return cached.client;
    let core = running && (await healthy(running)) ? running : null;
    if (!core) {
      starting ??= start().finally(() => {
        starting = null;
      });
      core = await starting;
    }
    if (!core) {
      cached = null;
      return null;
    }
    const transport = createConnectTransport({ baseUrl: core.endpoint, httpVersion: "1.1", interceptors: [bearer(core.token)] });
    cached = { core, client: createClient(DispatchService, transport) };
    return cached.client;
  }

  async function required(): Promise<Client<typeof DispatchService>> {
    const found = await client();
    if (!found) throw new Error("Dispatch is not installed.");
    return found;
  }

  const request = (input?: JsonObject) => (input ?? {}) as never;

  return {
    /** Whether Polterware Dispatch is installed (its binary is found or its core runs). */
    available: async () => ({ available: Boolean(readCore(dataDir) || findDispatchBinary(env)) }),
    listRuns: async (input?: JsonObject) => toJsonObject(await (await required()).listRuns(request(input))),
    listHistory: async (input?: JsonObject) => toJsonObject(await (await required()).listHistory(request(input))),
    startRun: async (input?: JsonObject) => toJsonObject(await (await required()).startRun(request(input))),
    listTiers: async (input?: JsonObject) => toJsonObject(await (await required()).listTiers(request(input))),
    listAgents: async (input?: JsonObject) => toJsonObject(await (await required()).listAgents(request(input))),
    listModels: async (input?: JsonObject) => toJsonObject(await (await required()).listModels(request(input))),
    config: async (input?: JsonObject) => toJsonObject(await (await required()).config(request(input))),
    /** Opens a dispatch://run link in the Dispatch app. */
    open: async (input?: JsonObject) => {
      const url = String(input?.url ?? "");
      if (!url.startsWith("dispatch://")) throw new Error("Only dispatch:// links open Dispatch.");
      await options.openExternal?.(url);
      return { ok: true };
    },
  };
}

export type DispatchControllers = ReturnType<typeof createDispatchClient>;
