# Architecture

Modeled on the Polterware local core (`backend/runtimes/cli/serve.go`, `backend/platform/runstate`,
`apps/desktop/src/main/core/*`). Read those files; copy the patterns, not the imports (D2).

## Processes

```text
                       ┌────────────────────────────────────────────┐
  Claude Code, Cursor  │  sparky (Go) — the CORE, one per database  │
  ── stdio ──▶ `sparky mcp` bridge ──▶  Connect API  (127.0.0.1, bearer)      │
  ── HTTP  ──▶ /mcp (streamable HTTP, loopback, bearer) │            │
                       │  SQLite (WAL)  ·  attachments dir  ·  FTS5 │
  Electron Main ──Connect-ES──▶  domain services ──▶ providers      │
   ▲ window.sparky (preload)   │  change stream (WatchChanges)      │
   │                           └───────────────▲────────────────────┘
  Renderer (Solid)                             │ Plan / Ack / Complete
       Electron Main + native notify module (Swift/ObjC++)
       └─▶ UNUserNotificationCenter under Sparky.app identity (the OS delivers)
```

1. **Core** (`dist/sparky serve`): the only process that opens SQLite. Detached daemon, lock-guarded, build handshake
   (replaced if a newer bundled build exists). Exposes: Connect RPC for the UI/CLI/notifier, MCP over HTTP and via stdio bridge.
2. **Electron app** (`apps/sparky-desktop`): thin Main (supervisor, Connect-ES clients, IPC, deep links, updater, menu), typed
   preload bridge `window.sparky`, Solid renderer. Closing the window does not stop the core.
3. **Native notify module** (`apps/sparky-desktop/native/notify`, loaded by Electron Main): schedules OS notifications under the
   Sparky.app identity from the core's plan, handles notification actions, relaunches hidden to refresh. No separate helper app (O1).

## Paths and files

| What | Where |
|------|-------|
| Data dir | `~/Library/Application Support/Sparky/` (`app.getPath("appData")/Sparky`) |
| Database | `<data>/sparky.db` (+ `-wal`, `-shm`) |
| Attachments | `<data>/attachments/<memory_id>/<attachment_id>.<ext>` |
| Run state | `<data>/run/core.json` (pid, endpoint, token, build, dbPath, startedAt), `core.lock`, `core.log` — mode 0600 |
| MCP token | `<data>/run/mcp-token` (stable, 0600; shown in Settings > Agents) |
| Settings | `settings` table (no plist, no JSON files besides run state) |
| Packaged core | `Sparky.app/Contents/Resources/sparky` |

## Network and auth

- Core listens on `127.0.0.1` only. Connect API: ephemeral port (`:0`) written to `core.json`; bearer = per-start session token
  (constant-time compare), same as Polterware.
- MCP HTTP: **preferred fixed port `47600`** (configurable in Settings; falls back to an ephemeral port and tells the UI) so
  client configs stay stable. Bearer = the stable `mcp-token`. Reject requests whose `Host` is not
  `127.0.0.1:<port>` / `localhost:<port>`; reject any request carrying an `Origin` header that is not loopback (blocks browser
  CSRF and DNS rebinding). No CORS headers at all.
- MCP stdio: `sparky mcp` reads `core.json`, attaches to (or starts) the core, and proxies. Zero config, no token in the client
  config. This is the **recommended** setup in the UI.
- The renderer never sees the endpoint or token. Only Electron Main holds Connect clients.

## Data flows

**UI write.** Renderer → `window.sparky.memories.update(input)` → Main → Connect → domain service (checks `rev`, validates,
writes, appends `activity`, updates FTS) → commit hook emits a change → `WatchChanges` stream → Main →
IPC event → renderer store invalidates and refetches affected entities.

**Agent write (MCP).** MCP client → tool → the **same domain service** (actor `mcp:<clientName>`) → commit → change stream →
open window refreshes. Writes are synchronous: the tool result already contains the new entity (no queue, no polling).

**Reminders.** Any schedule/memory/status change → change stream (or a hidden relaunch of Sparky if it is closed) → Main calls `Plan` →
the native notify module diffs against pending notifications → arms/cancels. Notification actions → `Complete`/`Snooze` RPC or `sparky://` deep link.
See [03-recurrence-and-reminders.md](03-recurrence-and-reminders.md).

**Conflict model.** Every mutable entity has `rev` (integer, +1 per write). Updates may send `expected_rev`; mismatch returns
`FAILED_PRECONDITION` with the current entity attached. The UI sends `expected_rev` for edits to an open entity; MCP tools
accept it optionally (default: last-write-wins for agents unless the entity's `rev` changed since the agent's last read in
the same session — see [04-mcp.md](04-mcp.md)).

## Lifecycle

- Electron Main `ensureCore()` (port of `apps/desktop/src/main/core/process-supervisor.ts`): read `core.json`, health-check,
  compare build id with the bundled binary, replace if older, spawn detached if missing. `POLTERWARE_*` env overrides become
  `SPARKY_CORE_ENDPOINT` / `SPARKY_CORE_TOKEN` for dev.
- Core never idle-exits by default (reminders must keep working). `--idle` stays available for tests.
- Reminders armed earlier keep firing after a reboot (the OS holds them); they are refreshed when Sparky next runs (see 03).
- Quit behavior (setting): "Closing the window keeps Sparky running in the background" (default on). A menu-bar item is **not**
  required in v1; the OS owns delivery of already-armed reminders.

## Deep links (`sparky://`)

`sparky://memory/<ulid>`, `sparky://memory/<ulid>?action=focus`, `sparky://today`, `sparky://calendar?date=YYYY-MM-DD`,
`sparky://search?q=…`. Main validates scheme, route whitelist and ULID shape; everything else is dropped. Single-instance lock
forwards links to the running window.

## Security checklist (W3-SECURITY verifies)

Electron: `contextIsolation: true`, `sandbox: true`, `nodeIntegration: false`, strict CSP (no remote scripts; `img-src`/
`connect-src` allow only the chosen map tile host, and only when online maps are enabled), navigation and `window.open` guards,
IPC channel allowlist with schema validation, no `remote`. Copy the Polterware tests (`window-security.test.ts`,
`desktop-navigation-policy.test.ts`, `ipc.test.ts`).
Core: bearer on every route, Host/Origin checks, file paths never accepted from MCP unless the "agents may attach local files"
setting is on (default off) and then only regular files ≤ 50 MB; attachment storage uses generated ids, never client names for
paths; SQL always parameterized; `core.json`/`mcp-token` 0600; delete tools require `confirm: true`.

## Observability

`<data>/run/core.log` (structured, rotated at 5 MB × 3). `sparky doctor` prints: core status/build, db path and size, WAL size,
MCP port/token presence, notifier status, pending reminders (armed vs planned), last 20 `activity` rows.

## Performance budgets (checked in W3-E2E)

Core idle RSS < 60 MB; cold start to listening < 300 ms; `list_memories` p95 < 30 ms at 10k memories; FTS query p95 < 30 ms;
renderer first meaningful paint < 1 s on an M-series Mac; memory list virtualized (`@tanstack/solid-virtual`) and smooth at 10k rows.
