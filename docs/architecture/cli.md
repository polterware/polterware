# CLI `polterware` (B8)

Status: Shipped; commands are clients of the single local core ([ADR 0007](../adr/0007-single-local-core.md))

Authority: [ADR 0004](../adr/0004-product-positioning-and-wedge.md), [`wedge-validation-sequence.md`](wedge-validation-sequence.md), [`docs/product-vision.md`](../product-vision.md) v3.

## Purpose

A single local Go binary serves the local core (`serve`), the MCP stdio bridge (`mcp`), and headless commands for project bootstrap, deterministic import, context compile, narrow drift, and Work cards — all through the one core that owns the SQLite store.

## Binary

| Item | Value |
|------|-------|
| Binary | `polterware` (`backend/runtimes/cli`) |
| Build | `pnpm build:cli` → `dist/polterware` |
| Database | `-db` (default: the Desktop database path) |

Only `serve` opens SQLite. `init`, `scan`, `repo`, `context`, `drift`, `apply`, `lint`, `query`, `status`, `handoff`, `hook`, `mcp`, and the TUI call the core over loopback Connect (`backend/platform/coreclient`), starting it when none is running; they do not require the Desktop. `format` is local. Git reads for `status` and `handoff` run inside the core, at the project root, with no network. Extra checkouts registered with `repo` are read only by `scan`.

## Commands

| Command | Behavior |
|---------|----------|
| `init` | Register a project (`SaveProject` + empty layout/settings seed) and a default `local` environment |
| `scan` | Deterministic MVP import (`ImportProjectSources` / B2). Git checkpoint of the project root, plus each checkout registered with `repo` |
| `repo` | `list`, `add --name <slug> --path <dir>`, `remove --name <slug>`. Extra local checkouts for one project. The project directory stays the primary checkout |
| `context` | Context Compiler package (B3/B4 shape) |
| `drift` | Narrow Drift Checker result (`checker: "narrow"`) |
| `apply` | Polter text → plan → one audited write to the Work model ([ADR 0006](../adr/0006-polter-format-and-work-model.md)) |
| `lint` | Validate Polter text; silent on success, one line per error or `hint:`, exit 1 only on errors |
| `format` | Print the Polter format spec |
| `query` | Work state in Polter format (`cards`, `card <slug>`, `status`, `handoff <slug>`) |
| `status` | Current card, branch, last handoff on that branch, open questions (blocking first), stage. `delivered` only when the card stage is `delivered` and `HEAD` is an ancestor of local `main` or `master` |
| `handoff` | Write a handoff for the card and current branch (`--card`, `--note`), cross-checked against the local diff |
| `hook` | Claude Code hook entry points (`session-start`, `session-end`, `lint`, `status`, `stop`), reading the hook JSON on stdin |
| `serve` | The local core: Connect services, `ChangeService.WatchChanges`, and MCP at `/mcp`, all behind the bearer session token (`--listen`, default `127.0.0.1:0`; `--token`, default `$POLTERWARE_SESSION_TOKEN` or minted; `--idle`, default 10m or `$POLTERWARE_CORE_IDLE`, `0` never exits). Exits 3 when another core holds the database lock |
| `mcp` | MCP stdio bridge: forwards each JSON-RPC line to the core's `/mcp` and re-initializes transparently when the core restarted |

### Flags

```
polterware [-db path] init --name <name> [--path <root>] [--id <id>] [--json]
polterware [-db path] scan --project <id> [--json]
polterware [-db path] context --project <id> [--query ...] [--entity ...] [--budget N] [--agent-run-id ...] [--json]
polterware [-db path] drift --project <id> [--entity ...] [--agent-run-id ...] [--json]
polterware [-db path] apply <file|-> [--dry-run] [--project <id>] [--summary ...] [--rationale ...] [--agent-run-id ...]
polterware [-db path] lint <file|dir|-> [--project <id>]
polterware [-db path] format
polterware [-db path] query "cards" | "card <slug>" | "status" | "handoff <slug>" [--project <id>]
polterware [-db path] status [--project <id>]
polterware [-db path] handoff [--card <slug>] [--note "..."] [--project <id>]
polterware [-db path] hook session-start [--label <name>] | session-end | lint | status | stop
polterware [-db path] serve [--listen addr] [--token token] [--idle duration]
polterware [-db path] mcp
```

