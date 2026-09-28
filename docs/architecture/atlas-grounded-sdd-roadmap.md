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

Validation so far: typecheck passed; `git diff --check` clean; `pnpm design:lint` fails only on files this work did not touch. Tests and browser not run.

Known leftovers: some UI copy still says "Architecture" (Atlas canvas tab label, toasts).

## Next slices

Recommended order: 3 (foundation every later slice reads and writes), then 4 (continuity test), then 5 and 6. Slices 3a–3c are sequential; 3a is the next unit of work.

### Slice 3a — Work model + Polter codec + CLI

Goal: an agent creates a roadmap or Work cards with one `polterware apply` call, cheaply. Design: [`polter-format.md`](polter-format.md).

- [ ] ADR (Proposed): Polter format as the agent ↔ Polterware protocol; Polterware store is the source of truth; repositories get evidence in and projections out.
- [ ] Work model decision and schema: typed tables for cards (features), tasks (with scope), questions (with blocking flag and options), and invariants, in `schemas` (proto) + SQLite baseline. Recommended: the existing five Knowledge artifacts (Spec/Plan/Tasks/Clarify/Checklist) stay as documents attached to a card; the Board reads the new tables.
- [ ] Stable ids (ULID) for new records; every mutation appended to the change log with the envelope (extends Unified Change History; this log becomes the Cloud sync contract).
- [ ] Codec package in the Go core: decode → plan (diff vs store) → validate → apply (one transaction), and encode at zoom/budget. Grammar v1 (cards, metadata, tasks with scope, `?` questions, `!` invariants, `polter: 1`). Upsert by slug; removal only explicit. Round-trip property.
- [ ] CLI: `polterware apply <file|-> [--dry-run]`, `polterware lint <file|dir|->` (syntax + reference levels, silent on success, one-line errors with the fix), `polterware format` (~150-token spec).
- [ ] Compact list output (one line per card) and single-card encode.
- [ ] Work › Board renders cards from the new tables.

### Slice 3b — Agent channels

- [ ] MCP: `polter.apply`, `polter.query`, `polter.format` (existing tools stay until the ADR approves migration).
- [ ] Claude Code plugin: skill with the format spec + `PostToolUse` hook running lint on `.pw.md` writes.
- [ ] One line in `AGENTS.md` for Codex and others (`polterware format`, `polterware apply`).
- [ ] Session-scoped AgentRun opened by hook; writes carry only `summary` + `rationale`, no start/complete calls.

### Slice 3c — Hints

- [ ] Duplicate and placement hints via normalization + SQLite FTS5 (no embeddings), reported by `lint` and `apply`. See [`polter-format.md`](polter-format.md#similarity-and-matching-no-embeddings-first).

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

- [ ] Import HEAD, branch, and recent commits as evidence (deterministic, no network).
- [ ] Stamp revision on observed facts.
- [ ] Commit ↔ task links by explicit reference convention (`git-metadata-import T2`); FTS5 match only as a suggestion.
- [ ] "Delivered" = commit on main; new drift case: task marked done with no matching commit, or code delivered with no task.

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
