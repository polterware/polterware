import { Card, Cpu, DataTable, Gauge, HardDrive, MemoryStick, Network, Select, Skeleton, StatusChip, type StatusTone } from "@polterware/frontend-design-system";
import { For, Show, createSignal } from "solid-js";
import { connectionPresentation, type ConnectionMetricsMode, type ConnectionStatus, type DiskIoStats, type DiskPartition, type MetricsSnapshot, type NetworkInterface, type SessionHistory } from "../api";
import { formatBytesPerSec, formatKb, formatPercent, formatUptime } from "../utils/format";
import { ProcessTable } from "./ProcessTable";
import { SparkChart } from "./SparkChart";

const HISTORY = 60;

function loadLevel(percent: number): "ok" | "warn" | "hot" {
  if (percent >= 85) return "hot";
  if (percent >= 60) return "warn";
  return "ok";
}

function pressureLevel(value: number): "ok" | "warn" | "hot" {
  if (value >= 50) return "hot";
  if (value >= 10) return "warn";
  return "ok";
}

function pressureTone(value: number): StatusTone {
  const level = pressureLevel(value);
  if (level === "hot") return "danger";
  if (level === "warn") return "warn";
  return "ok";
}

export function MetricsDashboard(props: {
  snapshot: MetricsSnapshot | null;
  history: SessionHistory;
  status: ConnectionStatus;
  mode?: ConnectionMetricsMode;
  message?: string;
  onKill: (pid: number) => void;
}) {
  const presentation = () => connectionPresentation(props.status);
  const waiting = () => props.snapshot === null && (props.status === "connecting" || props.status === "connected" || props.status === "reconnecting");
  const [pickedIface, setPickedIface] = createSignal<string | undefined>();
  const memPct = () => {
    const memory = props.snapshot?.memory;
    if (!memory || memory.totalKb <= 0) return 0;
    return (memory.usedKb / memory.totalKb) * 100;
  };
  const selectedIface = (network: NetworkInterface[]) => {
    const picked = pickedIface();
    if (picked && network.some((iface) => iface.name === picked)) return picked;
    return network[0]?.name;
  };
  const ifaceSeries = (name: string | undefined) => {
    if (!name) return { rx: [] as number[], tx: [] as number[] };
    return props.history.interfaces[name] ?? { rx: [], tx: [] };
  };

  return (
    <div class="vitals-stack">
      <div class="vitals-card-head">
        <div class="vitals-chips">
          <StatusChip tone={presentation().tone}>{presentation().label}</StatusChip>
          <Show when={props.mode === "fallback"}>
            <StatusChip tone="warn">Script fallback</StatusChip>
          </Show>
        </div>
        <Show when={props.message && props.status !== "connected"}>
          <p class="vitals-sub">{props.message}</p>
        </Show>
      </div>

      <Show
        when={!waiting()}
        fallback={
          <div class="vitals-metrics" aria-busy="true">
            <Card>
              <Skeleton label="Loading metrics" style={{ display: "block", height: "16px", width: "72px" }} />
              <Skeleton style={{ display: "block", height: "32px", width: "96px", "margin-top": "8px" }} />
            </Card>
            <Card>
              <Skeleton style={{ display: "block", height: "16px", width: "88px" }} />
              <Skeleton style={{ display: "block", height: "32px", width: "96px", "margin-top": "8px" }} />
            </Card>
            <Card>
              <Skeleton style={{ display: "block", height: "120px", width: "100%" }} />
            </Card>
          </div>
        }
      >
        <Show
          when={props.snapshot}
          fallback={<p class="vitals-sub">No metrics for this session.</p>}
        >
          {(snapshot) => (
            <>
              <div class="vitals-metrics">
                <Card>
                  <div class="vitals-card-head">
                    <h2 class="vitals-card-title">
                      <Cpu size={14} /> CPU
                    </h2>
                  </div>
                  <div class="vitals-stat">{formatPercent(snapshot().cpuTotalPercent)}</div>
                  <div class="vitals-sub">
                    Load {snapshot().load1.toFixed(2)} / {snapshot().load5.toFixed(2)} / {snapshot().load15.toFixed(2)}
                  </div>
                  <Show when={(snapshot().cpuCores?.length ?? 0) > 0}>
                    <div class="vitals-cores">
                      <For each={snapshot().cpuCores}>
                        {(core) => (
                          <div class="vitals-core">
                            <span>{core.name}</span>
                            <span>{formatPercent(core.usagePercent, 0)}</span>
                            <div class="vitals-bar" aria-hidden="true">
                              <span data-level={loadLevel(core.usagePercent)} style={{ width: `${Math.min(100, Math.max(0, core.usagePercent))}%` }} />
                            </div>
                          </div>
                        )}
                      </For>
                    </div>
                  </Show>
                  <SparkChart series={props.history.cpu.slice(-HISTORY)} label="CPU %" />
                </Card>

                <Card>
                  <h2 class="vitals-card-title">
                    <MemoryStick size={14} /> Memory
                  </h2>
                  <div class="vitals-stat">{formatPercent(memPct())}</div>
                  <div class="vitals-sub">
                    {formatKb(snapshot().memory.usedKb)} / {formatKb(snapshot().memory.totalKb)}
                  </div>
                  <div class="vitals-sub">
                    Swap: {formatKb(snapshot().memory.swapUsedKb)} / {formatKb(snapshot().memory.swapTotalKb)}
                  </div>
                  <div class="vitals-sub">Uptime: {formatUptime(snapshot().uptimeSeconds)}</div>
                </Card>

                <Show when={(snapshot().network?.length ?? 0) > 0}>
                  <Card>
                    <div class="vitals-stack">
                      <h2 class="vitals-card-title">
                        <Network size={14} /> Network
                      </h2>
                      <Show when={snapshot().network.length > 1}>
                        <Select
                          aria-label="Network interface"
                          options={snapshot().network.map((iface) => ({ value: iface.name, label: iface.name }))}
                          value={selectedIface(snapshot().network)}
                          onChange={setPickedIface}
                        />
                      </Show>
                      <div class="vitals-ifaces">
                        <For each={snapshot().network}>
                          {(iface) => (
                            <div class="vitals-iface" data-active={iface.name === selectedIface(snapshot().network) ? "true" : "false"}>
                              <span>{iface.name}</span>
                              <span>Down {formatBytesPerSec(iface.rxBytesPerSec)}</span>
                              <span>Up {formatBytesPerSec(iface.txBytesPerSec)}</span>
                            </div>
                          )}
                        </For>
                      </div>
                      <Show when={selectedIface(snapshot().network)}>
                        {(name) => (
                          <>
                            <div class="vitals-sub">Down {name()}</div>
                            <SparkChart
                              series={ifaceSeries(name()).rx.slice(-HISTORY)}
                              label={`Receive ${name()}`}
                              strokeToken="--color-chart-green"
                            />
                            <div class="vitals-sub">Up {name()}</div>
                            <SparkChart
                              series={ifaceSeries(name()).tx.slice(-HISTORY)}
                              label={`Transmit ${name()}`}
                              strokeToken="--color-chart-blue"
                            />
                          </>
                        )}
                      </Show>
                    </div>
                  </Card>
                </Show>

                <Show when={(snapshot().diskIo?.length ?? 0) > 0}>
                  <Card class="vitals-wide">
                    <h2 class="vitals-card-title">
                      <HardDrive size={14} /> Disk I/O
                    </h2>
                    <DataTable.Root rows={snapshot().diskIo ?? []} getRowId={(row) => row.device} virtualized={false} estimateRowHeight={36}>
                      <DataTable.Table aria-label="Disk I/O">
                        <DataTable.Header>
                          <DataTable.ColumnHeader<DiskIoStats> id="device" sortBy={(row) => row.device}>
                            Device
                          </DataTable.ColumnHeader>
                          <DataTable.ColumnHeader<DiskIoStats> id="read" sortBy={(row) => row.readBytesPerSec}>
                            Read
                          </DataTable.ColumnHeader>
                          <DataTable.ColumnHeader<DiskIoStats> id="write" sortBy={(row) => row.writeBytesPerSec}>
                            Write
                          </DataTable.ColumnHeader>
                          <DataTable.ColumnHeader<DiskIoStats> id="util" sortBy={(row) => row.utilPercent}>
                            Util
                          </DataTable.ColumnHeader>
                        </DataTable.Header>
                        <DataTable.Body<DiskIoStats>>
                          {(row) => (
                            <DataTable.Row row={row}>
                              <DataTable.Cell>{row.device}</DataTable.Cell>
                              <DataTable.Cell>{formatBytesPerSec(row.readBytesPerSec)}</DataTable.Cell>
                              <DataTable.Cell>{formatBytesPerSec(row.writeBytesPerSec)}</DataTable.Cell>
                              <DataTable.Cell>{formatPercent(row.utilPercent, 0)}</DataTable.Cell>
                            </DataTable.Row>
                          )}
                        </DataTable.Body>
                      </DataTable.Table>
                    </DataTable.Root>
                  </Card>
                </Show>

                <Show when={snapshot().pressure}>
                  {(pressure) => (
                    <Card>
                      <h2 class="vitals-card-title">
                        <Gauge size={14} /> Pressure
                      </h2>
                      <div class="vitals-stack">
                        <div class="vitals-pressure">
                          <span>CPU</span>
                          <div class="vitals-bar" aria-hidden="true">
                            <span data-level={pressureLevel(pressure().cpu)} style={{ width: `${Math.min(100, Math.max(0, pressure().cpu))}%` }} />
                          </div>
                          <StatusChip tone={pressureTone(pressure().cpu)}>{formatPercent(pressure().cpu)}</StatusChip>
                        </div>
                        <div class="vitals-pressure">
                          <span>Memory</span>
                          <div class="vitals-bar" aria-hidden="true">
                            <span data-level={pressureLevel(pressure().memory)} style={{ width: `${Math.min(100, Math.max(0, pressure().memory))}%` }} />
                          </div>
                          <StatusChip tone={pressureTone(pressure().memory)}>{formatPercent(pressure().memory)}</StatusChip>
                        </div>
                        <div class="vitals-pressure">
                          <span>I/O</span>
                          <div class="vitals-bar" aria-hidden="true">
                            <span data-level={pressureLevel(pressure().io)} style={{ width: `${Math.min(100, Math.max(0, pressure().io))}%` }} />
                          </div>
                          <StatusChip tone={pressureTone(pressure().io)}>{formatPercent(pressure().io)}</StatusChip>
                        </div>
                      </div>
                    </Card>
                  )}
                </Show>

                <Card class="vitals-wide">
                  <h2 class="vitals-card-title">
                    <HardDrive size={14} /> Disk
                  </h2>
                  <DataTable.Root rows={snapshot().disks} getRowId={(row) => row.mountPoint} virtualized={false} estimateRowHeight={36}>
                    <DataTable.Table aria-label="Disks">
                      <DataTable.Header>
                        <DataTable.ColumnHeader<DiskPartition> id="mount" sortBy={(row) => row.mountPoint}>
                          Mount
                        </DataTable.ColumnHeader>
                        <DataTable.ColumnHeader<DiskPartition> id="used" sortBy={(row) => row.usedKb}>
                          Used
                        </DataTable.ColumnHeader>
                        <DataTable.ColumnHeader<DiskPartition> id="size" sortBy={(row) => row.sizeKb}>
                          Size
                        </DataTable.ColumnHeader>
                        <DataTable.ColumnHeader<DiskPartition> id="use" sortBy={(row) => row.usePercent}>
                          Use %
                        </DataTable.ColumnHeader>
                      </DataTable.Header>
                      <DataTable.Body<DiskPartition>>
                        {(disk) => (
                          <DataTable.Row row={disk}>
                            <DataTable.Cell>{disk.mountPoint}</DataTable.Cell>
                            <DataTable.Cell>{formatKb(disk.usedKb)}</DataTable.Cell>
                            <DataTable.Cell>{formatKb(disk.sizeKb)}</DataTable.Cell>
                            <DataTable.Cell>{disk.usePercent}%</DataTable.Cell>
                          </DataTable.Row>
                        )}
                      </DataTable.Body>
                    </DataTable.Table>
                  </DataTable.Root>
                </Card>

                <Show when={snapshot().thermals.length > 0 || snapshot().gpus.length > 0}>
                  <Card>
                    <h2 class="vitals-card-title">Temperature / GPU</h2>
                    <div class="vitals-stack">
                      <For each={snapshot().thermals}>
                        {(zone) => (
                          <p class="vitals-sub">
                            {zone.id}: {zone.celsius.toFixed(1)} °C
                          </p>
                        )}
                      </For>
                      <For each={snapshot().gpus}>
                        {(gpu) => (
                          <p class="vitals-sub">
                            GPU {gpu.index} {gpu.name}: {gpu.temperatureC} °C, {gpu.utilizationPercent}% util, {gpu.memoryUsedMb}/{gpu.memoryTotalMb} MB
                          </p>
                        )}
                      </For>
                    </div>
                  </Card>
                </Show>
              </div>

              <Card>
                <h2 class="vitals-card-title">Processes</h2>
                <ProcessTable rows={snapshot().processes} onKill={props.onKill} />
              </Card>
            </>
          )}
        </Show>
      </Show>
    </div>
  );
}
