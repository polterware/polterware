# Atlas + Grounded SDD roadmap

Status: Working roadmap (branch `feat/atlas-grounded-sdd`, started 2026-09-28)

Authority: none yet. This note consolidates the 2026-09-28 brainstorm and tracks the build order. It is not an ADR and does not replace [`docs/spec.md`](../spec.md), which only describes what is implemented. Before shipping a slice that changes product direction, record the decision (ADR or `product-vision.md` update). Source ideas: [`docs/ideas/`](../ideas/).

**How to use this file:** at the start of a session, read "Done" and pick the first unchecked slice under "Next slices". Read the design documents the slice links before coding. If several agents work at once, write the branch/agent next to the slice you take (`— in progress: <branch>`) so nobody picks the same slice. When a slice lands, tick its boxes, move its summary to "Done", and update `docs/spec.md` for the shipped behavior only.

Design documents (all proposed, none implemented):

- [`polter-format.md`](polter-format.md) — Polter format, codec, lint, MCP protocol, storage and source of truth, similarity without embeddings.
- [`cloud-architecture.md`](cloud-architecture.md) — Cloud: operation-log replication, components, MCP local and remote, Fly.io + Supabase with Go, build order C0–C4.
- [`../ideas/2026-09-28-jev-feature-catalog.md`](../ideas/2026-09-28-jev-feature-catalog.md) — Jev features for Cloud (Judge).

Key decisions so far:

- Polterware (SQLite locally, Cloud for teams) is always the source of truth. Repositories give evidence in and receive projections out (ADRs, `AGENTS.md` blocks). `.pw.md` files are optional input, never the truth.
- Polter format is the agent ↔ Polterware language; the MCP converges on `polter.apply`, `polter.query`, `polter.format`. New capabilities are Polter commands, not new MCP tools.
- Similarity: normalization + SQLite FTS5 + explicit references; no embeddings until measured.
- Jev runs only in Cloud (ADR 0005); the local `pnpm jev` tooling was removed.

## Thesis

> Polterware keeps the project state — what it is, what was decided, what is in progress, and what was delivered — identical for any agent and any person, checked against the repositories.

- **Continuity.** Switching from Codex to Claude (or to a teammate, once Cloud exists) needs no re-explaining: the next session starts from the recorded state.
- **Grounded SDD.** Spec-driven development where the spec is a living graph, not a throwaway document. Same shape at every scale, continuously verified against the repo.
- **Atlas** is the multi-scale project graph. **Oracle** answers zoom-aware queries on it (the Context Compiler evolved). Deterministic, no LLM on the path.
- **Orchestration by coordination, not command.** Polterware is the neutral shared board every agent reads and updates (claims, handoffs). It is not a dispatcher that launches agents; vendors will commoditize that.
- **AI asks, humans decide.** Agents raise questions; a person decides. Agents decide only what was explicitly delegated, and that choice is recorded as "decided by agent under delegation".
- **Determinism, honestly.** Not deterministic LLM output, but bounded output: fixed decisions and contracts, same-zoom context, explicit delegated space.
- **Cloud later.** Local first (one person, many agents). Cloud adds teams, decision owners, and central claims.

What separates this from a `PLAN.md` in the repo:

1. Separates what an agent *declared* from what is *observed* (Git on main). "Delivered" only when Git confirms.
2. Per-branch/worktree state, so parallel agents do not overwrite each other.
3. Captured by hooks, not by discipline.
4. One place for everything (and, with Cloud, for the team).

## Core model

### Node (same primitive at every scale)

| Field | Meaning |
| --- | --- |
| Intent | What we want (declared) |
| Contract | Invariants, acceptance criteria, allowed write scope |
| Decisions | What was decided, by whom, what is delegated |
| Reality | What exists: observed facts and Git/code evidence |
| Status | proposed, in progress, delivered (Git-confirmed), drift |
| Children | Lower nodes |

Levels: System → Domain → Component → Feature → Task. A small app uses two levels; a large system uses more. A parent contract constrains all children.

### Zoom

