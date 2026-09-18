import { spawn, type ChildProcess } from "node:child_process";
import { createHash, randomBytes } from "node:crypto";
import { existsSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { app } from "electron";

const LISTEN_PATTERN = /listening on (http:\/\/127\.0\.0\.1:\d+)/;

export type CoreProcessHandle = {
  endpoint: string;
  token: string;
  child?: ChildProcess;
};

export function mintSessionToken(): string {
  return randomBytes(32).toString("hex");
}

export function resolveUserDatabasePath(): string {
  return join(app.getPath("userData"), "local.db");
}

export function resolveCoreBinaryPath(): string | undefined {
  if (app.isPackaged) {
    const packaged = join(process.resourcesPath, "polterware-core");
    return existsSync(packaged) ? packaged : undefined;
  }
  const fromApp = resolve(app.getAppPath(), "../../dist/polterware-core");
  if (existsSync(fromApp)) return fromApp;
  const fromMain = resolve(dirname(fileURLToPath(import.meta.url)), "../../../../dist/polterware-core");
  if (existsSync(fromMain)) return fromMain;
  return undefined;
}

export function resolveRepositoryRoot(): string {
  if (app.isPackaged) return app.getAppPath();
  return resolve(app.getAppPath(), "../..");
}

export async function startCoreProcess(options: {
  dbPath: string;
  token: string;
  log?: (message: string) => void;
}): Promise<CoreProcessHandle> {
  const overrideEndpoint = process.env.POLTERWARE_CORE_ENDPOINT?.trim();
  const overrideToken = process.env.POLTERWARE_CORE_TOKEN?.trim();
  if (overrideEndpoint) {
    return { endpoint: overrideEndpoint.replace(/\/$/, ""), token: overrideToken || options.token };
  }

  const binary = resolveCoreBinaryPath();
  const env = {
    ...process.env,
    POLTERWARE_SESSION_TOKEN: options.token,
  };
  const args = ["-listen", "127.0.0.1:0", "-db", options.dbPath];
  const child = binary
    ? spawn(binary, args, { env, stdio: ["ignore", "ignore", "pipe"] })
    : spawn("go", ["run", "./backend/runtimes/core", ...args], {
        cwd: resolveRepositoryRoot(),
        env,
        stdio: ["ignore", "ignore", "pipe"],
      });

  const endpoint = await waitForEndpoint(child, options.log, binary ? 20_000 : 60_000);
  return { endpoint, token: options.token, child };
}

export async function stopCoreProcess(handle: CoreProcessHandle | undefined): Promise<void> {
  const child = handle?.child;
  if (!child || child.killed || child.exitCode !== null) return;
  await new Promise<void>((resolveStop) => {
    const timer = setTimeout(() => {
      if (!child.killed) child.kill("SIGKILL");
      resolveStop();
    }, 2000);
    child.once("exit", () => {
      clearTimeout(timer);
      resolveStop();
    });
    child.kill("SIGTERM");
  });
}

function waitForEndpoint(child: ChildProcess, log?: (message: string) => void, timeoutMs = 20_000): Promise<string> {
  return new Promise((resolveEndpoint, reject) => {
    let buffer = "";
    const timer = setTimeout(() => {
      cleanup();
      reject(new Error("Timed out waiting for Polterware core to listen."));
    }, timeoutMs);
    const onData = (chunk: Buffer) => {
      buffer += chunk.toString("utf8");
      log?.(chunk.toString("utf8").trim());
      const match = LISTEN_PATTERN.exec(buffer);
      if (!match?.[1]) return;
      cleanup();
      resolveEndpoint(match[1]);
    };
    const onExit = (code: number | null) => {
      cleanup();
      reject(new Error(`Polterware core exited before listening (code ${code ?? "unknown"}).`));
    };
    const cleanup = () => {
      clearTimeout(timer);
      child.stderr?.off("data", onData);
      child.off("exit", onExit);
    };
    child.stderr?.on("data", onData);
    child.once("exit", onExit);
    child.once("error", (error) => {
      cleanup();
      reject(error);
    });
  });
}

export function fingerprintToken(token: string): string {
  return createHash("sha256").update(token).digest("hex").slice(0, 12);
}
