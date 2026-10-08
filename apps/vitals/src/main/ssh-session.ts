import { Client, type ClientChannel, type ConnectConfig } from "ssh2";
import { readFileSync } from "node:fs";
import { createHash } from "node:crypto";
import { buildSnapshot, type DeltaState } from "../shared/parsers/build-snapshot.js";
import type { MetricsSnapshot } from "../shared/metrics-types.js";
import { verifyFingerprint, setKnownFingerprint } from "./known-hosts.js";
import { buildRemoteMetricsScript } from "./remote-script.js";

export interface SessionAuth {
  password?: string;
  privateKeyPath?: string;
  privateKeyPassphrase?: string;
}

export interface SessionConfig {
  sessionId: string;
  host: string;
  port: number;
  username: string;
  auth: SessionAuth;
  onMetrics: (snapshot: MetricsSnapshot, collectedAt: number) => void;
  onStatus: (status: string, message?: string) => void;
  onHostKeyPrompt: (fingerprint: string) => void;
}

function fingerprintSha256(key: Buffer): string {
  return `SHA256:${createHash("sha256").update(key).digest("base64")}`;
}

export class SshSession {
  private client = new Client();
  private shell: ClientChannel | null = null;
  private interval: ReturnType<typeof setInterval> | null = null;
  private delta: DeltaState | null = null;
  private disposed = false;
  private remoteScript = buildRemoteMetricsScript();
  private hostKeyApproved = false;

  constructor(private config: SessionConfig) {}

  approveHostKey(fingerprint: string): void {
    setKnownFingerprint(this.config.host, this.config.port, fingerprint);
    this.hostKeyApproved = true;
  }

  connect(): void {
    this.disposed = false;
    const connectConfig: ConnectConfig = {
      host: this.config.host,
      port: this.config.port,
      username: this.config.username,
      readyTimeout: 20000,
      keepaliveInterval: 10000,
      keepaliveCountMax: 3,
      hostVerifier: (key: Buffer) => {
        const fp = fingerprintSha256(key);
        const verdict = verifyFingerprint(this.config.host, this.config.port, fp);
        if (verdict === "mismatch") {
          this.config.onStatus("error", "Host key changed — connection refused");
          return false;
        }
        if (verdict === "ok") return true;
        if (this.hostKeyApproved) return true;
        this.config.onHostKeyPrompt(fp);
        return false;
      },
    };

    if (this.config.auth.password) {
      connectConfig.password = this.config.auth.password;
    }
    if (this.config.auth.privateKeyPath) {
      connectConfig.privateKey = readFileSync(this.config.auth.privateKeyPath, "utf8");
      if (this.config.auth.privateKeyPassphrase) {
        connectConfig.passphrase = this.config.auth.privateKeyPassphrase;
      }
    }

    // Handlers are bound to this client instance: events from a client that
    // resetClient() already replaced must not drive the new connection.
    const client = this.client;
    const isCurrent = () => client === this.client;

    client
      .on("ready", () => {
        if (!isCurrent()) return;
        this.config.onStatus("connected");
        this.startPolling();
      })
      .on("error", (err) => {
        if (!isCurrent()) return;
        if (err.message.includes("Host verification failed")) {
          this.config.onStatus("connecting", "Waiting for host key approval");
          return;
        }
        this.config.onStatus("error", err.message);
      })
      .on("close", () => {
        if (!isCurrent()) return;
        this.stopPolling();
        if (!this.disposed) {
          this.config.onStatus("disconnected");
        }
      })
      .connect(connectConfig);
  }

  private startPolling(): void {
    this.stopPolling();
    this.interval = setInterval(() => {
      void this.collectOnce();
    }, 1000);
    void this.collectOnce();
  }

  private stopPolling(): void {
    if (this.interval) {
      clearInterval(this.interval);
      this.interval = null;
    }
  }

  private execScript(): Promise<string> {
    return new Promise((resolve, reject) => {
      this.client.exec(this.remoteScript, (err, stream) => {
        if (err) {
          reject(err);
          return;
        }
        let stdout = "";
        stream
          .on("close", (code: number) => {
            if (code !== 0 && !stdout.trim()) {
              reject(new Error(`Remote script exited with code ${code}`));
              return;
            }
            resolve(stdout);
          })
          .on("data", (d: Buffer) => {
            stdout += d.toString("utf8");
          })
          .stderr.on("data", () => {
            /* ignore */
          });
      });
    });
  }

  private async collectOnce(): Promise<void> {
    try {
      const raw = await this.execScript();
      const collectedAt = Date.now();
      const { snapshot, state } = buildSnapshot(raw, this.delta, collectedAt);
      this.delta = state;
      this.config.onMetrics(snapshot, collectedAt);
    } catch (e) {
      const msg = e instanceof Error ? e.message : "Metrics collection failed";
      this.config.onStatus("error", msg);
    }
  }

  killProcess(pid: number, signal: "TERM" | "KILL"): Promise<void> {
    const cmd = signal === "KILL" ? `kill -9 ${pid}` : `kill ${pid}`;
    return new Promise((resolve, reject) => {
      this.client.exec(cmd, (err, stream) => {
        if (err) {
          reject(err);
          return;
        }
        stream.on("close", (code: number) => {
          if (code === 0) resolve();
          else reject(new Error(`kill exited with ${code}`));
        });
      });
    });
  }

  openShell(
    cols: number,
    rows: number,
    onData: (data: string) => void,
    onClose: () => void,
  ): Promise<void> {
    this.closeShell();
    return new Promise((resolve, reject) => {
      this.client.shell({ term: "xterm-256color", cols, rows }, (err, stream) => {
        if (err) {
          reject(err);
          return;
        }
        this.shell = stream;
        stream.on("data", (d: Buffer) => onData(d.toString("utf8")));
        stream.stderr.on("data", (d: Buffer) => onData(d.toString("utf8")));
        stream.on("close", () => {
          if (this.shell === stream) this.shell = null;
          onClose();
        });
        resolve();
      });
    });
  }

  writeShell(data: string): void {
    this.shell?.write(data);
  }

  resizeShell(cols: number, rows: number): void {
    this.shell?.setWindow(rows, cols, 0, 0);
  }

  closeShell(): void {
    const s = this.shell;
    this.shell = null;
    s?.close();
  }

  dispose(): void {
    this.disposed = true;
    this.stopPolling();
    this.closeShell();
    this.client.end();
  }

  resetClient(): void {
    this.stopPolling();
    this.shell = null;
    const old = this.client;
    this.client = new Client();
    old.removeAllListeners();
    old.on("error", () => {});
    old.end();
    this.delta = null;
  }
}
