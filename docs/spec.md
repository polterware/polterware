# Polterware — First Version Specification

Status: Active  
Scope: Local SQLite-backed Desktop with Go core execution authority

## Current scope versus product direction

This document is the source of truth for behavior implemented in the current product. [`product-vision.md`](product-vision.md) (v3) defines product direction: **Project Context & Change Intelligence for AI Coding**, the wedge (PPM facts/evidence, Context Compiler, MCP, Drift, AgentRun, CLI). [ADR 0004](adr/0004-product-positioning-and-wedge.md) records the positioning decision. Direction capabilities are not part of this specification unless they are explicitly added here by a later accepted change.

Operations, Terminal-as-product, and Resources Nodes / process control were removed on the kill branch ([`architecture/kill-ops-terminal-nodes.md`](architecture/kill-ops-terminal-nodes.md)). Integrations Connect (evidence path + intermediate UI) is kept; residual local PTY exists only for Connect auth.

Historical specifications under `docs/future/` describe earlier states or superseded plans and are not evidence of current behavior.

## Objective

Deliver a maintainable first version focused on understanding and documenting local software projects for AI coding. The Desktop reads and writes through the local core (`polterware serve`) and may run CLI-backed integration probes (including Connect auth PTY) on the host machine.

## Deployment boundary

Only the `local` deployment mode is active in this first version. Self-hosted and Cloud functionality is not implemented today. The public deployment contract and capability boundary are defined in [deployment-model.md](architecture/deployment-model.md); repository ownership is defined in [repository-map.md](architecture/repository-map.md) and [ADR 0001](adr/0001-open-source-cloud-boundary.md); the single-Desktop distribution and optional Cloud connection are defined in [ADR 0002](adr/0002-single-desktop-and-cloud-connection.md).

## Information architecture

Atlas / Canvas is the initial route and the destination after entering or changing a project.

| Surface | Tabs | Content |
| --- | --- | --- |
| Inbox | — (flat page) | Empty state only; no items are routed here yet |
| Atlas | Canvas, Views | Architecture canvas and saved views |
| Work | Board, Specs | Feature board by stage; Specs list with the project Constitution |
| Decisions | — (flat ADR list) | Library restricted to `adr` |
| Reality | Inventory, Sources, Facts, Docs | Resource inventory, registered sources, PPM facts/evidence, `doc` and `runbook` items |
| Agents | Access, History, Context | MCP access, Unified Change History, compile + narrow drift preview |
| Integrations | Connected, Catalog | **KEEP** as evidence + Connect UI |
| Settings | General, Environments, Shortcuts, Advanced | — |

Routes from the previous navigation redirect to their new homes: Architecture → Atlas, Resources → Reality › Inventory, Knowledge Specs → Work › Specs, Knowledge Library ADRs → Decisions, Knowledge docs/runbooks → Reality › Docs, Knowledge Sources → Reality › Sources, and Agents Facts → Reality › Facts.

Settings Advanced includes an app-level Appearance control: Reduce transparency switches the macOS title bar and sidebar from frosted vibrancy to a solid fill. The preference is stored in the Desktop userData directory and does not live in SQLite.

Entity detail opens in one shared detail panel with collapsed, open, and maximized modes. Collections dock it at the bottom (Architecture Canvas included). Operational records show overview content beside a properties rail; Knowledge documents show inline properties and linked-entity chips above the Markdown editor. Destructive commands such as Archive and Delete live in the header overflow menu.

The sidebar contains the Inbox, Atlas, Work, Decisions, Reality, Agents, and Integrations surfaces. Settings is fixed in its footer. The title bar retains local project and environment switchers, Commander, history controls, and a Help menu. The Help menu opens short topic dialogs and a static CLI reference page (`polterware` commands and flags); that page is a route outside the sidebar, not a product surface. In-app Commander opens as the shared command palette dialog with the same opaque float surface as other product dialogs. The global Commander shortcut opens a separate frameless, draggable overlay (`?surface=commander`) with the same opaque panel material, search at the top, an Open Desktop control, and an overview strip (Knowledge, Integrations) that drills into in-overlay lists — not a miniature copy of the full Desktop shell. The in-app palette remains search-only.

