# CLI `polterware` (B8)

Status: Shipped for local in-process SQLite commands

Authority: [ADR 0004](../adr/0004-product-positioning-and-wedge.md), [`wedge-validation-sequence.md`](wedge-validation-sequence.md), [`docs/product-vision.md`](../product-vision.md) v3.

## Purpose

A local Go CLI shares `polterware-core` domain behavior for headless project bootstrap, deterministic import, context compile, and narrow drift — the same SQLite model Desktop and MCP use.

## Binary

| Item | Value |
|------|-------|
| Binary | `polterware` (`backend/runtimes/cli`) |
| Build | `pnpm build:cli` → `dist/polterware` |
| Database | `-db` (default: same path as `polterware-core`) |

Runtime is **in-process SQLite**. The CLI does not require Desktop or a Connect session.

## Commands

| Command | Behavior |
|---------|----------|
| `init` | Register a project (`SaveProject` + empty layout/settings seed) and a default `local` environment |
| `scan` | Deterministic MVP import (`ImportProjectSources` / B2) |
| `context` | Context Compiler package (B3/B4 shape) |
| `drift` | Narrow Drift Checker result (`checker: "narrow"`) |

### Flags

```
polterware [-db path] init --name <name> [--path <root>] [--id <id>] [--json]
polterware [-db path] scan --project <id> [--json]
polterware [-db path] context --project <id> [--query ...] [--entity ...] [--budget N] [--agent-run-id ...] [--json]
polterware [-db path] drift --project <id> [--entity ...] [--agent-run-id ...] [--json]
```

- `init --path` defaults to the current working directory (resolved absolute).
- `init --id` optional; otherwise a `proj-<hex8>` id is minted.
- `--json` prints one JSON object on stdout (same shapes as MCP for `context` / `drift`).
- Optional `--agent-run-id` on `context` / `drift` appends a step when the run is open (same rules as MCP).

Human-readable summaries go to stdout; errors to stderr with non-zero exit.

## Shared composition

MCP and CLI call [`backend/compositions/wedge`](../../backend/compositions/wedge): `InitProject`, `ScanProject`, `CompileContext`, `CheckDrift`. Desktop Connect RPC continues to expose `ImportProjectSources` / `CompileContext` via local-core, which uses the same scan/compile paths where extracted.

## Out of scope

- `POLTERWARE_SESSION_FILE` / Connect HTTP client against a live Desktop core
- Packaging the CLI into Electron `extraResources` or Desktop PATH installers
- Legacy big-bang commands (`health`, `ops`, `systems`, `pipelines`)
- Desktop UI for CLI status
- Unified Change History (B9)

## Code

- Composition: `backend/compositions/wedge`
- Runtime: `backend/runtimes/cli`
- Related: [`deterministic-import.md`](deterministic-import.md), [`stable-mcp-reads.md`](stable-mcp-reads.md), [`narrow-drift-checker.md`](narrow-drift-checker.md)