An agent working on a task gets the full task, the component summarized, the domain in one line, and inherited contracts. Token cost depends on zoom depth, not project size.

### Feature lifecycle

```text
 INTENT ──► SHAPE ──► CONTRACT ──► PLAN ──► BUILD ──► VERIFY ──► DELIVERED ──► LIVING
 human      human +   Polterware   agent     agents    Polterware  Git confirms  drift keeps
 writes     agent     checks       proposes, claim     runs        (main)        it true
            raise     conflicts    human     tasks,    acceptance
            questions with parent  approves  handoff   + checks
```

### Architecture evolution

1. Structural change is a proposal (an *Architecture Change* feature whose intent edits the Atlas).
2. Decision lifecycle: `proposed → active → challenged → superseded`; the new decision points at the old one; history is never deleted; a review trigger can move a decision to `challenged`.
3. Migration state on nodes (`migrating: A → B`), with the contract saying what applies where.
4. Timeline: the Atlas is versioned by the operation log (view it at a date, or at the commit an operation is linked to).
5. Change from below: drift shows reality diverged → Inbox asks "fix code or accept and update the Atlas?".

### Inbox rules

- Only three item types: **Decision** (raised by an agent, not decided or delegated), **Drift**, **Verify** (observed fact worth confirming). Nothing else.
- Every item says whether it **blocks**. Blocking: the task stops until decided. Non-blocking: the agent proceeds with a declared default and the item waits for confirmation.
- The same item appears in the Inbox, the sidebar count, `polterware status`, the Atlas node, the Work feature, and the session-start hook (so the next agent asks in chat and the answer closes the item).
- Cloud later adds an **owner** field; structure does not change.

## Done

- [x] **Slice 1 — Navigation swap** (uncommitted on `feat/atlas-grounded-sdd`). Sidebar: Inbox, Atlas, Work, Decisions, Reality, Agents, Integrations; Settings in the footer. Default route Atlas › Canvas. Legacy routes (`architecture`, `resources`, `knowledge/*`, `agents/facts`) redirect. Commander, `docs/spec.md`, and `AGENTS.md` updated. Files: `packages/features/shell/src/route.ts`, `sidebar.tsx`, `index.tsx`, `inbox-page.tsx`, `apps/desktop/src/renderer/App.tsx`, commander files.
- [x] **Slice 2 — Work › Board** (uncommitted). Five stage columns (Shape, Plan, Build, Verify, Delivered). `stage` lives in the spec artifact `extra_json`; derived when missing (draft → Shape, ready → Build, all tasks checked → Verify). Stage Select in the feature detail panel, audited via `justifyChange`. Files: `packages/features/knowledge/src/work-board.tsx`, `work-board.css`, `work-stage.ts`, `spec-feature-detail.tsx`, `apps/desktop/src/renderer/workspace/map-records.ts`, `persist.ts`.
- [x] Inbox page exists with an honest empty state only.
- [x] **Slice 3a — backend pass** (uncommitted). [ADR 0006](../adr/0006-polter-format-and-work-model.md) (Proposed). Work tables (`work_cards`, `work_tasks`, `work_questions`, `work_invariants`) and append-only `change_ops` in migration `0014_work_model.sql`; ULID ids (`backend/platform/ids`). Codec `backend/domains/polter` (decode, lint, plan, encode, format spec, query parse), store `providers/sqlite/work.go`, composition `wedge/work.go` (`ApplyPolter`, `QueryPolter`, `ResolveProject`). CLI `apply [--dry-run]`, `lint`, `format`, `query "cards" | "card <slug>"`. `apply` without `--agent-run-id` auto-opens an agent run labelled `cli`; the change step carries the `change_ops` batch id. Tests written in `domains/polter/polter_test.go` and `compositions/wedge/work_test.go`, not run.
- [x] **Slice 3a — Desktop pass** (uncommitted). `schemas/polterware/local/v1/work.proto` (`WorkService.ListWorkCards`, `ApplyPolter`); handlers in `compositions/local-core/work_handlers.go`; `wedge.ApplyInput` gained `ActorKind`/`Tool`. Desktop: `core.work.listCards` / `core.work.apply` through shared IPC, Main controllers, and preload. Renderer maps `workCards` (`map-records.ts`) and writes via `persistWorkCreateCard`, `persistWorkSetStage`, `persistWorkRemoveCard` (`persist.ts`). Knowledge package: `work-board.tsx`, `work-card-detail.tsx` (read-only intent/tasks/questions/invariants, stage Select, remove), `work-stage.ts`, `card-slug.ts`, card filters; Commander overview reads cards. Removed: `spec-feature-detail.tsx`, `spec-templates.ts` (Constitution template kept in `constitution-template.ts`), `feature-key.ts`, spec knowledge kinds and seed items. Test `work_handlers_test.go` written, not run.
- [x] **Slice 3b — Agent channels** (uncommitted). MCP `polter.format`, `polter.query`, `polter.apply` (`providers/mcp/polter.go`; server version 0.5.0). Session runs in `wedge/session.go` (`SessionRuns`, `ApplyInSession`): one agent run per project per session, opened on the first real write, never for invalid, no-op, or dry-run text. MCP keeps it per process (label = client name, completed on exit); the CLI keeps it per Claude Code session through `polterware hook session-start` (exports `POLTERWARE_SESSION` to `$CLAUDE_ENV_FILE`), `<db dir>/run/sessions/<id>.json`, and `hook session-end`. `hook lint` returns `.pw.md` errors to the agent (exit 2). Claude Code plugin in `plugins/polterware` (skill `polter` embedding `polter.Spec`, hooks) with marketplace `.claude-plugin/marketplace.json`. AGENTS.md Work cards line; Desktop AI skill teaches `polter.*`. Tests written in `providers/mcp/polter_test.go`, `wedge/session_test.go`, `domains/polter/skill_test.go`, not run.

