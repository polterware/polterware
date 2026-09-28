# Cloud architecture

Status: Proposed (2026-09-28 discussion). Nothing here is implemented.

Authority: none yet. Part of [`atlas-grounded-sdd-roadmap.md`](atlas-grounded-sdd-roadmap.md). Builds on [ADR 0001](../adr/0001-open-source-cloud-boundary.md), [ADR 0002](../adr/0002-single-desktop-and-cloud-connection.md), [ADR 0005](../adr/0005-post-wedge-cloud-and-enrichment-boundary.md), [`web-repository-boundary.md`](web-repository-boundary.md), and [`polter-format.md`](polter-format.md). Needs an ADR before implementation. Pricing is not decided.

## Principles

1. **Polterware is the source of truth, locally and in Cloud.** The same project model and change log live in local SQLite and, for team projects, in Cloud. Git repositories are evidence in and projections out, never the truth.
2. **Local-first.** The local `polterware` core works offline and without an account. Cloud adds team continuity; it never unlocks local features.
3. **One project, many repositories and services.** The Atlas sits above repositories; Cloud ingests all of them through one GitHub App installation per organization.
4. **No source code stored.** Cloud keeps the project model, commit metadata, and judgments. Diffs pass through only transiently for opt-in Jev checks.
5. **One authority per concern.** Cloud authorizes every hosted operation; the Web App and Desktop render server-provided capabilities.
6. **Same code on both sides.** Domain logic and the Polter codec are one Go module used by the local binary and the Cloud service.

## Components

```text
  Claude Code · Codex · Cursor              Codex cloud · Claude web · ChatGPT
        │ CLI + plugin, local MCP (stdio)          │ remote MCP (HTTP + OAuth)
        ▼                                          │
  polterware (local core, SQLite) ── ops sync ──►┌─▼──────────────────────────────┐
  Desktop ─────────────────────────────────────► │ polterware-cloud (Go, Fly.io)  │
  Web App (polterware-web) ─────── API/SSE ─────►│ ├─ Sync: order + fan out ops   │
                                                 │ ├─ Coordinator: claims, sessions│
  GitHub ───── webhooks (GitHub App) ───────────►│ ├─ Ingest: commits, PRs, files │
         ◄──── PRs, Checks, comments ────────────│ ├─ Projection: ADRs, AGENTS.md │
                                                 │ ├─ Judge: Jev workers (queue)  │
                                                 │ ├─ Inbox: owners, notifications│
                                                 │ └─ Remote MCP + HTTP API       │
                                                 └───────────────┬────────────────┘
                                                                 ▼
                                                  Supabase: Postgres + Auth
```