## Local project model

The application supports several local projects. A project has an identifier, name, directory, environments, and an active/archived status. Project creation asks for name, a directory chosen through the native folder dialog, and environments. Mutations persist in the local SQLite store through the local core.

## Minimal Polterware Project Model (PPM)

The core maintains a thin entity registry (`ppm_entities`) that projects architecture canvas nodes, inventory resources, and environments into typed refs with provenance (`truth_class`, `source`, `actor`, `observed_at`). Typed relationships (`ppm_relationships`) support predicates `DOCUMENTS`, `OPERATES`, `RELATED_TO`, and `DEFINES`. The canvas `architecture_layouts.body_json` remains the visual graph SSOT; the registry does not duplicate canvas geometry. Sync runs when layouts, resources, or environments are saved and via `SyncProjectModel`. Architecture Canvas persists layout edits with debounced autosave and an explicit save shortcut (Cmd/Ctrl+S) that flushes the current layout and shows a short success toast.

The core also stores PPM **facts** (`ppm_facts`) and **evidence** (`ppm_evidence`) linked by `ppm_fact_evidence` (evidence supports a fact). Fact truth classes are `declared`, `observed`, `inferred`, and `verified`. A `verified` fact requires a non-`system` actor. Entity registry rows may still use `operational` as a truth class; facts do not. Evidence kinds are `file`, `observation`, `test`, `commit`, and `other`. RPCs cover list/get/upsert for facts and evidence plus `LinkEvidenceToFact` and `ImportProjectSources`. `CompileContext` includes matching facts with linked evidence refs and machine `reasons` on every packaged item.

`ImportProjectSources` discovers a fixed MVP set under the project's `root_path` (package manifests, agent/README markdown, Docker/Compose, OpenAPI/Swagger, and shallow Terraform `*.tf`), upserts observed facts and file evidence with stable import IDs, and registers matching Knowledge `source` rows. Generic recursive filesystem indexing is not shipped. Details: [`docs/architecture/deterministic-import.md`](architecture/deterministic-import.md).

## Work cards and Specs

Work features are cards in the Work model (`work_cards`, `work_tasks`, `work_questions`, `work_invariants`; [ADR 0006](adr/0006-polter-format-and-work-model.md)). Each card has a unique slug, title, intent, stage, optional owner, `under`, and `decisions`, plus numbered tasks (`open`, `done`, `dropped`), questions (blocking flag, options, `open` or `answered`), and invariants. Agents write cards with Polter format text through `polterware apply`; the five-document Spec / Plan / Tasks / Clarify / Checklist model is discontinued and not migrated.

The Desktop reads cards through `WorkService.ListWorkCards` (active cards only) and writes through `WorkService.ApplyPolter`, which runs the same codec as the CLI as one human change with a change envelope. Work › Board groups cards into five stage columns: Shape, Plan, Build, Verify, and Delivered. Board cards show task progress (dropped tasks excluded) and the count of open questions. The card detail panel shows intent, tasks, questions, and invariants read-only, changes the stage, and removes the card from the board (`status: removed`; the row is kept). New cards start in Shape with a slug derived from the title. Tasks, questions, and invariants are edited through `polterware apply`, not in the Desktop.

Work › Specs shows the project Constitution (`kind = constitution`) above a table of the same cards. Creating the Constitution seeds an English principles template titled "Project constitution". Archiving removes the singleton so Specs returns to the empty constitution CTA. Specs do not run Spec Kit CLI and do not bind to `.specify/` on disk.

## Knowledge Library, Sources, links, and search