- [x] **Slice 3c — Hints** (uncommitted). `domains/polter/hints.go`: `Lint` with `References.Cards` reports non-blocking `hint` diagnostics for new cards, questions, and invariants similar to active ones (normalized word sets, Jaccard), and suggests `under:` for new cards from existing `under:` values. `wedge.ApplyPolter` reuses its snapshot for references; CLI `lint`/`apply`, `hook lint` (exit 2), and MCP `polter.apply` print hints prefixed `hint:`; the Desktop already receives them as `PolterDiagnostic`. FTS5 index deferred to the first ranked-search consumer (see `polter-format.md`). Test `TestLintHints` written, not run.

- [x] **Slice 3d — Single local core + live sync** (uncommitted). [ADR 0007](../adr/0007-single-local-core.md). Only `polterware serve` opens SQLite (lock `run/core.lock`, idle exit, build handshake). Commit hooks → `platform/events` bus → `ChangeService.WatchChanges`; Desktop Main forwards to `window.polterware.core.onChange`, renderer refreshes affected slices (`renderer/workspace/live-refresh.ts`). MCP moved into the core (`/mcp`), `polterware mcp` is a bridge; CLI and TUI use `platform/coreclient`. New RPCs: `ResolveProject`, `InitProject`, `LintPolter`, `QueryPolter`, `EndSession`, `ListMcpSessions`, `Shutdown`. Removed: per-process MCP presence files, session state files, core `owner`. Not run end to end.

Validation so far: typecheck passed; `git diff --check` clean; `pnpm design:lint` fails only on files this work did not touch. Tests and browser not run.

Known leftovers: some UI copy still says "Architecture" (Atlas canvas tab label, toasts).

## Next slices

Recommended order: 3 (foundation every later slice reads and writes), then 3d (single core + live sync), then 4 (continuity test), then 5 and 6. Slices 3a–3c are done; 3d is implemented and awaits end-to-end validation; slice 4 is next.

