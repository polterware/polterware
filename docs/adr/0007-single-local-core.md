# ADR 0007: Single local core process

- Status: Proposed
- Date: 2026-09-28
- Relates to: [ADR 0006](0006-polter-format-and-work-model.md), [`atlas-grounded-sdd-roadmap.md`](../architecture/atlas-grounded-sdd-roadmap.md) (slice 3d), [`cli.md`](../architecture/cli.md), [`local-wedge-runtime.md`](../architecture/local-wedge-runtime.md)

## Context

Today every entry point opens the SQLite store in its own process: each CLI command, each `polterware mcp` stdio process, the TUI, and the core (`polterware serve`) that the Desktop talks to. They share the file but not knowledge of each other's writes. A card written by `polterware apply` or MCP `polter.apply` stayed invisible in the Desktop until a manual reload, because the core never saw the write and the Desktop has no channel to be told.

The product thesis is that project state is identical for every agent and person. Several processes writing one database, with no notification path, cannot deliver that. Cloud later has the same shape as a local server with many clients.

## Decision

1. **The core is the only process that opens the SQLite store.** `polterware serve` owns the database. The Desktop, CLI commands, the status TUI, and MCP reach it over loopback Connect with the bearer token from `<db dir>/run/core.json` (mode 0600). Tests may open the store directly.
2. **One core per database**, enforced by an exclusive lock on `<db dir>/run/core.lock`.
3. **Daemon with idle exit.** Any client starts the core when none is running and attaches otherwise. The core outlives the Desktop and exits by itself after a period with no requests, open event streams, or MCP sessions. There is no "owner" distinction between Desktop-started and CLI-started cores.
4. **Version handshake.** `HealthService.Check` reports the core build. A client with a different build asks the core to shut down (`MaintenanceService.Shutdown`) and starts its own.
5. **Change events come from the database, not from call sites.** SQLite commit hooks record which tables a committed transaction touched; an in-memory bus coalesces them into change events (sequence number, project ids, kinds) served by `EventService.Watch`. A new write path cannot forget to emit. The Desktop Main process forwards events to the renderer through `window.polterware`; the renderer refreshes only the affected slices.
6. **MCP logic runs inside the core.** The core serves MCP over HTTP at `/mcp`; `polterware mcp` becomes a stdio ↔ HTTP bridge so clients that launch stdio servers keep working.
7. **CLI commands are thin clients.** They parse flags, read local files, call typed Connect RPCs, and print results. `polterware format` stays local.

## Alternatives considered

- **Keep direct writers; the core polls `PRAGMA data_version` or watches the WAL file.** Fixes visibility but keeps several writers, duplicates session state across processes, and needs table fingerprints to know what changed.
- **Route writes through the core only when it is running.** Two write paths, and writes made while the core is down still bypass it.
- **Always-on login service (launchd).** No start latency, but a permanent process on the machine; the idle-exit daemon gives the same behavior while in use.

## Consequences

- The Desktop, TUI, agents, and CLI see the same state live; the Connect contract is the single local API and the model for Cloud.
- The first CLI or MCP call after the core exits pays a start (well under a second). The Claude Code `hook lint` path becomes a local RPC.
- Rolled out in phases (slice 3d): core lifecycle, event stream + live Desktop, MCP in core, CLI as client, TUI as client. Until the CLI and MCP phases land, their direct writes are still visible only through reloads.
