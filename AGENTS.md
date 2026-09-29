# Polterware Agent Instructions

## Communication and editing

- Speak to Erick in Brazilian Portuguese.
- Write code, UI strings, technical documentation, commits, logs, and identifiers in English.
- Preserve unrelated work and make the smallest safe change.
- Do not create branches or commits unless explicitly requested.
- Do not run browsers, automated tests, builds, previews, distributions, or development servers unless explicitly requested in the current task.

## Work cards

- Codex and other agents: run `polterware format` once for the Polter grammar, read with `polterware query cards`, and write Work cards with `polterware apply <file|->` (frontmatter `summary` and `rationale`). Claude Code gets the same through the `plugins/polterware` plugin (a symlink to `backend/providers/agentsetup/plugin`, which the binary embeds); `polterware setup claude|codex` installs it, and `polterware hooks enable|disable <name>` switches its hooks.
- Codex and other agents: start a session with `polterware status` and end it with `polterware handoff`.

## Product positioning

Polterware's category is **Project Context & Change Intelligence for AI Coding**: every agent and person reads the same evidence-backed project state, and drift from declared intent is flagged. The shipped wedge is PPM facts/evidence, deterministic import, Context Compiler, stable MCP reads, Drift, AgentRun, audited MCP writes, CLI, Unified Change History, the Polter format with Work cards, the Atlas, the Inbox, and the decision lifecycle. Operations, Terminal-as-product, and Nodes/remote execution were removed. Integrations remain as evidence/Connect UI only.

[`docs/future.md`](docs/future.md) is the only planning document and lists what is **not** implemented (Cloud, Jev, enrichment backlog, research). It is direction, not shipped behavior; never present its items as shipped. The code, this file, `polterware --help`, and `polterware format` describe what exists.

## Current product boundary

Polterware's first version is a local SQLite-backed Desktop with project surfaces Inbox, Work, Atlas, Decisions, Reality, Agents, and Integrations. Settings is fixed in the sidebar footer. Inbox is the default route.

The Desktop supports multiple local projects and environments. It has no hosted or shared workspace model. One local core process (`polterware serve`) per database is the only process that opens SQLite (it is a detached daemon guarded by a lock, with a build handshake); Electron Main, the CLI, the TUI, and the MCP bridge attach to it or start it. The renderer reads and writes through `window.polterware` only.

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

- Inbox: flat page of derived items — Decision (open card questions), Drift (warning/conflict findings), Verify (facts with no evidence); count in the sidebar, also in `polterware status` and `polterware inbox`
- Work: Board, Specs
  - The Polterware store, not Polter text, is the source of truth; `.pw.md` files are input only.
  - Board groups Work cards (`work_cards`) by stage (Shape, Plan, Build, Verify, Delivered). The Desktop reads through `WorkService.ListWorkCards` and writes Polter text through `WorkService.ApplyPolter`; agents use `polterware apply`.
  - Specs hold one Constitution per project (`constitution`) above a table of the same cards. The five-document Spec / Plan / Tasks / Clarify / Checklist model is removed.
- Atlas: Map, Hierarchy, Views (nodes: system, domain, component; a feature is a Work card)
- Decisions: ADRs by lifecycle state (Active, Challenged, Proposed, Superseded)
- Reality: Inventory, Sources, Facts, Docs
  - Sources lists registered project-relative paths (`kind = source`); Import from project root runs `ImportProjectSources` for the MVP file set. Generic filesystem indexing beyond that set is not shipped.
  - Docs lists `doc` and `runbook` items.
- Agents: Access, Sessions, History, Context
- Integrations: Connected, Catalog (**KEEP** — evidence + Connect UI)
- Settings: General, Environments, Shortcuts, Advanced

Keep project and environment switchers, Commander, history controls, and Settings. Do not introduce additional product surfaces without an approved decision. Do not reintroduce removed Ops/Terminal/Nodes surfaces.

## UI law

`DESIGN.md` and `@polterware/frontend-design-system` are the visual source of truth. Product features must use shared `Input`, `Textarea`, `Select`, `Field`, `Button`, `Card`, `Surface`, Ark-backed overlays, `PageToolbar`, `FilterChip`, `DataTable`, `DitherIconTile`, `StatusChip`, and the global `DetailPanel` where applicable.

Do not hand-roll shared control chrome, menus, dialogs, operational tables, or entity panels. Use only radius and color tokens. Structural surfaces and text are opaque; alpha is limited to shared border, shadow, scrim, and transition tokens. Exception: on macOS the main window title bar and sidebar may use Electron vibrancy with `--color-shell-glass` washes; Commander and other product chrome stay opaque. Hugeicons Free Stroke Rounded is the only product icon set.

## Verification

After relevant changes, use static checks allowed by the task: typecheck, `pnpm design:lint`, `buf lint`, `go vet ./...`, and `git diff --check`. Maintain risk-proportional tests, but do not run them without explicit permission.

Do not add Markdown design or planning documents. Record decisions and work in the Polterware store (`polterware apply`); put unshipped direction in `docs/future.md` and delete items from it when they ship.
