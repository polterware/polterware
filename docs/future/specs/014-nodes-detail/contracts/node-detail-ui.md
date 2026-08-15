# Contract: Resources Node Detail UI

**Owner**: `packages/features/resources`  
**Extends**: 004 Resources Nodes list; aligns with `docs/spec.md` §28 Nodes, §10.3 polter-node, §55 Node security  
**Host**: Global `DetailPanel` via `DetailPanel.Content` (MainSurface)

## View model

```ts
type NodeHealthStatus =
  | "registering"
  | "online"
  | "degraded"
  | "offline"
  | "draining"
  | "upgrading"
  | "revoked"
  | "unknown";

type NodeCapability = {
  id?: string;
  name: string;
  authorized?: boolean;
};

type NodeAssignedResource = {
  name: string;
  type: string;
  status: string;
};

type NodeRecentOperation = {
  at: string;
  command: string;
  actor: string;
  result: string; // Success | Failed | Denied | Cancelled | Pending | …
};

type NodeConnection = {
  state?: "connected" | "reconnecting" | "disconnected" | "unknown";
  lastConnectedAt?: string;
  /** Honest product copy; outbound is the protocol default. */
  modelNote?: string;
};

type TargetNode = {
  id: string;
  name: string;
  location: string;
  os: string;
  version: string;
  status: NodeHealthStatus;
  lastHeartbeatAt: string;
  /** Stable registry id when different presentation is needed; defaults to `id`. */
  nodeId?: string;
  hostname?: string;
  ip?: string;
  arch?: string;
  cpu?: string;
  memory?: string;
  storage?: string;
  /** Operator-facing health note (e.g. heartbeat loss ≠ process death). */
  healthNote?: string;
  connection?: NodeConnection;
  capabilities?: string[] | NodeCapability[];
  assignedResources?: NodeAssignedResource[];
  recentOperations?: NodeRecentOperation[];
  /** Optional upgrade / agent channel label. */
  upgradeAvailable?: string | null;
  dataSource: "core" | "fixture";
};
```

## Composition API

```ts
type NodeDetailProps = {
  node: TargetNode | null | undefined;
  loading?: boolean;
  error?: string | null;
  notFound?: boolean;
  onTerminal?: () => void;
  onRunCommand?: () => void;
};
```

## Layout (mandatory)

1. `DetailPanel.Content` → `Header` (HardDrive icon + display name + actions) → `Body layout="sections"` → `Grid`.
2. Metrics row: Status · Last heartbeat · Version · Capabilities count.
3. Sections:
   - **Identity** — node id (mono), hostname, location, OS, architecture, IP, source honesty
   - **Machine** — CPU, memory, storage (when any present)
   - **Connection** — connection state chip, last connected, model note (when connection present)
   - **Health** — status chip + optional healthNote (especially offline/degraded)
   - **Capabilities** — `DataTable` (Capability · Authorized when modeled); no Actions column
   - **Assigned resources** — `DataTable` (Resource · Type · Status chip); no Actions column
   - **Recent operations** — `DataTable` (Command · Result chip · Actor · When); no Actions column
   - **Agent** — version, optional upgrade available (when present)
4. Header actions:
   - **Open terminal** when node may accept interactive session (`isNodeTerminalAvailable`)
   - **Run command** (primary) when node may accept dispatch (`isNodeDispatchAvailable`)
   - Both must be hidden or disabled for offline / revoked (and preferably draining without drain-exempt path)
5. Status everywhere uses `StatusChip` filled tones via `nodeStatusTone` / `nodeStatusLabel`.
6. List PrimaryCell icon for nodes remains HardDrive / infrastructure; header reuses the same family.

## Status mapping (normative UI)

| Status | Label | Tone |
| --- | --- | --- |
| online | Online | healthy |
| degraded | Degraded | warn |
| offline | Offline | down |
| registering | Registering | warn |
| draining | Draining | warn |
| upgrading | Upgrading | warn |
| revoked | Revoked | danger |
| unknown | Unknown | unknown |

## Honesty

- `dataSource: "fixture"` never presented as live polter-node stream.
- Terminal / Run command feedback must say fixture/session until core exists when mock path is used.
- Heartbeat loss / offline must not be worded as “all remote processes dead”.
- No secrets, private keys, or full certificates in fixtures or UI.
- Revoked nodes never accept new dispatch in UI affordances.

## Non-goals

- Live outbound node protocol / real heartbeats
- Remote desktop or privileged shell outside existing Terminal mock
- Full process manager (Processes tab remains separate)
- Certificate/credential material UI
- Separate full-page node editor
- Actions column on nested tables