| Component | Responsibility |
| --- | --- |
| Sync | Accept operations from replicas, assign order, persist to the log, fan out to other replicas |
| Coordinator | Claims with leases, live sessions, handoffs; the only arbiter for scope conflicts |
| Ingest | GitHub App webhooks: commits, branches, PRs, and relevant files as evidence; links commits to tasks |
| Projection | Writes ADRs (`docs/adr/`) and a generated `AGENTS.md` block into each repository through PRs |
| Judge | TypeSafe Jev checks as background jobs (see [Judge](#judge-jev)) |
| Inbox | Decision, Drift, and Verify items with owners, blocking flag, notifications |
| Remote MCP + API | Same Polter tools as the local MCP; HTTP API for Web App and Desktop |

## Code layout

```text
 polterware/polterware (public)             polterware/cloud (private)
 ┌─────────────────────────────────┐        ┌──────────────────────────────┐
 │ Go module: domains + Polter codec│◄import─│ polterware-cloud binary      │
 ├────────────────┬────────────────┤        │ Sync, Coordinator, Ingest,   │
 │ SQLite adapter │ `polterware`    │        │ Projection, Judge, MCP, API  │
 │                │ binary          │        │ Postgres adapter             │
 └────────────────┴────────────────┘        └──────────────────────────────┘
```

- Domain logic and the codec live in the public repository as a Go module. Cloud imports it (ADR 0001 boundary).
- Only the storage adapter differs: SQLite locally, Postgres in Cloud, behind the same interface.
- The local binary gains `polterware login` (device flow), a background sync client, and claim forwarding for team projects.
- The Web App frontend lives in `polterware/polterware-web` and talks only to the Cloud API.

## Data and sync

### Storage

- Relational, normalized tables for nodes, edges, tasks, decisions, questions, claims, facts, evidence, sessions, handoffs, and Inbox items.
- An **append-only operation log**: every mutation with actor, `summary`, `rationale`, timestamp, and the affected ids. It is the audit trail (Unified Change History) and the sync contract.
- **Ids generated where the change happens** (ULID), so offline creation never collides.

### Replication

```text
  Erick's machine                  Cloud                     Ana's machine
  ┌──────────────┐     ops     ┌───────────────┐    ops    ┌──────────────┐
  │ SQLite       │ ──────────► │ Sync: orders  │ ◄──────── │ SQLite       │
  │ (replica)    │ ◄────────── │ and fans out  │ ────────► │ (replica)    │
  │ offline queue│ others' ops │ Postgres      │           │ offline queue│
  └──────────────┘             │ (authority)   │           └──────────────┘
                               └───────────────┘
```

- **Solo, no Cloud:** local SQLite is the authority (today's behavior).
- **Linking a project to Cloud:** the local log is uploaded; Cloud becomes that project's authority; machines become replicas that keep working offline and push queued operations on reconnect.
- **The log is synced, not the tables.** SQLite and Postgres schemas may differ in details; operations are the contract (defined in `schemas`, generated to Go and TypeScript).
- **Conflicts:**
  - Field-level last-writer-wins for independent fields (hybrid logical clock timestamps).
  - Semantic conflicts (two people changing the same decision or contract) become Inbox items; nothing is silently overwritten.
- **Claims require connectivity.** Offline claims are tentative and confirmed or rejected on reconnect.

Open: operation granularity (per field vs per entity), long-offline replicas, log compaction and snapshots.

### Cloud tables (Postgres)

| Table | Content |
| --- | --- |
| `orgs`, `members`, `projects`, `repos` | A project spans many repositories and services |
| `ops` | The ordered operation log per project |
| `nodes`, `tasks`, `decisions`, `questions`, `facts`, `evidence` | Materialized model (same shape as local) |
| `claims` | Task, scope (`repo:path/*`), holder, lease expiry |
| `sessions`, `handoffs` | Agent, machine, branch, state |
| `commits`, `links` | Commit ↔ task links with method (`explicit`, `fts`, `jev`) and confidence |
| `inbox_items` | Kind, owner, blocking, state, origin |
| `ownership` | Who decides what: derived from `CODEOWNERS` plus node `owner:` |
| `judgments` | Question version, model, probabilities, threshold, outcome, human accept/reject |

`judgments` with human accept/reject is a labeled dataset for per-team threshold calibration; thresholds change without re-running inference.

## Main flows

### A. Push to main (Delivered)

1. The GitHub App webhook arrives; Ingest fetches commit metadata (and diff only when Judge is enabled) through the GitHub API, without cloning.
2. Commits link to tasks: explicit reference (`git-metadata-import T2`) first, then FTS match, then Jev as a suggestion.
3. Linked tasks become Delivered (observed, not declared).
4. Judge runs the decision detector on the diff; findings become Inbox items.

### B. Pull request (Check)

- A GitHub Check reports, for example, "1 unrecorded decision (new dependency: Redis)".
- Advisory by default. A team may make it a required check: this is the Change Gate from ADR 0005, living where the team already works.

### C. Agent session (team project)

1. The session-start hook runs `polterware status`; the local core returns team-wide claims, open questions, and the last handoff (synced from Cloud).
2. `task claim` goes to the Coordinator: a lease with expiry and heartbeat, released automatically if the agent dies.
3. The session-end hook writes the handoff; Judge checks it against the diff.

### D. Decision from anywhere

1. A person decides in the Web App, the Desktop, or by answering an agent in chat.
2. The decision is an operation in the log (actor, date, rationale).
3. Projection opens a PR that updates the ADR and the `AGENTS.md` block in the repository the decision governs.

### E. Offline

The local core keeps working; claims become tentative. On reconnect, the queue is pushed, others' operations are pulled, and conflicts become Inbox items.

## MCP: local and remote

| | Local MCP | Remote MCP |
| --- | --- | --- |
| Runs as | `polterware mcp` over stdio (as today) | HTTP at a Cloud endpoint, OAuth |
| For | Claude Code, Codex, Cursor on a machine | Agents that cannot run a local binary (Codex cloud, Claude web, ChatGPT) |
| Reads and writes | Local SQLite, synced in the background | Cloud Postgres directly |
| Offline | Yes | No |

Both expose the same Polter tools (`polter.apply`, `polter.query`, `polter.format`) through the same codec. Rule: prefer local when a core exists (faster, offline); remote is for sandboxed agents.

## Identity and access

- **People:** GitHub login through Supabase Auth. Polterware organizations mirror GitHub organizations.
- **Go validates every request:** Supabase JWT verified with the project's public keys (JWKS); user id from `sub`.
- **Local core:** `polterware login` device flow. The CLI shows a code, the person signs in through the browser (Supabase), and Go issues its own token scoped to that machine and project.
- **Remote MCP:** OAuth with Go as the authorization server, delegating sign-in to Supabase.
- **Agents:** scoped tokens per project; every write still carries the change envelope.
- The Web App never authorizes; Cloud is the only authority ([`web-repository-boundary.md`](web-repository-boundary.md)).

## Hosting

**Go service on Fly.io, in the same region as the Supabase project.** Every request talks to the database, so co-location matters more than the provider.

| Option | Pros | Cons |
| --- | --- | --- |
| **Fly.io (chosen)** | Simple Go container, long-lived connections (SSE/websocket for live claims), São Paulo region, low starting cost | Fewer managed services around it |
| Google Cloud Run | Scales to zero, easy credits | Request timeouts hurt long-lived realtime connections |
| Railway / Render | Easiest deploy | Less control over region and networking |
| AWS ECS | Enterprise standard | Too heavy for the current team size |

## Supabase with Go

Use:

- **Postgres** through `pgx` (`pgxpool`), queries generated with `sqlc`, migrations owned by the Go service. Use the direct connection or the pooler in session mode; transaction mode breaks `pgx` prepared statements.
- **Auth** for GitHub sign-in; Go verifies JWTs.
- **River** (Go job queue on Postgres) for Ingest, Judge, and Projection jobs. No Redis.
- **Storage** only if exports or backups need it.
- **RLS** enabled as a second safety layer, not the primary authorization.

Do not use:

- **Edge Functions** (Deno): logic stays in Go.
- **`supabase-js` from the Web App to the database:** the Web App calls the Go API only.
- **Supabase Realtime:** realtime is SSE from the Go service, which already arbitrates claims.

Leaving Supabase later means changing the connection string and the auth provider, not rewriting logic.

## Judge (Jev)

Jev returns typed judgments with probabilities (Choice, Noul, Score). It is cheap enough to run on every push, handoff, and apply. Pattern: code finds candidates (FTS, Git), Jev selects, a threshold decides between applying and sending to the Inbox. Jev output stays `inferred` until a person or gate confirms it. Keys live only in the Cloud service (ADR 0005).

Candidate checks, strongest first:

1. **Decision detector** on every push/PR: does the diff contain an unrecorded decision? which kind (dependency, persistence, API, pattern)? one yes/no per invariant of the affected node: violated?
2. **Handoff verification:** does the diff support what the agent said it did?
3. **Commit ↔ task linking** when no explicit reference exists: FTS candidates, Jev picks one or none.
4. **Duplicate or contradiction** when a decision or question is created.
5. **Automatic challenge:** does a new fact meet an ADR's review trigger? If yes, the ADR becomes `challenged`.
6. **Inbox routing:** owner, blocking, priority.
7. **Oracle packing:** which blocks fit the token budget.

Sending diffs to TypeSafe requires per-repository opt-in and a clear retention policy. Full catalog: [`../ideas/2026-09-28-jev-feature-catalog.md`](../ideas/2026-09-28-jev-feature-catalog.md).

## Offer (draft)

| Plan | Contents |
| --- | --- |
| Local (free) | Everything local: Polter format, `status`, `handoff`, local claims, Desktop |
| Team | Shared model and board, claims across people, Inbox with owners, delivery feed ("what reached main this week and why"), remote MCP |
| Guard | Jev checks on every PR (unrecorded decision, verified handoff, invariant contradiction), required check, per-team calibration |

## Build order

1. **C0 — Read-only continuity.** Login, link a local project (one-way log upload), GitHub App Ingest, Web App delivery feed. Answers "what reached main, in which card" for a teammate.
2. **C1 — Coordination.** Two-way sync, claims with leases, sessions, handoffs, remote MCP.
3. **C2 — Decisions.** Inbox with owners (`CODEOWNERS`), Projection PRs (ADRs, `AGENTS.md` blocks).
4. **C3 — Judge.** Decision detector and handoff verification as GitHub Checks.
5. **C4 — Gates and billing.** Required checks, calibration, plans.

## Risks

- **GitHub lock-in:** GitLab is already in the integration catalog; Ingest needs an adapter boundary from the start, even if only GitHub ships first.
- **Diffs to a third party (TypeSafe):** opt-in per repository, retention policy.
- **Webhook loss:** replay and backfill from the last known SHA per repository.
- **Sync complexity:** the hardest part of the system; start one-way (C0) and keep operations small and explicit.
- **ADR 0005 wording:** "project sync and managed backup" still holds, now defined as operation-log replication; a new ADR should record this model.
