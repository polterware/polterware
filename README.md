# Polterware

Polterware is a local-first project context and change-intelligence tool for AI coding. It keeps one evidence-backed model of a software project (what it is, what was decided, what is in progress, what was delivered) so every coding agent and person starts from the same state, and it flags when changes drift from declared intent.

## Current product

A local SQLite-backed Desktop (Electron + Solid) over a single Go binary, `polterware`. Project navigation:

- **Inbox** — items that need a person: open card questions, drift findings, and facts with no evidence
- **Work** — Board (cards by stage: Shape, Plan, Build, Verify, Delivered) and Specs (the same cards as a table)
- **Atlas** — Map and Components of the system, domain, and component graph
- **Truth** — Decisions (ADRs with a state filter, principles pinned), Facts, and Evidence (sources, docs, runbooks), all with provenance
- **Activity** — the Unified Change History of every justified change, by person or agent
- **Agents** — Access (MCP setup), Sessions, History, and Context (what an agent sees at a zoom and token budget)
- **Integrations** — Connected and Catalog (GitHub, GitLab, AWS, Google Cloud, Azure, Vercel, Supabase, Terraform, Docker); evidence and provider login only
- **Settings** — General, Environments, Shortcuts, Advanced

Agents talk to Polterware in the Polter format (markdown that writes Work cards, Atlas nodes, and decisions) through the CLI or MCP (`polter.apply`, `polter.query`, `polter.format`, plus context, drift, and AgentRun tools). Every write carries a summary and rationale and lands in one change history.

One local core process (`polterware serve`) per database is the only process that opens SQLite. Electron Main, the CLI, the TUI, and the MCP bridge attach to it or start it. The renderer reads and writes only through `window.polterware`. Provider credentials stay in the OS or CLI credential store and are never persisted in SQLite.

Only the local deployment mode exists. Cloud, Jev, and other unshipped direction are listed in [docs/future.md](docs/future.md).

## Using it with agents

```bash
polterware status                 # current card, branch, last handoff, open questions
polterware format                 # Polter grammar
polterware query cards            # one line per card
polterware apply plan.pw.md       # write cards (frontmatter needs summary and rationale)
polterware handoff --note "..."   # end a session
polterware setup claude           # install the Claude Code plugin (or: setup codex)
```

Run `polterware` with no arguments in a terminal for the status TUI. Run `polterware --help` for every command.

## Repository

- `apps/desktop` — Electron Main, preload bridge, and Solid renderer
- `packages/features` — product surfaces
- `packages/frontend` — shared design system, canvas, and state
- `backend` — Go runtime (single `polterware` binary: CLI, TUI, `serve`, `mcp`)
- `apps/sparky-desktop`, `packages/features/sparky-*`, `sparky/` — Sparky, the second app (see below)
- `schemas` — Protobuf contracts and generated Go/TypeScript messages
- `DESIGN.md` — visual and interaction contract
- `docs/future.md` — planned work, not implemented

## Development

```bash
pnpm install
pnpm dev:desktop
```

`pnpm dev:desktop` starts the Desktop together with the local core. For standalone core development:

```bash
pnpm dev:core
```

Static checks:

```bash
pnpm typecheck
pnpm design:lint
cd schemas && buf lint
cd ../backend && go vet ./...
git diff --check
```

## Sparky

Sparky is a local-first memory and reminder app that lives in this monorepo next to Polterware. It reuses the Polterware design system and has its own Go core (`sparky/backend`: SQLite with FTS5, Connect RPC, an MCP server for agents, and the `sparky` CLI), an Electron + Solid desktop (`apps/sparky-desktop`), a native macOS reminders module, and feature packages under `packages/features/sparky-*`. Contracts live in `sparky/schemas`.

```bash
pnpm install
pnpm dev:sparky          # desktop + local core
bash tooling/verify-sparky.sh
pnpm dist:sparky:mac     # ad-hoc signed dmg/zip under apps/sparky-desktop/release
```

Sparky releases use tags `sparky-v<version>`; the release workflow creates a draft. Builds are ad-hoc signed, so macOS asks for confirmation on first open and updates are manual.

## Open source and Cloud

This repository is the public product: Desktop, local runtime, contracts, and shared UI. Polterware Cloud is a separate managed service on a private control plane and is not implemented here. The public website and future Web App live in `polterware-web`.

## Contributing and security

See [CONTRIBUTING.md](CONTRIBUTING.md), [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md), [TRADEMARKS.md](TRADEMARKS.md), and [SECURITY.md](SECURITY.md).