Knowledge Library lists Documents, ADRs, and Runbooks (`kind` = `doc`, `adr`, `runbook`) in one table. Kind, status, severity, and linked-entity filters live in the shared Filter popover with removable active chips under the toolbar; list search is a separate toolbar field. Detail panels remain kind-specific; runbooks keep severity metadata (Run-as-operation is retired with Operations). Items may link to PPM entities through `knowledge_entity_links` with the same predicates as relationships. Architecture node detail shows knowledge linked to the matching component entity.

Knowledge Sources lists registered project paths (`kind = source`) that ground the model — manifests, contracts, and repo files. Create adds a title and a path relative to the project root (`extra_json.path`). Knowledge → Sources also offers **Import from project root**, which calls `ImportProjectSources` for the MVP file set and upserts Source rows for discovered paths. Generic automatic filesystem indexing beyond that MVP set is not shipped.

Knowledge kinds are `doc`, `adr`, `runbook`, `source`, and `constitution`. Library does not introduce a new storage kind beyond the existing document kinds.

`SearchKnowledge` provides exact id match and SQLite FTS5 full-text search over title, body, summary, and tags within a project. Results carry basic provenance fields (`truth_class`, `source`). Commander knowledge hits for `doc`, `adr`, and `runbook` open the Library tab.

## Create templates

When the Work cards, Library, or Sources list is empty, the Desktop shows local inspiration template cards. The matching create dialog also offers those templates. Choosing a template creates the item immediately with a suggested title and, for Library docs, ADRs, and runbooks, a suggested body or runbook `extra_json`; for Sources, a suggested path in `extra_json`. Work card templates only supply a title; the card is created in Shape. Blank create remains available from the toolbar and create dialog form.

When a Resources Inventory list is empty, the Desktop shows local inspiration template cards. The matching dialog also offers those templates. Choosing a template creates the resource immediately with suggested fields. Blank create remains available from the toolbar and dialog form.

When Architecture Canvas is empty, the Desktop shows featured component templates (API, Service, PostgreSQL, Redis, Worker). Choosing one places the component immediately. Architecture Views empty shows Overview, Applications, and Data plane templates that create a saved view immediately. The existing Architecture catalog dialog remains the full blank/browse path.

When Settings Environments is empty, the Desktop shows Development, Staging, and Production templates that create the environment immediately. The create dialog also offers those templates.

Integrations do not use content templates; they use the fixed provider catalog.

## Context Compiler and stable MCP reads

`CompileContext` assembles a bounded context package from entity resolution, knowledge links, one-hop relationships, PPM facts with linked evidence, and full-text hits under a token budget. Items include `kind`, `id`, `title`, `excerpt`, `truth_class`, `source`, `path`, machine `reasons`, and `evidence` refs. There is no embedding model, GraphRAG, or fixed PCTX serialization. See [`architecture/context-compiler.md`](architecture/context-compiler.md).