- `init --path` defaults to the current working directory (resolved absolute).
- `init --id` optional; otherwise a `proj-<hex8>` id is minted.
- `--json` prints one JSON object on stdout (same shapes as MCP for `context` / `drift`).
- Optional `--agent-run-id` on `context` / `drift` appends a step when the run is open (same rules as MCP).
- `apply`, `lint`, `query`, `status`, and `handoff` resolve the project from `--project` or from the registered project whose root contains the current directory. Flags may follow the file argument.
- `handoff` without `--card` uses the only active card that already has a handoff on the current branch, otherwise the only active card. `--note` accepts `Goal:`, `Done:`, `In progress:`, `Next:`, `Touched:`, and `Open question:` lines; a note without those labels is in progress. Branch and, when `Touched:` is absent, touched files come from local Git. A declared `Touched:` or `Branch:` that disagrees with Git is a `hint:`, and the write still lands. The change envelope defaults to `Handoff for <slug>` / `Session handoff`.
- `hook status` prints `status` as Claude Code `additionalContext` and exits 0 if the core is down. `hook stop` reminds the agent to run `handoff` and does not block the session.
- `apply` without `--agent-run-id` joins the agent session's run when `POLTERWARE_SESSION` is set (one run per project, opened on the first write); otherwise it opens and completes an agent AgentRun labelled `cli` for the write.
- `hook session-start` appends `POLTERWARE_SESSION` and `POLTERWARE_AGENT_LABEL` (default `claude-code`) to `$CLAUDE_ENV_FILE`; it opens no run. `hook session-end` asks the core to complete the session's runs (`WorkService.EndSession`). `hook lint` lints the written file when it ends in `.pw.md` and exits 2 with errors and hints on stderr.

Human-readable summaries go to stdout; errors to stderr with non-zero exit.

## Status TUI

`polterware` with no subcommand, when stdin and stdout are terminals, opens a Bubble Tea TUI (`backend/runtimes/cli/tui.go`). Otherwise it prints usage and exits 2, so scripts are not affected.

| Row | Source | Action |
|-----|--------|--------|
| Core | `<db dir>/run/core.json` | The TUI starts the core on launch. `enter` / `s` stops it (`MaintenanceService.Shutdown`) or starts it again (detached `polterware -db <db> serve --listen 127.0.0.1:0`, log `run/core.log`). A running Desktop restarts a stopped core. |
| Clients | `claude mcp get`, `codex mcp get`, `~/.cursor/mcp.json` | `enter` configures or removes the `polterware` server as `<polterware> -db <db> mcp` (absolute path of the running executable) after a `y` confirmation. Claude Code uses `--scope user`. For Cursor, only `mcpServers.polterware` is merged; invalid JSON is never overwritten. |
| MCP | `ListMcpSessions` | Live MCP sessions in the core with client name/version from `initialize.clientInfo`. |
| Runs / Calls / History | `ListAgentRuns`, `ListAgentToolCalls`, `ListChanges` | Read-only through the core, refreshed every 2s. |

`p` cycles projects. The default is the project whose root contains the cwd, otherwise the most recently updated project. `i` registers the cwd (`InitProject`) and scans it.

### Run state

`backend/platform/runstate` owns `<db dir>/run/` (dir 0700, files 0600):

- `core.lock` — exclusive `flock` held by the running core; a second `serve` exits 3.
- `core.json` — `pid`, `endpoint`, `token`, `dbPath`, `build` (`<mtime unix>-<size>` of the core binary), `startedAt`. It is written after `serve` listens and removed on shutdown.
- `core.log` — stdout/stderr of detached cores.

A stale `core.json` (dead pid) is pruned on read. Every client, Electron Main included, attaches to the live core whose `dbPath` resolves to its database, and replaces it when its own binary has a newer build. MCP sessions and agent session runs live in the core's memory.

## Shared composition

MCP and CLI call [`backend/compositions/wedge`](../../backend/compositions/wedge): `InitProject`, `ScanProject`, `CompileContext`, `CheckDrift`. Desktop Connect RPC continues to expose `ImportProjectSources` / `CompileContext` via local-core, which uses the same scan/compile paths where extracted.

## Out of scope

- Packaging the CLI into Electron `extraResources` or Desktop PATH installers
- Legacy big-bang commands (`health`, `ops`, `systems`, `pipelines`)
- Unified Change History (B9)

## Code

- Composition: `backend/compositions/wedge`
- Runtime: `backend/runtimes/cli`
- Related: [`deterministic-import.md`](deterministic-import.md), [`stable-mcp-reads.md`](stable-mcp-reads.md), [`narrow-drift-checker.md`](narrow-drift-checker.md)
