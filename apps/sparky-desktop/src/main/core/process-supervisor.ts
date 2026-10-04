import { spawn } from "node:child_process";
import { closeSync, existsSync, mkdirSync, openSync, readFileSync, realpathSync, statSync } from "node:fs";
import { dirname, isAbsolute, join, resolve } from "node:path";
import { createCoreClients, validateEndpoint } from "./client.js";

export type CoreHandle = { endpoint: string; token: string; pid?: number };
export function buildId(binary: string): string {
  const info = statSync(binary);
  return `${Math.floor(info.mtimeMs / 1000)}-${info.size}`;
}
export function isNewerBuild(local: string, running: string): boolean {
  return Number(local.split("-")[0]) > Number(running.split("-")[0]);
}
export function readCoreState(dbPath: string): CoreHandle | undefined {
  try {
    const statePath = join(dirname(dbPath), "run/core.json");
    const info = statSync(statePath);
    if (!info.isFile() || info.size > 16384 || (info.mode & 0o077) !== 0 || info.uid !== process.getuid?.()) return;
    const state = JSON.parse(readFileSync(statePath, "utf8"));
    if (!Number.isSafeInteger(state.pid) || state.pid <= 1 || typeof state.token !== "string" || !state.token ||
      typeof state.dbPath !== "string" || realpathSync(state.dbPath) !== realpathSync(dbPath)) return;
    process.kill(state.pid, 0);
    return { endpoint: validateEndpoint(state.endpoint), token: state.token, pid: state.pid };
  } catch { return; }
}
export function resolveBinary(appPath: string, resourcesPath: string, packaged: boolean): string {
  const override = process.env.SPARKY_CORE_BIN?.trim();
  if (override) {
    if (!isAbsolute(override)) throw new Error("SPARKY_CORE_BIN must be absolute");
    if (!existsSync(override)) throw new Error("Sparky core binary is missing");
    return override;
  }
  const binary = packaged ? join(resourcesPath, "sparky") : resolve(appPath, "../../dist/sparky");
  if (!existsSync(binary)) throw new Error("Sparky core binary is missing; run pnpm build:sparky-core");
  return binary;
}
export async function ensureCore(options: { dbPath: string; binary: string }): Promise<CoreHandle> {
  const endpoint = process.env.SPARKY_CORE_ENDPOINT?.trim();
  const token = process.env.SPARKY_CORE_TOKEN?.trim();
  if (endpoint || token) {
    if (!endpoint || !token) throw new Error("Both SPARKY_CORE_ENDPOINT and SPARKY_CORE_TOKEN are required");
    const handle = { endpoint: validateEndpoint(endpoint), token };
    await createCoreClients(handle.endpoint, token).health.check({}, { timeoutMs: 2000 });
    return handle;
  }
  const localBuild = buildId(options.binary);
  let spawned = false;
  for (let attempt = 0; attempt < 60; attempt++) {
    const state = readCoreState(options.dbPath);
    if (state) {
      try {
        const health = await createCoreClients(state.endpoint, state.token).health.check({}, { timeoutMs: 1000 });
        if (realpathSync(health.dbPath) !== realpathSync(options.dbPath)) throw new Error("Core database mismatch");
        if (isNewerBuild(localBuild, health.build)) {
          // Frozen MaintenanceService has no Shutdown. Signal only after authenticated identity validation.
          process.kill(state.pid!, "SIGTERM");
          for (let wait = 0; wait < 50; wait++) {
            await delay(100);
            try { process.kill(state.pid!, 0); } catch { break; }
          }
          if (readCoreState(options.dbPath)) throw new Error("Older core did not exit");
          spawned = false;
        } else return state;
      } catch { /* A daemon may still be starting. Never kill an unverified process. */ }
    } else if (!spawned) {
      await spawnCore(options.dbPath, options.binary);
      spawned = true;
    }
    await delay(200);
  }
  throw new Error("Core unavailable; inspect Sparky/run/core.log");
}
async function spawnCore(dbPath: string, binary: string): Promise<void> {
  const run = join(dirname(dbPath), "run");
  mkdirSync(run, { recursive: true, mode: 0o700 });
  const fd = openSync(join(run, "core.log"), "a", 0o600);
  try {
    const env = { ...process.env };
    delete env.SPARKY_CORE_TOKEN;
    delete env.POLTERWARE_SESSION_TOKEN;
    const child = spawn(binary, ["-db", dbPath, "serve", "--listen", "127.0.0.1:0"],
      { detached: true, stdio: ["ignore", fd, fd], env });
    await new Promise<void>((resolveSpawn, reject) => {
      child.once("error", reject);
      child.once("spawn", resolveSpawn);
    });
    child.unref();
  } finally { closeSync(fd); }
}
const delay = (ms: number) => new Promise<void>(resolveDelay => setTimeout(resolveDelay, ms));