`polterware mcp` serves a stable MCP stdio contract backed by the local core (a stdio ↔ HTTP bridge to the core's `/mcp` endpoint; see Local core process): Polter tools `polter.format` (the format spec), `polter.query` (`cards`, `card <slug>`), and `polter.apply` (Polter text, optional `dry_run`, `summary`, `rationale`, `agent_run_id`); reads `context.get`, `entity.get`, `evidence.get`, and `drift.check`; AgentRun audit tools `agent_run.start`, `agent_run.append`, `agent_run.complete`, `agent_run.get`, and `agent_run.list`; and audited write tools `fact.upsert`, `evidence.upsert`, `evidence.link`, `knowledge.upsert`, and `knowledge.link`. Write tools require `summary`, `rationale`, and an open `agent_run_id`; they persist PPM/Knowledge mutations and append attributable change steps. `polter.apply` runs the same codec as `polterware apply`; without `agent_run_id` it joins one agent run per project for the MCP session, labelled with the client name, opened on the first real write (never for invalid, no-op, or dry-run text) and completed when the bridge exits, the session is idle for 30 minutes, or the core stops. `context.get` returns the Context Compiler package. `drift.check` runs the Narrow Drift Checker (`checker: "narrow"`) and may return findings for import dependency diffs, edges under accepted ADRs without evidence, and declared-versus-observed fact conflicts. Optional `agent_run_id` on `context.get` / `drift.check` appends steps to an open AgentRun. See [`architecture/stable-mcp-reads.md`](architecture/stable-mcp-reads.md), [`architecture/narrow-drift-checker.md`](architecture/narrow-drift-checker.md), [`architecture/minimal-agent-run-audit.md`](architecture/minimal-agent-run-audit.md), and [`architecture/audited-mcp-writes.md`](architecture/audited-mcp-writes.md).

Agents is the Desktop operator surface for this access: start/stop a parked local MCP helper, copy Cursor MCP config, copy or download the Polterware AI skill (markdown that teaches agents how to use those tools), preview `CompileContext` for the current project (optional query and entity id), preview the Narrow Drift Checker via Connect `CheckDrift` (optional entity id; inspection only, no `agent_run_id`), inspect and justify PPM facts and evidence (`ListFacts` / `UpsertFact` / `UpsertEvidence` / `LinkEvidenceToFact` with change envelope), and Unified Change History — a feed of justified `human` and `agent` project-model changes (summary, rationale, audit chain) for the current project. Desktop Knowledge and PPM semantic mutations require the same change envelope (`summary`, `rationale`); the core records them on human AgentRuns. See [`architecture/unified-change-history.md`](architecture/unified-change-history.md).

## Local CLI

The `polterware` binary (`pnpm build:cli` → `dist/polterware`) is the only Go binary. Its headless commands are clients of the local core for the database given by `-db` (default: the Desktop database path) and start the core when none is running:

- `init` — register a project and a default `local` environment
- `scan` — deterministic MVP import (`ImportProjectSources`)
- `context` — Context Compiler package (optional `--query`, `--entity`, `--budget`, `--agent-run-id`, `--json`)
- `drift` — Narrow Drift Checker (optional `--entity`, `--agent-run-id`, `--json`)
- `apply` — write Polter text (Work cards, tasks, questions, invariants) as one audited change (`--dry-run` prints the plan)
- `lint` — validate Polter text (syntax, plus `decisions:` ADR references and similarity hints when the project resolves); silent on success
- `format` — print the Polter format spec
- `query` — read Work state in Polter format (`cards`, `card <slug>`)
- `hook` — Claude Code hook entry points (`session-start`, `session-end`, `lint`; JSON on stdin)
- `serve` — the local core (Connect + bearer session token, `/mcp`), the only process that opens the store
- `mcp` — MCP stdio bridge to the core for coding agents

`apply`, `lint`, and `query` resolve the project from `--project` or from the registered project whose root contains the current directory. Polter text (grammar v1, frontmatter `polter: 1`) is decoded by a line-oriented parser, diffed against the stored Work model, and written in one SQLite transaction. Cards are upserted by slug; tasks keep per-card numbers (`T1`, `T2`, …) that are never reused; questions and invariants match by normalized text. A text only changes what it mentions, and removal is explicit (`status: removed`, `- [-]`, trailing `[removed]`); rows are never deleted. Each apply needs `summary` and `rationale` (frontmatter or flags), records one change step on an AgentRun, and appends one `change_ops` row per operation. Without `--agent-run-id`, an apply inside an agent session (`POLTERWARE_SESSION` set by `hook session-start`) joins that session's open run for the project, opened on the first write and completed by `hook session-end` (session state lives in the core); outside a session it opens and completes a run labelled `cli`. When the project resolves, `lint` and `apply` (CLI, MCP `polter.apply`, and `WorkService.ApplyPolter`) also report similarity hints, prefixed `hint:` and never blocking: a new card, question, or invariant whose words closely match an existing active card, open question, or active invariant (after folding case, accents, plurals, stop words, and word order), and, for a new card without `under:`, the `under:` value whose path and cards share the most words. Hints use no index and no embeddings. `hook lint` lints a written `*.pw.md` file and exits 2 with line-numbered errors and hints so Claude Code returns them to the agent. The Desktop reads the same tables through `WorkService` (see Work cards and Specs). See [ADR 0006](adr/0006-polter-format-and-work-model.md) and [`architecture/polter-format.md`](architecture/polter-format.md).

Run with no subcommand in a terminal, `polterware` opens a read-mostly status TUI. It starts the core when none is running and reads through it: core status, MCP client configuration, live MCP sessions, AgentRuns, MCP tool calls, and Unified Change History for one project. It can stop and start the core, configure or remove the `polterware` MCP server in Claude Code, Codex, and Cursor, and register and scan the current directory as a project. Without a terminal, it prints usage and exits 2. See [`architecture/cli.md`](architecture/cli.md).

## Local core process

One core process (`polterware serve`) per database is the only process that opens the SQLite store ([ADR 0007](adr/0007-single-local-core.md)); an exclusive lock on `<db dir>/run/core.lock` enforces it. The Desktop, CLI commands, the TUI, and the MCP bridge find it through `<db dir>/run/core.json` (pid, endpoint, token, build; mode 0600), start it detached when none is running (log `run/core.log`), and replace it when it runs an older build than theirs (`HealthService.Check` build, `MaintenanceService.Shutdown`). The core outlives the Desktop and exits after 10 minutes without requests or open change streams (`--idle`, `POLTERWARE_CORE_IDLE`).

SQLite commit hooks record which tables each committed transaction wrote, the project ids in those rows, and the clients whose calls were in flight. The core coalesces them for 100 ms into numbered change events (kinds: projects, environments, settings, work, knowledge, ppm, agent_runs, architecture, integrations) served by `ChangeService.WatchChanges`, with `resync` when a client missed events. Electron Main keeps one stream open, reconnects (restarting the core when needed), and forwards events to renderers through `window.polterware.core.onChange`. The renderer skips events caused only by the Desktop and otherwise reloads the affected slices of the current project after 300 ms: the project shell, the extension slices, or only Work cards.

## Execution authority

Electron Main attaches to the local core or starts it (see Local core process); quitting the Desktop does not stop it. The renderer reads and writes through `window.polterware` only. Go core owns local execution:

- Project creation picks the project directory through Electron Main's native folder dialog (`app.selectDirectory`); the renderer never browses the filesystem directly.
- Integrations Connect may open a core-owned local PTY for provider authentication commands (`auth-*` / local synthetic targets). Product Terminal sessions and SSH host profiles (Nodes) are not shipped.

## Integration CLI auth

The provider registry stores provider id, label, executable name, authentication commands, and connection status. Allowed states are `cli-unavailable`, `ready`, `authenticating`, `connected`, and `failed`.

The catalog contains exactly GitHub, GitLab, AWS, Google Cloud, Azure, Vercel, Supabase, Terraform, and Docker. Flows may detect the CLI binary, start a fixed provider authentication command in the core-owned PTY, optionally open the provider documentation URL through Electron Main, and probe connection status. Credentials remain in the OS/CLI credential store. No app-owned field requests or persists provider tokens in SQLite. Raw CLI output is shown as terminal output and is not guaranteed to be sanitized.

Connected detail contains Overview, Resources, CLI, and Sync.

## Reduced core

The repository retains one Go runtime, the `polterware` binary (CLI, TUI, `serve`, `mcp`). Active domains are Projects, Architecture, Resources (inventory), Knowledge, Project Model (PPM), Context, AgentRun, Integrations, Terminal (Connect PTY only), Health, and Maintenance.

Core requests use one technical bearer session token without an account directory. The database is initialized from a clean SQLite baseline with no compatibility guarantee for earlier databases.

## Acceptance

- The initial route is Atlas / Canvas.
- The sidebar exposes Inbox, Atlas, Work, Decisions, Reality, Agents, and Integrations plus footer Settings.
- Work exposes Board and Specs; Decisions lists ADRs; Reality exposes Inventory, Sources, Facts, and Docs.
- Work › Board shows Work cards (`WorkService.ListWorkCards`) in Shape, Plan, Build, Verify, and Delivered columns; stage changes, new cards, and removal go through `WorkService.ApplyPolter` with a change envelope.
- Work › Specs shows the project Constitution (`kind = constitution`) and a table of the same cards.
- Library lists `doc`, `adr`, and `runbook` items with Kind / Status / Severity / Linked filters via the Filter popover and active chips; those kinds can link to PPM entities and Architecture shows linked knowledge for the selected component.
- Sources lists `source` items with a project-relative path; blank create, templates, and Import from project root (`ImportProjectSources` MVP set) are available. Generic filesystem indexing beyond that MVP set is not shipped.
- `SearchKnowledge` returns exact and FTS hits with provenance fields.
- PPM facts and evidence persist with provenance; `verified` facts require a non-`system` actor; evidence can link to facts via `LinkEvidenceToFact`. `ImportProjectSources` upserts observed facts/evidence from MVP project files. `CompileContext` includes facts with `reasons` and `evidence` refs.
- Work cards, Library, and Sources, Resources Inventory, Architecture, and Settings Environments empty states show inspiration template cards that create the item immediately where applicable; create dialogs also offer templates, and blank create remains on the toolbar and form.
- `CompileContext` returns a bounded package with provenance fields, machine reasons, and evidence refs.
- `ContextService.CheckDrift` returns the Narrow Drift Checker result (`checker: "narrow"`) for Desktop Agents → Context preview.
- `polterware mcp` exposes `polter.format`, `polter.query`, and `polter.apply` (session-scoped AgentRun when `agent_run_id` is absent), stable reads `context.get`, `entity.get`, `evidence.get`, and `drift.check` over stdio (`drift.check` uses checker `narrow` with typed findings), AgentRun audit tools `agent_run.start` / `append` / `complete` / `get` / `list`, and audited writes `fact.upsert` / `evidence.upsert` / `evidence.link` / `knowledge.upsert` / `knowledge.link` (required `summary`, `rationale`, open `agent_run_id`).
- `SyncProjectModel` projects canvas nodes and edges into PPM entities and relationships (`DEPENDS_ON` / `CALLS` / `RELATED_TO`).
- Agents exposes MCP helper start/stop, Cursor config copy, the Polterware AI skill, CompileContext preview, Narrow Drift Checker preview (`CheckDrift`), PPM Facts (list/inspect facts and evidence; upsert/link with change envelope), and Unified Change History (justified human + agent mutations with envelope and audit chain). Desktop Knowledge/PPM semantic writes require `summary` and `rationale`.
- `AgentRunService.ListChanges` lists change steps for the project with optional `actor_kind` filter (`human` | `agent`).
- `polterware` CLI supports `init`, `scan`, `context`, `drift`, `apply`, `lint`, `format`, `query`, and `hook` against the local SQLite store (`-db`). With no subcommand in a terminal, it opens the status TUI (core start/stop, MCP client setup for Claude Code / Codex / Cursor, live MCP sessions, runs, tool calls, and history).
- The provider catalog contains exactly the nine specified providers.
- Provider connect may probe CLI availability and authentication status without storing tokens.
- Static validation passes: typecheck, design lint, Buf lint, Go vet, and `git diff --check`.

Previous specifications are archived under `docs/future/` and are non-normative. Product direction is defined separately by `docs/product-vision.md` v3 and [ADR 0004](adr/0004-product-positioning-and-wedge.md).
