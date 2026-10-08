import { createHash } from "node:crypto";
import { readFileSync, statSync } from "node:fs";
import { join } from "node:path";
import type { Client, SFTPWrapper } from "ssh2";

export type AgentArch = "amd64" | "arm64";

export class UnsupportedArchError extends Error {
  readonly code = "unsupported_arch" as const;

  constructor(readonly uname: string) {
    super(`Unsupported remote architecture: ${uname}`);
    this.name = "UnsupportedArchError";
  }
}

/** Map `uname -m` to the bundled agent arch. Only x86_64 and aarch64 are supported. */
export function mapUnameArch(uname: string): AgentArch {
  switch (uname.trim()) {
    case "x86_64":
      return "amd64";
    case "aarch64":
      return "arm64";
    default:
      throw new UnsupportedArchError(uname.trim());
  }
}

export function agentBinaryFileName(arch: AgentArch): string {
  return `vitals-agent-linux-${arch}`;
}

export function resolveLocalAgentBinary(
  arch: AgentArch,
  options: { appPath: string; resourcesPath: string; packaged: boolean },
): string {
  const name = agentBinaryFileName(arch);
  if (options.packaged) {
    return join(options.resourcesPath, "agent", name);
  }
  return join(options.appPath, "resources", "agent", name);
}

export function sha256File(path: string): string {
  return createHash("sha256").update(readFileSync(path)).digest("hex");
}

export function remoteAgentCachePath(home: string, sha256Hex: string): string {
  const short = sha256Hex.slice(0, 12);
  return `${home.replace(/\/$/, "")}/.cache/vitals/agent-${short}`;
}

function execCapture(client: Client, command: string): Promise<{ stdout: string; code: number }> {
  return new Promise((resolve, reject) => {
    client.exec(command, (err, stream) => {
      if (err) {
        reject(err);
        return;
      }
      let stdout = "";
      stream
        .on("close", (code: number) => {
          resolve({ stdout, code: code ?? 0 });
        })
        .on("data", (d: Buffer) => {
          stdout += d.toString("utf8");
        });
      stream.stderr.on("data", () => {
        /* ignore */
      });
      stream.on("error", reject);
    });
  });
}

function openSftp(client: Client): Promise<SFTPWrapper> {
  return new Promise((resolve, reject) => {
    client.sftp((err, sftp) => {
      if (err) reject(err);
      else resolve(sftp);
    });
  });
}

function sftpStat(sftp: SFTPWrapper, path: string): Promise<{ size: number } | null> {
  return new Promise((resolve) => {
    sftp.stat(path, (err, stats) => {
      if (err || !stats) {
        resolve(null);
        return;
      }
      resolve({ size: stats.size });
    });
  });
}

function sftpFastPut(sftp: SFTPWrapper, localPath: string, remotePath: string): Promise<void> {
  return new Promise((resolve, reject) => {
    sftp.fastPut(localPath, remotePath, (err) => {
      if (err) reject(err);
      else resolve();
    });
  });
}

function sftpChmod(sftp: SFTPWrapper, path: string, mode: number): Promise<void> {
  return new Promise((resolve, reject) => {
    sftp.chmod(path, mode, (err) => {
      if (err) reject(err);
      else resolve();
    });
  });
}

export interface InstallAgentResult {
  remotePath: string;
  arch: AgentArch;
  sha256: string;
  uploaded: boolean;
}

export interface InstallAgentOptions {
  appPath: string;
  resourcesPath: string;
  packaged: boolean;
}

/**
 * Detect remote arch, ensure the matching bundled agent is present under
 * ~/.cache/vitals/agent-<sha12> with mode 0700. Skips upload when size matches.
 */
export async function installAgent(
  client: Client,
  options: InstallAgentOptions,
): Promise<InstallAgentResult> {
  const unameResult = await execCapture(client, "uname -m");
  if (unameResult.code !== 0) {
    throw new Error(`uname -m failed with code ${unameResult.code}`);
  }
  const arch = mapUnameArch(unameResult.stdout);

  const localPath = resolveLocalAgentBinary(arch, options);
  const localStat = statSync(localPath);
  const sha256 = sha256File(localPath);
  const size = localStat.size;

  const homeResult = await execCapture(client, 'printenv HOME || echo "$HOME"');
  const home = homeResult.stdout.trim().split("\n")[0]?.trim();
  if (!home) {
    throw new Error("Could not resolve remote HOME");
  }

  const remoteDir = `${home}/.cache/vitals`;
  const remotePath = remoteAgentCachePath(home, sha256);

  const sftp = await openSftp(client);
  try {
    await execCapture(client, `mkdir -p ${shellQuote(remoteDir)}`);

    const existing = await sftpStat(sftp, remotePath);
    if (existing && existing.size === size) {
      await sftpChmod(sftp, remotePath, 0o700);
      return { remotePath, arch, sha256, uploaded: false };
    }

    await sftpFastPut(sftp, localPath, remotePath);
    await sftpChmod(sftp, remotePath, 0o700);
    return { remotePath, arch, sha256, uploaded: true };
  } finally {
    sftp.end();
  }
}

function shellQuote(path: string): string {
  return `'${path.replace(/'/g, `'\\''`)}'`;
}
