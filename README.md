# Polterware

Polterware is a local-first project context and change-intelligence tool for AI coding. It keeps one evidence-backed model of a software project (what it is, what was decided, what is in progress, what was delivered) so every coding agent and person starts from the same state, and each card shows the commits that did its work.

## Current product

A local SQLite-backed Desktop over a single Go binary, `polterware`. The shipped Desktop is Electron + Solid (`apps/polterware-desktop`); its GPUI successor is `desktop-gpui`, which replaces it once it reaches parity. Project navigation:

- **Inbox** — items that need a person: Conflicts an agent raised against a Rule or Decision, failed deploys, open card questions, and deploys whose provider CLI is not connected
- **Milestones** — the Roadmap of boards; each milestone is a Board (cards by stage: Idea, Planned, In progress, In review, Done) and a List of the same cards
- **Structure** — the project tree of domains and components, each component optionally linked to a repository, with its deploys
- **Canvas** — canvases that mix drawing shapes with components linked to Structure nodes
- **Knowledge** — Decisions (with a state filter, principles pinned), Rules, and Docs
- **Activity** — the Unified Change History of every justified change, by person or agent
- **Project settings** — General, Environments, Platforms (the provider CLIs, sign-in only), Danger zone
- **App settings** — Agents (MCP server and clients, Agent setup, Hooks, AI skill, CLI reference) and App

Agents talk to Polterware in the Polter format (markdown that writes Work cards, Structure nodes, and decisions) through the CLI or MCP (`polter.apply`, `polter.query`, `polter.format`, plus `doc.upsert`). Every write carries a summary and rationale and lands in one change history.

One local core process (`polterware serve`) per database is the only process that opens SQLite. Electron Main, the GPUI app, the CLI, and the MCP bridge attach to it or start it. The renderer reads and writes only through `window.polterware`. Provider credentials stay in the OS or CLI credential store and are never persisted in SQLite.

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

- `apps/polterware-desktop` — Electron Main, preload bridge, and Solid renderer
- `desktop-gpui` — the GPUI Desktop (Rust, on [polterware/ui](https://github.com/polterware/ui)), reading the core over Connect-JSON
- `apps/gallery` — the design system gallery
- `packages/features` — product surfaces
- `packages/frontend` — shared design system, canvas, and state
- `backend` — Go runtime (single `polterware` binary: CLI, `serve`, `mcp`)
- `schemas` — Protobuf contracts and generated Go/TypeScript messages; `schemas/vendor/dispatch` holds Dispatch's contracts, synced by `tooling/sync-dispatch-proto.sh`
- `DESIGN.md` — visual and interaction contract
- `docs/future.md` — planned work, not implemented

## Development

```bash
pnpm install
pnpm dev:polterware       # the Electron Desktop
pnpm dev:gallery          # the design system gallery
cargo run --manifest-path desktop-gpui/Cargo.toml   # the GPUI Desktop
```

The Desktop `dev` builds its local core first and starts it with the app. The same `<task>[:<app>]` pattern applies to `build`, `preview`, `typecheck`, and `test`. Dev ports are fixed: Polterware 5173, gallery 5177. For standalone core development:

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

## Desktop releases

`bash tooling/release/dist-mac.sh polterware` (or `pnpm dist:polterware:mac`) builds an ad-hoc signed macOS arm64 DMG and ZIP under the app's `release/<version>` with `SHA256SUMS`, a bundle verification, and a build report.

Pushing a tag `polterware-v<version>` (for example `polterware-v0.1.0-alpha.1`) runs the release workflow, which attaches the artifacts to a draft GitHub release; a version with a suffix is marked as a pre-release. Nothing is published automatically. Builds are not notarized, so macOS asks for confirmation on first open, and updates are manual: there is no auto-update feed.

## Ecosystem

Polterware is both the umbrella and the main app, with [Polterware Dispatch](https://github.com/polterware/dispatch), [Vitals](https://github.com/polterware/vitals), and other tools that integrate with each other, each in its own repository, on the shared GPUI library [polterware/ui](https://github.com/polterware/ui). What qualifies, and what stays a personal project, is in [docs/ecosystem.md](docs/ecosystem.md).

## Open source and Cloud

This repository is the public product: Desktop, local runtime, contracts, and shared UI. Polterware Cloud is a separate managed service on a private control plane and is not implemented here. The public website and future Web App live in `polterware-web`.

## Contributing and security

See [CONTRIBUTING.md](CONTRIBUTING.md), [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md), [TRADEMARKS.md](TRADEMARKS.md), and [SECURITY.md](SECURITY.md).
