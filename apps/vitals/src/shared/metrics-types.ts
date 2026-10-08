export interface CpuCoreStats {
  name: string
  usagePercent: number
}

export interface MemoryStats {
  totalKb: number
  availableKb: number
  usedKb: number
  swapTotalKb: number
  swapFreeKb: number
  swapUsedKb: number
}

export interface DiskPartition {
  filesystem: string
  sizeKb: number
  usedKb: number
  availKb: number
  usePercent: number
  mountPoint: string
}

export interface NetworkInterface {
  name: string
  rxBytesPerSec: number
  txBytesPerSec: number
}

export interface ProcessRow {
  pid: number
  user: string
  cpuPercent: number
  memPercent: number
  command: string
}

export interface ThermalZone {
  id: string
  celsius: number
}

export interface GpuStats {
  index: number
  name: string
  temperatureC: number
  utilizationPercent: number
  memoryUsedMb: number
  memoryTotalMb: number
}

export interface MetricsSnapshot {
  cpuTotalPercent: number
  cpuCores: CpuCoreStats[]
  load1: number
  load5: number
  load15: number
  uptimeSeconds: number
  memory: MemoryStats
  disks: DiskPartition[]
  network: NetworkInterface[]
  processes: ProcessRow[]
  thermals: ThermalZone[]
  gpus: GpuStats[]
}
