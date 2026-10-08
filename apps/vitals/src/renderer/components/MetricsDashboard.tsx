import { Card, Cpu, DataTable, HardDrive, MemoryStick, Skeleton, StatusChip } from "@polterware/frontend-design-system";
import { For, Show } from "solid-js";
import { connectionPresentation, type ConnectionStatus, type DiskPartition, type MetricsSnapshot, type SessionHistory } from "../api";
import { formatBytesPerSec, formatKb, formatUptime } from "../utils/format";
import { ProcessTable } from "./ProcessTable";
import { SparkChart } from "./SparkChart";

const HISTORY = 60;

export function MetricsDashboard(props: {
  snapshot: MetricsSnapshot | null;
  history: SessionHistory;
  status: ConnectionStatus;
  message?: string;
  onKill: (pid: number) => void;
}) {
  const presentation = () => connectionPresentation(props.status);
  const waiting = () => props.snapshot === null && (props.status === "connecting" || props.status === "connected" || props.status === "reconnecting");
  const net = () => props.snapshot?.network[0];
  const memPct = () => {
    const memory = props.snapshot?.memory;
    if (!memory || memory.totalKb <= 0) return 0;
    return (memory.usedKb / memory.totalKb) * 100;
  };

  return (
    <div class="vitals-stack">
      <div class="vitals-card-head">
        <StatusChip tone={presentation().tone}>{presentation().label}</StatusChip>
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
                  <div class="vitals-stat">{snapshot().cpuTotalPercent.toFixed(1)}%</div>
                  <div class="vitals-sub">
                    Load {snapshot().load1.toFixed(2)} / {snapshot().load5.toFixed(2)} / {snapshot().load15.toFixed(2)}
                  </div>
                  <div class="vitals-cores">
                    <For each={snapshot().cpuCores}>
                      {(core) => (
                        <div class="vitals-core">
                          <span>{core.name}</span>
                          <div class="vitals-bar" aria-hidden="true">
                            <span style={{ width: `${Math.min(100, core.usagePercent)}%` }} />
                          </div>
                          <span>{core.usagePercent.toFixed(0)}%</span>
                        </div>
                      )}
                    </For>
                  </div>
                  <SparkChart series={props.history.cpu.slice(-HISTORY)} label="CPU %" />
                </Card>

                <Card>
                  <h2 class="vitals-card-title">
                    <MemoryStick size={14} /> Memory
                  </h2>
                  <div class="vitals-stat">{memPct().toFixed(1)}%</div>
                  <div class="vitals-sub">
                    {formatKb(snapshot().memory.usedKb)} / {formatKb(snapshot().memory.totalKb)}
                  </div>
                  <div class="vitals-sub">
                    Swap: {formatKb(snapshot().memory.swapUsedKb)} / {formatKb(snapshot().memory.swapTotalKb)}
                  </div>
                  <div class="vitals-sub">Uptime: {formatUptime(snapshot().uptimeSeconds)}</div>
                </Card>

                <Show when={net()}>
                  {(iface) => (
                    <Card>
                      <h2 class="vitals-card-title">Network ({iface().name})</h2>
                      <div class="vitals-sub">Down {formatBytesPerSec(iface().rxBytesPerSec)}</div>
                      <SparkChart series={props.history.rx.slice(-HISTORY)} label="Receive" strokeToken="--color-chart-green" />
                      <div class="vitals-sub">Up {formatBytesPerSec(iface().txBytesPerSec)}</div>
                      <SparkChart series={props.history.tx.slice(-HISTORY)} label="Transmit" strokeToken="--color-chart-blue" />
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
