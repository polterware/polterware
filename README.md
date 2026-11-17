# Polterware

Polterware is a local-first project context and change-intelligence tool for AI coding. It keeps one evidence-backed model of a software project (what it is, what was decided, what is in progress, what was delivered) so every coding agent and person starts from the same state, and each card shows the commits that did its work.

## Current product

A local SQLite-backed Desktop over a single Rust binary, `polterware`. The Desktop is a native GPUI app in Rust (`app`), on the shared [polterware/ui](https://github.com/polterware/ui) library. Project navigation:

- **Inbox** — items that need a person: Conflicts an agent raised against a Rule and open card questions
- **Milestones** — the Roadmap of boards; each milestone is a Board (cards by stage: Idea, Planned, In progress, In review, Done) and a List of the same cards
- **Structure** — the project tree of domains and components, each component optionally linked to a repository
- **Canvas** — canvases that mix drawing shapes with components linked to Structure nodes
- **Knowledge** — Rules and Docs
- **Activity** — the Unified Change History of every justified change, by person or agent
- **Project settings** — General, Danger zone
- **App settings** — Agents (MCP server and clients, Agent setup, Hooks, AI skill, CLI reference) and App

Agents talk to Polterware in the Polter format (markdown that writes Work cards, Structure nodes, and Rules) through the CLI or MCP (`polter.apply`, `polter.query`, `polter.format`, plus `doc.upsert`). Every write carries a summary and rationale and lands in one change history.

One local core process (`polterware serve`) per database is the only process that opens SQLite. The Desktop, the CLI, and the MCP bridge attach to it or start it; the Desktop reads and writes only through the core, over Connect-JSON. Provider credentials stay in the OS or CLI credential store and are never persisted in SQLite.

Only the local deployment mode exists. Cloud, Jev, and other unshipped direction are listed in [docs/future.md](docs/future.md).

## Using it with agents

```bash
polterware status                 # current card, branch, last handoff, open questions
polterware format                 # Polter grammar
polterware query cards            # one line per card
polterware apply plan.pw.md       # write cards (frontmatter needs summary and rationale)
polterware handoff --note "..."   # end a session
polterware setup claude           # install the Claude Code plugin (or: setup codex)
polterware setup mcp add all      # register the MCP server with Claude Code, Codex, and Cursor
```

Run `polterware --help` for every command.

## Install

Polterware ships as alpha builds for macOS on Apple Silicon. Install, or update an installed copy, with:

```bash
curl -fsSL https://raw.githubusercontent.com/polterware/polterware/main/scripts/install-desktop.sh | bash
```

Run the same command again to update.

## Repository

- `app` — the Desktop (Rust, GPUI, on [polterware/ui](https://github.com/polterware/ui)), reading the core over Connect-JSON; its icons are `app/resources/polterware.icon`, `polterware-ice.icon`, and `polterware-fire.icon`
- `core` — Rust runtime (single `polterware` binary: CLI, `serve`, `mcp`)
- `schemas` — Protobuf contracts, generated into Rust contracts shared by the core and Desktop (`tooling/gen-contracts`); `schemas/vendor/dispatch` holds Dispatch's contracts, synced by `tooling/sync-dispatch-proto.sh`
- `DESIGN.md` — visual and interaction contract
- `docs/future.md` — planned work, not implemented

## Development

The root is a Cargo workspace (`app`, `core`, `tooling/gen-contracts`).

```bash
cargo run -p polterware-gpui      # the Desktop (or ./scripts/dev.sh)
./scripts/dev.sh --core-only      # the local core alone
bash tooling/generate.sh          # regenerate Rust contracts from schemas/
```

The development script builds the core into `dist/polterware`, where the Desktop finds it. Static checks:

```bash
cargo fmt --check
cargo clippy --all-targets
cargo test
cd schemas && buf lint
git diff --check
```

## Desktop releases

`bash tooling/release/bundle.sh` builds an ad-hoc signed macOS arm64 `Polterware.app` with the core, the icon, and Sparkle, plus its DMG, ZIP, and `SHA256SUMS`, under `target/release-bundle/<version>` (the workspace version in `Cargo.toml`). There is no release workflow: releases are built and published by hand. Builds are not notarized, so macOS asks for confirmation on first open.

## Ecosystem

Polterware is both the umbrella and the main app, with [Polterware Dispatch](https://github.com/polterware/dispatch), [Vitals](https://github.com/polterware/vitals), and other tools that integrate with each other, each in its own repository, on the shared GPUI library [polterware/ui](https://github.com/polterware/ui). What qualifies, and what stays a personal project, is in [docs/ecosystem.md](docs/ecosystem.md).

## Open source and Cloud

This repository is the public product: Desktop, local runtime, and contracts. Polterware Cloud is a separate managed service on a private control plane and is not implemented here. The public website and future Web App live in `polterware-web`.

## Contributing and security

See [CONTRIBUTING.md](CONTRIBUTING.md), [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md), [TRADEMARKS.md](TRADEMARKS.md), and [SECURITY.md](SECURITY.md).
