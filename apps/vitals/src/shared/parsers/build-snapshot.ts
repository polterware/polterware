import type { MetricsSnapshot } from '../metrics-types'
import { parseDf } from './df'
import { parseLoadavg } from './loadavg'
import { parseMeminfo } from './meminfo'
import { cpuUsagePercent, parseProcStat, type ProcStatSample } from './proc-stat'
import { netRates, parseNetDev, type NetDevSample } from './net-dev'
import { parseNvidiaSmi } from './nvidia-smi'
import { parsePs } from './ps'
import { splitSections } from './sections'
import { parseThermal } from './thermal'
import { parseUptime } from './uptime'

export interface DeltaState {
  procStat: ProcStatSample[]
  netDev: NetDevSample[]
  collectedAt: number
}

export function buildSnapshot(
  raw: string,
  prev: DeltaState | null,
  collectedAt: number
): { snapshot: MetricsSnapshot; state: DeltaState } {
  const sections = splitSections(raw)
  const procSamples = parseProcStat(sections.stat ?? '')
  const netSamples = parseNetDev(sections.netdev ?? '')

  const intervalSec =
    prev && prev.collectedAt > 0
      ? Math.max(0.001, (collectedAt - prev.collectedAt) / 1000)
      : 1

  const prevMap = new Map((prev?.procStat ?? []).map((s) => [s.name, s]))
  const cpuCores = procSamples
    .filter((s) => s.name.startsWith('cpu') && s.name !== 'cpu')
    .map((s) => ({
      name: s.name,
      usagePercent: prevMap.has(s.name) ? cpuUsagePercent(prevMap.get(s.name)!, s) : 0
    }))

  const total = procSamples.find((s) => s.name === 'cpu')
  const prevTotal = prevMap.get('cpu')
  const cpuTotalPercent =
    total && prevTotal ? cpuUsagePercent(prevTotal, total) : 0

  const load = parseLoadavg(sections.loadavg ?? '')
  const memory = parseMeminfo(sections.meminfo ?? '')
  const disks = parseDf(sections.df ?? '')
  const network =
    prev?.netDev.length
      ? netRates(prev.netDev, netSamples, intervalSec)
      : netSamples.map((n) => ({ name: n.name, rxBytesPerSec: 0, txBytesPerSec: 0 }))
  const processes = parsePs(sections.ps ?? '')
  const thermals = parseThermal(sections.thermal ?? '')
  const gpus = parseNvidiaSmi(sections.nvidia ?? '')

  const snapshot: MetricsSnapshot = {
    cpuTotalPercent,
    cpuCores,
    load1: load.load1,
    load5: load.load5,
    load15: load.load15,
    uptimeSeconds: parseUptime(sections.uptime ?? ''),
    memory,
    disks,
    network,
    processes,
    thermals,
    gpus
  }

  return {
    snapshot,
    state: { procStat: procSamples, netDev: netSamples, collectedAt }
  }
}
