# Polterware Agent Instructions

## Communication and editing

- Speak to Erick in Brazilian Portuguese.
- Write code, UI strings, technical documentation, commits, logs, and identifiers in English.
- Preserve unrelated work and make the smallest safe change.
- Do not create branches or commits unless explicitly requested.
- Do not run browsers, automated tests, builds, previews, distributions, or development servers unless explicitly requested in the current task.

## Work cards

- Codex and other agents: run `polterware format` once for the Polter grammar, read with `polterware query cards`, and write Work cards with `polterware apply <file|->` (frontmatter `summary` and `rationale`). Claude Code gets the same through the `plugins/polterware` plugin.

## Product positioning

Polterware's category is **Project Context & Change Intelligence for AI Coding** ([`docs/product-vision.md`](docs/product-vision.md) v3, [ADR 0004](docs/adr/0004-product-positioning-and-wedge.md)). Evolution priority is the wedge: PPM facts/evidence, deterministic import, Context Compiler, stable MCP reads, Drift, AgentRun, audited MCP writes, CLI, and Unified Change History. Operations, Terminal-as-product, and Nodes/remote execution were removed on the kill branch ([`docs/architecture/kill-ops-terminal-nodes.md`](docs/architecture/kill-ops-terminal-nodes.md)). Integrations remain as evidence/Connect UI only. Wedge delivery order: [`docs/architecture/wedge-validation-sequence.md`](docs/architecture/wedge-validation-sequence.md) (B1–B9 complete).

Active redesign (Atlas + Grounded SDD, branch `feat/atlas-grounded-sdd`): read [`docs/architecture/atlas-grounded-sdd-roadmap.md`](docs/architecture/atlas-grounded-sdd-roadmap.md) at session start for the thesis, what is done, and the next slice. It is a working roadmap, not shipped behavior.

## Current product boundary

Polterware's first version is a local SQLite-backed Desktop with project surfaces Inbox, Atlas, Work, Decisions, Reality, Agents, and Integrations. Settings is fixed in the sidebar footer. Atlas / Canvas is the default route.

The Desktop supports multiple local projects and environments. It has no hosted or shared workspace model. One local core process (`polterware serve`) per database is the only process that opens SQLite ([ADR 0007](docs/adr/0007-single-local-core.md)); Electron Main, the CLI, the TUI, and the MCP bridge attach to it or start it. The renderer reads and writes through `window.polterware` only.

The Desktop may execute on the local machine through the local core only. The renderer never spawns shells or touches Node APIs; it talks through `window.polterware`.

- Integrations: detect CLI binaries, start provider authentication commands in a core-owned PTY, optionally open provider documentation through Main `openExternal`, and probe connection status. Credentials stay in the OS/CLI credential store. **KEEP** as evidence/Connect path (not a provider console). Residual `terminals.*` IPC and `providers/terminal` exist only for that Connect auth PTY path.

The integration catalog contains exactly GitHub, GitLab, AWS, Google Cloud, Azure, Vercel, Supabase, Terraform, and Docker. Do not add placeholders or app-owned provider-token fields, and never persist provider credentials in SQLite. Raw PTY output is not guaranteed to be sanitized; do not describe it as redacted.

## Architecture boundaries

- Go local authority: the single `polterware` binary (`backend/runtimes/cli`: TUI, commands, `serve`, `mcp`) and retained domains only.
- Desktop: `apps/desktop` with thin Electron Main, typed preload bridge, and Solid renderer.
- Features: `packages/features/<feature>`.
- Shared frontend: `packages/frontend`.
- Contracts: `schemas` to generated Go code in `backend/platform/gen` and TypeScript in `packages/contracts`.
- Renderer access: `window.polterware` only; no raw Electron or Node.js.
- The core uses a technical bearer session token without an account directory.
- There is one clean SQLite baseline and no compatibility requirement for earlier databases.

## Product navigation

- Inbox: flat page (empty state; nothing routed yet)
- Atlas: Canvas, Views
- Work: Board, Specs
  - Board groups Work cards (`work_cards`, ADR 0006) by stage (Shape, Plan, Build, Verify, Delivered). The Desktop reads through `WorkService.ListWorkCards` and writes Polter text through `WorkService.ApplyPolter`; agents use `polterware apply`.
  - Specs hold one Constitution per project (`constitution`) above a table of the same cards. The five-document Spec / Plan / Tasks / Clarify / Checklist model is removed.
- Decisions: ADRs (Library restricted to `adr`)
- Reality: Inventory, Sources, Facts, Docs
  - Sources lists registered project-relative paths (`kind = source`); Import from project root runs `ImportProjectSources` for the MVP file set. Generic filesystem indexing beyond that set is not shipped.
  - Docs lists `doc` and `runbook` items.
- Agents: Access, History, Context
- Integrations: Connected, Catalog (**KEEP** — evidence + Connect UI)
- Settings: General, Environments, Shortcuts, Advanced

Keep project and environment switchers, Commander, history controls, and Settings. Do not introduce additional product surfaces without an approved specification update. Do not reintroduce removed Ops/Terminal/Nodes surfaces.

## UI law

`DESIGN.md` and `@polterware/frontend-design-system` are the visual source of truth. Product features must use shared `Input`, `Textarea`, `Select`, `Field`, `Button`, `Card`, `Surface`, Ark-backed overlays, `PageToolbar`, `FilterChip`, `DataTable`, `DitherIconTile`, `StatusChip`, and the global `DetailPanel` where applicable.

Do not hand-roll shared control chrome, menus, dialogs, operational tables, or entity panels. Use only radius and color tokens. Structural surfaces and text are opaque; alpha is limited to shared border, shadow, scrim, and transition tokens. Exception: on macOS the main window title bar and sidebar may use Electron vibrancy with `--color-shell-glass` washes; Commander and other product chrome stay opaque. Hugeicons Free Stroke Rounded is the only product icon set.

## Verification

After relevant changes, use static checks allowed by the task: typecheck, `pnpm design:lint`, `buf lint`, `go vet ./...`, and `git diff --check`. Maintain risk-proportional tests, but do not run them without explicit permission.

Previous specifications live under `docs/future/` and are non-normative.
`docs/spec.md` defines implemented behavior. `docs/product-vision.md` v3 and ADR 0004 define product direction; never present planned post-wedge depth (Change Gates, Web App, Cloud) as already shipped.
