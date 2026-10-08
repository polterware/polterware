import type { StatusTone } from "@polterware/frontend-design-system";

export type AuthMode = "password" | "privateKey";

export interface SavedServerPublic {
  id: string;
  label: string;
  host: string;
  port: number;
  username: string;
  authMode: AuthMode;
  privateKeyPath?: string;
}

export interface ConnectPayload {
  serverId?: string;
  label?: string;
  host: string;
  port: number;
  username: string;
  authMode: AuthMode;
  password?: string;
  privateKeyPath?: string;
  privateKeyPassphrase?: string;
  save?: boolean;
}

export type ConnectionStatus =
  | "idle"
  | "connecting"
  | "connected"
  | "reconnecting"
  | "disconnected"
  | "error";

export interface ConnectionStateEvent {
  sessionId: string;
  status: ConnectionStatus;
  message?: string;
  label: string;
  host: string;
}

export interface HostKeyPrompt {
  sessionId: string;
  host: string;
  port: number;
  fingerprint: string;
}

export interface CpuCoreStats {
  name: string;
  usagePercent: number;
}

export interface MemoryStats {
  totalKb: number;
  availableKb: number;
  usedKb: number;
  swapTotalKb: number;
  swapFreeKb: number;
  swapUsedKb: number;
}

export interface DiskPartition {
  filesystem: string;
  sizeKb: number;
  usedKb: number;
  availKb: number;
  usePercent: number;
  mountPoint: string;
}

export interface NetworkInterface {
  name: string;
  rxBytesPerSec: number;
  txBytesPerSec: number;
}

export interface ProcessRow {
  pid: number;
  user: string;
  cpuPercent: number;
  memPercent: number;
  command: string;
}

export interface ThermalZone {
  id: string;
  celsius: number;
}

export interface GpuStats {
  index: number;
  name: string;
  temperatureC: number;
  utilizationPercent: number;
  memoryUsedMb: number;
  memoryTotalMb: number;
}

export interface MetricsSnapshot {
  cpuTotalPercent: number;
  cpuCores: CpuCoreStats[];
  load1: number;
  load5: number;
  load15: number;
  uptimeSeconds: number;
  memory: MemoryStats;
  disks: DiskPartition[];
  network: NetworkInterface[];
  processes: ProcessRow[];
  thermals: ThermalZone[];
  gpus: GpuStats[];
}

export interface MetricsEvent {
  sessionId: string;
  snapshot: MetricsSnapshot;
  collectedAt: number;
}

export interface KillProcessPayload {
  sessionId: string;
  pid: number;
  signal?: "TERM" | "KILL";
}

export interface VitalsApi {
  connect(payload: ConnectPayload): Promise<{ sessionId: string }>;
  disconnect(sessionId: string): Promise<void>;
  listServers(): Promise<SavedServerPublic[]>;
  deleteServer(id: string): Promise<void>;
  acceptHostKey(sessionId: string): Promise<void>;
  rejectHostKey(sessionId: string): Promise<void>;
  killProcess(payload: KillProcessPayload): Promise<void>;
  getTheme(): Promise<"dark" | "light">;
  setTheme(theme: "dark" | "light"): Promise<void>;
  onConnectionState(cb: (ev: ConnectionStateEvent) => void): () => void;
  onMetrics(cb: (ev: MetricsEvent) => void): () => void;
  onHostKeyPrompt(cb: (ev: HostKeyPrompt) => void): () => void;
}

export interface SessionHistory {
  rx: number[];
  tx: number[];
  cpu: number[];
}

export function connectionPresentation(status: ConnectionStatus): { label: string; tone: StatusTone } {
  switch (status) {
    case "connected":
      return { label: "Live", tone: "ok" };
    case "reconnecting":
      return { label: "Reconnecting", tone: "syncing" };
    case "connecting":
      return { label: "Connecting", tone: "syncing" };
    case "error":
    case "disconnected":
      return { label: "Failed", tone: "danger" };
    default:
      return { label: "Idle", tone: "muted" };
  }
}