Git is two local reads that do not share results. The checklist is [Git — verify at the end](#git--verify-at-the-end). Slice 5 has local Git evidence and the revision stamp; commit links and the new drift case are still open.

### Slice 3a — Work model + Polter codec + CLI

Goal: an agent creates a roadmap or Work cards with one `polterware apply` call, cheaply. Design: [`polter-format.md`](polter-format.md).

- [x] ADR (Proposed): [ADR 0006](../adr/0006-polter-format-and-work-model.md).
- [x] Work model schema in the SQLite baseline (migration `0014_work_model.sql`). Decided: the five Knowledge artifacts (Spec/Plan/Tasks/Clarify/Checklist) are discontinued, not attached or migrated.
- [x] Stable ids (ULID) for new records; every applied operation appended to `change_ops` with a batch id carried by the AgentRun change step.
- [x] Codec package: decode → lint → plan (diff vs store) → apply (one transaction); encode list and single card. Grammar v1, upsert by slug, explicit removal, round-trip property (tests written). Zoom/budget encoding moves to slice 8.
- [x] CLI: `apply [--dry-run]`, `lint` (syntax + `decisions:` references), `format`, `query`.
- [x] Compact list output (one line per card) and single-card encode.
- [x] **Desktop pass:** proto `WorkService` (`ListWorkCards`, `ApplyPolter`) → generated Go/TS; core handlers; Main/preload wiring (`window.polterware.core.work`); Work › Board and Specs render cards from the Work tables; stage change, create, and remove go through `ApplyPolter` as Polter text (human actor, tool `desktop.work.apply`). Five-document spec flow removed (feature detail, templates, `mapSpecFeatures`, `extra_json` stage, spec kinds in `knowledge.ValidKind`, seed spec items). Constitution kept.

### Slice 3b — Agent channels

- [x] MCP: `polter.apply`, `polter.query`, `polter.format` (existing tools stay until the ADR approves migration).
- [x] Claude Code plugin: skill with the format spec + `PostToolUse` hook running lint on `.pw.md` writes.
- [x] One line in `AGENTS.md` for Codex and others (`polterware format`, `polterware apply`).
- [x] Session-scoped AgentRun opened by hook; writes carry only `summary` + `rationale`, no start/complete calls.

### Slice 3c — Hints

- [x] Duplicate and placement hints via normalization (no embeddings), reported by `lint` and `apply`. FTS5 deferred to ranked prose search. See [`polter-format.md`](polter-format.md#similarity-and-matching-no-embeddings-first).

### Slice 3d — Single local core + live sync

Goal: every surface shows the same state without a manual reload. Decided in [ADR 0007](../adr/0007-single-local-core.md) (Proposed): one core process owns SQLite; everything else is a client.

- [x] ADR 0007; one core per database (`run/core.lock`), daemon with 10-minute idle exit, build handshake (`HealthService.Check` build, `MaintenanceService.Shutdown`).
- [x] Change events from SQLite commit hooks (`platform/db/sqlite/hooks.go`) → coalescing bus (`platform/events`) → `ChangeService.WatchChanges` with kinds, project ids, origins, resync.
- [x] Desktop Main attaches or starts the core detached, swaps clients on restart, and forwards events (`window.polterware.core.onChange`); the renderer reloads only affected slices and skips its own writes.
- [x] MCP runs in the core (`/mcp`, `providers/mcp/http.go`); `polterware mcp` is a stdio bridge.
- [x] CLI commands and the TUI are core clients (`platform/coreclient`); agent session runs live in the core (`WorkService.EndSession`).
- [ ] Validate end to end (Desktop open, `polterware apply` from a terminal, MCP from Claude Code) and measure start latency of the first CLI call.

### Slice 4 — Continuity loop (the two-week test)

Goal: switch between Codex and Claude on Polterware's own development without re-explaining context.

- [ ] Handoff record in Polter format (see slice 3), indexed by the core: feature, branch, goal, done, in progress, next, open questions (with blocking flag), touched files, agent, timestamp. Keyed per feature, linked to branch.
- [ ] `polterware handoff [--note "..."]` writes the record (short, structured; cross-checked against the real diff).
- [ ] `polterware status`: current node/feature, branch, active claims, open questions, next step, "delivered" only when Git confirms.
- [ ] Exposed through `polter.apply` / `polter.query` (no new MCP tool names).
- [ ] Claude Code hooks: `SessionStart` → `polterware status` (+ Oracle at task zoom when available); `Stop` → `polterware handoff`.
- [ ] One line in `AGENTS.md` for Codex and others: start with `polterware status`, end with `polterware handoff`.
- [ ] Measure for two weeks: how often context had to be re-explained; whether the agent started from the right point; whether Erick opened `status` himself.

Expected handoff shape:

```text
Feature: git-metadata-import (branch feat/git-import)
Goal: import HEAD, branch and recent commits as evidence
Done: commit evidence upsert in core; spec.md updated
In progress: stamping revision on observed facts
Next: wire into ImportProjectSources; Desktop shows revision
Open question: include merge commits? (blocking — ask Erick)
Touched: backend/.../import.go, docs/spec.md
```

### Slice 5 — Git metadata import

- [x] Import HEAD, branch, and recent commits as evidence (deterministic, no network). One HEAD row and up to 20 first-parent commits on `ImportProjectSources` / `polterware scan`, at the single project `root_path`. Not a work tree: skip, scan still succeeds.
- [x] Stamp revision on observed facts. Each file fact from that scan links to the HEAD evidence row (`ppm_fact_evidence`); no new column.
- [ ] Commit ↔ task links by explicit reference convention (`git-metadata-import T2`); FTS5 match only as a suggestion.
- [ ] "Delivered" = commit on main; new drift case: task marked done with no matching commit, or code delivered with no task.

### Git — verify at the end

Check this when the Atlas slices are far enough to exercise, or before calling slices 4 and 5 done. Both reads are local. Neither fetches, and neither looks at GitHub, Vercel, or AWS.

**A. Session** (`providers/git.Observe`, used by `polterware status` and `polterware handoff`):

- [ ] Inside a work tree, `polterware status` prints the current branch.
- [ ] Outside a work tree, status still runs. The branch prints as unknown. Handoff file hints are skipped.
- [ ] A card whose stage is `delivered` prints `delivered` only when `HEAD` is an ancestor of local `main` or `master`. On a feature branch it prints `verify`.
- [ ] Git does not move a card to `delivered`. The stored stage has to already be `delivered`.
- [ ] `handoff` compares `touched:` and `branch:` with the local diff and `HEAD`. A mismatch is a hint, not an error.
- [ ] Changed paths are `merge-base(HEAD, main|master)..HEAD`, plus unstaged, staged, and untracked files.

**B. Import** (`providers/git.ReadMetadata`, used by `polterware scan` / `ImportProjectSources`, one project `root_path`):

- [ ] Work tree with commits: the scan message includes `Git branch <name>.` Evidence gains one `source = git:HEAD` row (summary is the branch, path is the full SHA) and up to 20 `source = git:commit` rows (summary is the subject, path is the SHA, first-parent only).
- [ ] A second scan updates that same HEAD row when the SHA changes. Commit rows that fall out of the window of 20 stay in the database.
- [ ] Each file fact from that scan is linked to the HEAD evidence row. Reality can show that link. There is no revision column.
- [ ] Not a work tree, or a repository with no commits: the message includes `Git metadata skipped.` File import still succeeds. No commit rows are written.
- [ ] Detached `HEAD`: the HEAD row summary is `HEAD`.
- [ ] `polterware repo add --name api --path <other-checkout>` then `scan`: the message also includes `Git api branch <name>.` Reality shows a separate `git:api:HEAD` row. File facts stay linked to the project-root HEAD.
- [ ] `polterware status` still prints the branch of the project directory, not the extra checkout.
- [ ] Settings → General → Local repositories adds and removes the same rows.

**Not built. Do not look for these:**

- A commit subject does not link to a Work task (`slug T2` / `polter: slug#T2`).
- A task marked done does not require a commit, and a commit does not create drift.
- `delivered` in status is the display rule in A. It is not "this commit is on main and matches a task".
- Atlas nodes do not carry `repo:` yet. A project can register extra local checkouts; that is not the Slice 8 node model.
- FTS5 suggestions from commit text are not implemented.

Tests are written and have not been run: `backend/providers/git/git_test.go`, `backend/compositions/wedge/scan_git_test.go`, `backend/domains/polter/handoff_test.go`. From `backend`:

```sh
go test ./providers/git/ ./domains/polter/ ./compositions/wedge/ -count=1 -run 'TestReadMetadata|TestScanSkipsGit|TestScanStampsHead|TestDisplayStage'
```

### Slice 6 — Claims

- [ ] `polterware task claim <id>` reserves the task write scope; `task done <id>` (declared; Git confirms later); `task release <id>`.
- [ ] Two agents never claim overlapping scopes.
- [ ] Exposed through `polter.apply` (no new MCP tool names).
- [ ] Work › Board shows task rows with claim owner and scope (see screen below).

### Slice 7 — Inbox with real items

- [ ] Items derived from existing data: unchecked Clarify lines (Decision), `checkDrift` findings (Drift), `declared`/`observed` facts without verification (Verify).
- [ ] Blocking flag per item; actions: Decide / Delegate / Defer, Fix code / Update intent / Ignore, Mark verified / Reject.
- [ ] Sidebar count badge; items also surfaced in `polterware status` and `polterware inbox`.

### Slice 8 — Atlas hierarchy and Oracle

- [ ] Node model (Intent, Contract, Decisions, Reality, Status, Children) and levels System → Domain → Component → Feature → Task.
- [ ] Multi-repository projects: nodes carry `repo:` and `path:`; claim scopes include the repository (`api:src/auth/*`); a project has several local roots.
- [ ] Projections: ADRs exported to `docs/adr/` of the governing repository; generated `AGENTS.md` block per repository.
- [ ] Atlas canvas with zoom breadcrumb, node status legend, side panel per node.
- [ ] `polterware atlas [--node <id>] [--at <rev>]`.
- [ ] `polterware oracle "<question>" [--zoom task|component|domain|system] [--budget N]`; `context` becomes an alias. Exposed through `polter.query`.
- [ ] Agents › Oracle preview (what an agent sees at a zoom and token budget).

### Slice 9 — New-feature flow

- [ ] Stepper Intent → Shape → Contract → Plan, replacing "create five documents".
- [ ] Shape shows Oracle impact: touched nodes, active decisions, inherited contracts, open questions (to Inbox or delegated).
- [ ] CLI: `polterware feature new "<intent>" [--under <node>]`, `feature shape|contract|plan <id>`, `polterware verify <feature|task>`.

### Slice 10 — Decision lifecycle

- [ ] States proposed / active / challenged / superseded, supersede chain, review triggers, governed nodes.
- [ ] Decisions tabs by state; `polterware decision list|propose|supersede <id>`; exposed through `polter.apply`.

### Slice 11 — Architecture evolution

- [ ] Architecture Change features, node migration state, Atlas timeline at a date/commit.

### Slice 12 — Agents › Sessions

- [ ] Live sessions (agent, task, scope, duration, branch) and recent handoffs. TUI gains Work rows (active claims) and the Inbox count.

### Later

- [ ] Light enforcement via Claude Code `PreToolUse` ("this file is governed by ADR X", "adding a dependency requires a decision"). Advisory first.
- [ ] Jev as Cloud Judge (decision detector, invariant guard, handoff verification first): see [`../ideas/2026-09-28-jev-feature-catalog.md`](../ideas/2026-09-28-jev-feature-catalog.md). Never in Desktop or the local binary without a new ADR.
- [ ] Engineering packs that generate *questions* (Inbox items), never rules.
- [ ] Cloud (C0–C4): see [`cloud-architecture.md`](cloud-architecture.md).
- [ ] Research: BR variance metric and H3 (explicit decisions reduce incompatible changes); interviews and H4 token measurement.
- [ ] Measure FTS5 + normalization misses (duplicates, commit links) on real use; add local embeddings (`sqlite-vec`, small local model, opt-in download) only if misses are material.

### Explicitly not now

- Rigid state machine for agents (UNDERSTAND → PLAN → …): fights the harness.
- Polterware as an agent dispatcher/orchestrator.
- LLM or user LLM keys inside Desktop or the binary.
- Sidebar for skills/plugins/hooks as a product pillar (at most a setup "doctor").
- Reintroducing Ops/Terminal/Nodes.

## Target screens

Sidebar (shipped in slice 1):

```text
┌─────────────────────┐
│ Inbox           (4) │  everything that needs a human
│ Atlas               │  canvas with zoom + time
│ Work                │  features, tasks, claims
│ Decisions           │  ADRs with lifecycle
│ Reality             │  Inventory + Sources + Facts + Docs
│ Agents              │  sessions, handoffs, oracle
│ ─────────────────── │
│ Integrations        │
│ Settings            │
└─────────────────────┘
```

Atlas:

```text
┌─ Atlas ──────────────────────────────────────────────────────────────────┐
│ polterware › Core › Import                    [ System ▸ Domain ▸ Comp ] │
│ ◷ Now ▾   (timeline: v0.3 ─ v0.4 ─ ● now)                  ⚠ 1 drift     │
├──────────────────────────────────────────────┬───────────────────────────┤
│   ┌────────────┐      ┌─────────────┐        │ Import  (component)       │
│   │ Scan       │─────►│ Facts store │        │ Intent   Deterministic... │
│   │ ● active   │      │ ● active    │        │ Contract 3 invariants     │
│   └─────┬──────┘      └─────────────┘        │ Decisions ADR-0007 active │
│   ┌─────▼──────┐      ┌─────────────┐        │ Reality  12 facts · 4 ev. │
│   │ Git import │      │ Ctx files   │        │ Work     1 feature ◐      │
│   │ ◐ building │      │ ○ proposed  │        │ [Zoom in]  [New feature]  │
│   └────────────┘      └─────────────┘        │                           │
└──────────────────────────────────────────────┴───────────────────────────┘
  ● active  ◐ in progress  ○ proposed  ⇄ migrating  ⚠ drift
```

Work with claims (slice 6 target; cards come from slice 3a tables):

```text
┌─ Work ───────────────────────────────────────────────────────────────────┐
│ [All] [Mine] [Agents]   Filter: Core ▾                  [+ New feature]  │
├──────────────┬──────────────┬──────────────┬──────────────┬──────────────┤
│ SHAPE (2)    │ PLAN (1)     │ BUILD (3)    │ VERIFY (1)   │ DELIVERED    │
│ Ctx files    │ Verify-fact  │ Git import   │ Drift panel  │ CLI ref  ✓   │
│ ? 2 open q.  │ 4 tasks      │ ├ T1 ✓       │ accept 3/3   │ main a1b2c3  │
│              │ awaiting you │ ├ T2 ◐ codex │ waiting Git  │              │
│              │              │ │  import/*  │              │              │
│              │              │ └ T3 ○ free  │              │              │
└──────────────┴──────────────┴──────────────┴──────────────┴──────────────┘
```

New feature (slice 9):

```text
┌─ New feature ────────────────────────────────────────────────────────────┐
│ ① Intent ─ ② Shape ─ ③ Contract ─ ④ Plan                                 │
│ Intent: Import Git metadata (HEAD, branch, recent commits) as evidence   │
│ Lives under:  Core › Import  (suggested by Oracle)            [Change]   │
│ Oracle impact                                                            │
│  Touches     Scan, Facts store                                           │
│  Applies     ADR-0007 Deterministic import only  ● active                │
│  Inherits    "No network calls during scan" (Core contract)              │
│  Open        ? Include merge commits?          → Inbox                   │
│              ? How many commits back?          → delegated to agent      │
│                                      [Save as proposed]  [Next: Contract]│
└──────────────────────────────────────────────────────────────────────────┘
```

Inbox (slice 7):

```text
┌─ Inbox ──────────────────────────────────────────────────────────────────┐
│ [All 3] [Decisions 1] [Drift 1] [Verify 1]              ⛔ 1 blocking    │
├──────────────────────────────────────────────────────────────────────────┤
│ ⛔ DECISION  Include merge commits?            Git import › T2   codex   │
│    Options: yes · no · first-parent             [Decide] [Delegate]      │
│ ◦ DRIFT     AGENTS.md "npm test" vs pnpm        Ctx files        system  │
│    [Fix code] [Update intent] [Ignore]                                   │
│ ◦ VERIFY    "Local auth is bearer token"        Core › Auth      scan    │
│    [Mark verified] [Reject]                                              │
└──────────────────────────────────────────────────────────────────────────┘
```

Decisions (slice 10):

```text
┌─ Decisions ──────────────────────────────────────────────────────────────┐
│ [Active 12] [Challenged 1] [Proposed 2] [Superseded 5]                   │
├───────────────────────────────────────┬──────────────────────────────────┤
│ ● ADR-0007 Deterministic import only  │ ADR-0009 Storage engine          │
│ ⚡ ADR-0009 SQLite as local store      │ Status   ⚡ challenged            │
│ ● ADR-0011 Renderer via bridge only   │ Trigger  "multi-user sync"       │
│                                       │ Chain    ADR-0003 ─► ADR-0009    │
│                                       │ Governs  4 nodes · 2 contracts   │
│                                       │ [Open review] [Supersede…]       │
└───────────────────────────────────────┴──────────────────────────────────┘
```

Agents (slices 8 and 12):

```text
┌─ Agents ── Connections · Sessions · Oracle ──────────────────────────────┐
│ ● codex        Git import › T2   scope import/git/*     14m              │
│ ● claude-code  Drift panel › T4  scope features/drift/*  3m              │
│ ○ claude-code  ended 1h ago — handoff: "T1 done; next wire revision…"    │
│ ORACLE  Zoom: [Task ▾]  Node: Git import › T2   Budget: 1500 → 1,120 used│
└──────────────────────────────────────────────────────────────────────────┘
```

## Target CLI

```text
EXISTING (small changes)
  polterware init | scan | serve | mcp
  polterware context  → alias of `oracle`
  polterware drift    → feeds the Inbox

NEW
  polterware apply <file|-> [--dry-run]  Polter text → plan → store
  polterware lint <file|dir|->           validate Polter text
  polterware format                      print the format spec
  polterware login                       link to Cloud (device flow, later)
  polterware status                      node, branch, claims, open questions, next step
  polterware handoff [--note "..."]      write session state (used by the Stop hook)
  polterware oracle "<q>" [--zoom task|component|domain|system] [--budget N]
  polterware atlas [--node <id>] [--at <rev>]
  polterware feature new "<intent>" [--under <node>]
  polterware feature shape|contract|plan <id>
  polterware task claim|done|release <id>
  polterware verify <feature|task>
  polterware decision list|propose|supersede <id>
  polterware inbox

Agent loop
  SessionStart hook → polterware status (+ oracle --zoom task)
  agent works       → task claim → edit → task done
  Stop hook         → polterware handoff
```

## Pending discussion

- ~~Embeddings/RAG/vector search~~ — decided: normalization + FTS5 + explicit references first; embeddings only if measurement shows material misses (see Later).
- ~~Jev~~ — decided: Cloud-only Judge; catalog in [`../ideas/2026-09-28-jev-feature-catalog.md`](../ideas/2026-09-28-jev-feature-catalog.md). Open: which P1 check headlines the Guard plan.
- Cloud sync details: operation granularity, long-offline replicas, log compaction. Architecture draft: [`cloud-architecture.md`](cloud-architecture.md).

## Open questions

1. ~~Where does the graph live?~~ Decided: Polterware store is the truth (SQLite locally, Cloud for teams); repositories provide evidence and receive projections. See [`polter-format.md`](polter-format.md#source-of-truth-and-storage). Needs an ADR.
2. Who creates the upper levels (System, Domain)? By hand, imported from code, or agent-suggested and human-approved?
3. Does Polterware own tasks, or read them from GitHub Issues/Linear and link them to code and decisions?
4. Is work state per feature, per branch, or per session? (Leaning: per feature, with branches linked.)
5. For a teammate on Cloud, does "delivered" mean on main or in production?
6. Should the default route flip from Atlas to Work?
