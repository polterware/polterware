# Sparky Next — migration plan

Status: **ready for Wave 0**. Owner: Erick. Audience: coding agents (Cursor) running in parallel, one per workstream (WS).
Language: this plan, code, commits and UI strings are English (repo rule). Chat with Erick in Brazilian Portuguese.

> **AGENTS.md exception.** `AGENTS.md` says "Do not add Markdown design or planning documents". Erick approved this
> directory as a temporary exception. Delete `plans/sparky-next/` when W3-CLEANUP finishes. Do not add other planning docs.
> For the duration of this plan, the execution rules in [AGENT-PROTOCOL.md](AGENT-PROTOCOL.md) override the AGENTS.md
> restrictions on branches, commits, tests, builds, dev servers and browsers.

## Goal

Rebuild **Sparky** (today SwiftUI + SwiftData + a Bun MCP server with mirror/queue/pairing) as a desktop app on the Polterware
stack, sharing the Polterware design system so both apps look like one product family (like Notion and Notion Calendar):

- UI: Electron + Solid + Tailwind 4 + Ark UI + Motion, using `@polterware/frontend-design-system`.
- Backend: a **Go core** (`sparky`) that is the only process opening SQLite (WAL), exposes the UI API (Connect RPC) and the
  **MCP server** on loopback. No hosted server, no pairing, no mirror, no command queue.
- Reminders: computed by the core, delivered by the **operating system** through a tiny signed Swift login-item helper
  (`SparkyNotifier`), so banners fire with every Sparky process closed.
- Separate app, separate code, separate backend from Polterware. Shared: design system, tokens, brand assets, repo.

Nobody uses the current Sparky (including Erick). **There is no data migration, no importer, no Sparkle path, no coexistence.**
The Swift app stays untouched as a read-only reference.

## v1 scope (everything below ships in this plan)

Core: Minds (hierarchy), Memories (title, Markdown note, status, pin, priority, due date), checklist, tags, schedule +
recurrence + reminders, Inbox/Today views, MCP.
Plus: calendar (day/week/month), focus (pomodoro, standalone or tied to a memory), review (stats, streak, contribution
calendar), attachments (photo, audio, link, file), **free map** (places on memories, no geofence arming), full-text search
(FTS5 only), expanded MCP (tags, attachments, batch writes, occurrences, review), CLI + local API.
Out of scope: iPhone, geofence arming, accounts, sync, Cloud, Windows/Linux, App Store.

## Read first (in this order)

| # | Doc | What it gives you |
|---|-----|-------------------|
| 1 | [AGENT-PROTOCOL.md](AGENT-PROTOCOL.md) | How to work: worktrees, ownership, merges, definition of done |
| 2 | [00-decisions.md](00-decisions.md) | Locked decisions, assumptions, open questions |
| 3 | [01-architecture.md](01-architecture.md) | Processes, paths, auth, data flows, packaging |
| 4 | [02-data-and-contracts.md](02-data-and-contracts.md) | SQL schema, RPC surface, error model |
| 5 | [03-recurrence-and-reminders.md](03-recurrence-and-reminders.md) | Schedule semantics, test vectors, notifier protocol |
| 6 | [04-mcp.md](04-mcp.md) | MCP transports, tools, token budgets |
| 7 | [05-search-attachments-map.md](05-search-attachments-map.md) | FTS5, attachments, map |
| 8 | [06-ui.md](06-ui.md) | Brand, shell, screens, design-system work |
| 9 | [07-wave0.md](07-wave0.md) … [10-wave3-and-verification.md](10-wave3-and-verification.md) | Workstreams per wave |
| 10 | [11-swift-reference-map.md](11-swift-reference-map.md) | Which Swift file explains what |
| 11 | [12-risks.md](12-risks.md) | Risks and fallbacks |

## Waves and gates

| Wave | Purpose | Workstreams (parallel) | Gate to leave the wave |
|------|---------|------------------------|------------------------|
| **0** | Scaffold, spikes, contracts freeze | W0-SCAFFOLD, W0-CONTRACTS, W0-NOTIFIER, W0-MCP, W0-STORE, W0-MAP, W0-DS-AUDIT | **G0**: notifier spike passes; MCP SDK, FTS5/vector and map provider decided; contracts tagged `sparky-contracts-v1` |
| **1** | Backend, Electron shell, design system, notifier | W1-STORE, W1-DOMAIN, W1-RECURRENCE, W1-CORE, W1-MCP, W1-SEARCH, W1-ATTACH, W1-ELECTRON, W1-DS, W1-NOTIFIER, W1-ASSETS | **G1**: Go tests green, MCP smoke green, DS non-regression for Polterware, Electron boots against the real core |
| **2** | Features (UI + CLI) | W2-SHELL, W2-MEMORIES, W2-MINDS, W2-SCHEDULE, W2-CALENDAR, W2-ATTACH-UI, W2-FOCUS, W2-REVIEW, W2-MAP, W2-SETTINGS, W2-SEARCH-UI, W2-CLI | **G2**: every feature works end to end against the real core; `verify-sparky` green |
| **3** | Integration, security, packaging, polish | W3-E2E, W3-SECURITY, W3-PACKAGING, W3-POLISH, W3-CONVERGE, W3-CLEANUP | **G3**: release checklist in [10-wave3-and-verification.md](10-wave3-and-verification.md) |

A wave starts only after the previous gate passes. If a Wave-0 spike fails, follow its fallback in [12-risks.md](12-risks.md) and
amend [00-decisions.md](00-decisions.md) **before** Wave 1 starts.

## Target directory layout (created by W0-SCAFFOLD)

```text
apps/sparky-desktop/        Electron: src/main, src/preload, src/renderer, resources, electron-builder.yml
apps/sparky-notifier/       Swift login-item helper (SparkyNotifier.app)
sparky/backend/             Go module github.com/polterware/polterware/sparky/backend
  domains/                  pure domain logic (no I/O imports)
  providers/                sqlite, mcp, search, attachments
  compositions/core/        Connect handlers + wiring
  platform/                 runstate, ids, token, listen, changes, clock
  runtimes/cli/             the `sparky` binary (serve, mcp, status, doctor, CLI verbs)
  tests/                    contract + integration
sparky/schemas/             buf module (protos) -> sparky/backend/platform/gen + packages/sparky-contracts/src/gen
packages/sparky-contracts/  generated TS + hand-written helpers
packages/features/sparky-*  shell, memories, minds, schedule, calendar, attachments, focus, review, map, settings, search
packages/frontend/*         shared (design-system, state, canvas, platform) — extended by W1-DS
tooling/verify-sparky.sh    Sparky gate script
```

Reference (read-only, never edit): `/Users/erickpatrickbarcelos/codes/sparky-workspace/sparky` (Swift app, specs, docs) and
`/Users/erickpatrickbarcelos/codes/sparky-workspace/sparky-mcp` (Bun MCP server, `docs/CONTRACT.md`).
