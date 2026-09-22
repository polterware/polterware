# Kill branch: Operations / Terminal / Nodes

Status: Done (UI, IPC, Go domains, protos, SQLite DROP, and docs)

Authority: [ADR 0004](../adr/0004-product-positioning-and-wedge.md), [`docs/product-vision.md`](../product-vision.md) v3. Shipped behavior is defined in [`docs/spec.md`](../spec.md).

## Goal

Remove condemned product surfaces without breaking Integrations Connect or Resources Inventory, and without rewriting `polterware-core` in another language.

## Removal order

1. **UI** — hide/remove sidebar sections Operations and Terminal; Resources tabs Nodes and Processes; title-bar Terminal shortcut; Commander overview Ops/Processes cards; create dialogs and empty templates for those surfaces.
2. **Feature packages** — drop `@polterware/feature-operations` product wiring; remove Terminal product pages from `@polterware/feature-shell` while **keeping** `TerminalEmulator` / `TerminalTranscript` for Connect.
3. **IPC / preload** — remove Operations/Workflows/Schedules/Processes/Nodes product channels that nothing else needs; **keep** `terminals.*` for Connect auth PTY. **Done.**
4. **Go domains** — Operations, Workflows, schedule runner, Process sessions product RPCs; NodeService. **Done.**
5. **Schemas** — delete unused ConnectRPC services; regenerate Go/TS. **Done.**
6. **SQLite** — migration `0010_kill_ops_terminal_nodes.sql` drops `operations`, `workflows`, `operation_schedules`, `log_chunks`, `process_sessions`, `nodes`. **Done.**

## Must remain

| Keep | Reason |
| --- | --- |
| Resources Inventory | Project resource registry for PPM / Architecture |
| Integrations Connect + CLI probe | Evidence path; uses core PTY + `os/exec` |
| `providers/terminal` + `terminals.*` IPC | Interactive auth in Connect wizard |
| Architecture Canvas | Human inspector of the model (not the wedge thesis) |
| Knowledge / Agents / PPM / Context / MCP | Wedge foundation |

## Acceptance

- Sidebar has no Operations or Terminal product sections; Resources is a flat Inventory page (no tab rail).
- Connect provider auth still opens an in-app PTY and completes probe (`auth-*` / local targets require no nodes table).
- `polterware-core serve` and `mcp` start cleanly.
- `docs/spec.md` lists five project surfaces + Settings (no condemned Ops/Terminal/Nodes IA).
- Static checks: typecheck, `pnpm design:lint`, `go vet ./...`, `git diff --check`.

## Risks

- Connect auth targets (`auth-*`) must not require product Nodes lookup — handled by `isSyntheticLocalTarget`.
- Runbook “Run” previously created Operations — retired with the runner.
- Shared `providers/process` removed; short-lived spawn lives in `providers/exec` for git/pkg helpers.

## Explicit non-goals of this kill branch

- Implementing wedge Facts/Evidence, Drift, AgentRun, or CLI (separate slices).
- Deleting `docs/future/` archive.
