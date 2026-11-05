# Polterware — future work

Status: direction only. **Nothing in this file is implemented.** It is the only planning document in the repository. Shipped behavior is defined by the code, `AGENTS.md`, `polterware --help`, and `polterware format`. When an item ships, delete it from here.

Category: **Project Context & Change Intelligence for AI Coding.** Polterware keeps the project state (what it is, what was decided, what is in progress, what was delivered) identical for any agent and any person, checked against the repositories.

## Principles every item must respect

- One project model. No parallel architecture, documentation, or agent models.
- Structured before inferred. Import deterministic facts (manifests, Git, OpenAPI, Terraform, Docker) before asking an LLM. Inferred knowledge stays `inferred` until a recorded human decision promotes it.
- Provenance before confidence. Every claim says where it came from, when, and what supports it.
- Local value without Cloud. Cloud adds continuity and collaboration; it never gates local features.
- People and agents share the same model, contracts, and audit semantics.
- Drift is a finding, never an automatic overwrite.
- Ops, Terminal-as-product, and Nodes stay removed.
- The Polterware store (SQLite, later Cloud) is the source of truth. Repositories give evidence in and receive projections out.
- AI asks, humans decide. Agents decide only what was explicitly delegated, and that is recorded.

## Explicitly not now

- Light themes and theme selection across the ecosystem. All apps currently use dark mode exclusively.

- A rigid state machine for agents (UNDERSTAND → PLAN → …).
- LLM calls or user LLM keys inside the Desktop or the local binary.
- A sidebar for skills, plugins, or hooks as a product pillar (setup lives in App settings › Agents).
- AI-memory or "chat over the repo" positioning; codebase-wide RAG as ingest.
- Reintroducing Operations, Terminal, or Nodes.

## Local

1. **Manual Git verification** (never run end to end). Session: `status` prints the branch; outside a work tree it still runs; `delivered` prints only when the stored stage is `delivered` and `HEAD` is an ancestor of local `main`/`master`; `handoff` compares `touched:` and `branch:` with the local diff.
2. **Two-week continuity measure.** How often context had to be re-explained, whether the agent started from the right point, whether the operator opened `status` unprompted.
3. **Broader deterministic import and scan reconcile.** More source families; prune on re-scan so drift stays honest.
4. **Provider observations (read path).** Connected CLIs/APIs become facts, not a control plane.
5. **Architecture as Code export.** Versionable YAML/JSON/Markdown projection of the relevant model. No proprietary DSL.
6. **Context profiles and secret filtering.** Intent-shaped packages; strip secrets before MCP or Desktop packages.
7. **Richer Drift UI.** Findings open the entity, decision, or fact in the shared DetailPanel.
8. **Structured SDD.** Requirements, plan, tasks, and evidence as a graph beyond Work cards.
9. **Local Change Gates.** Optional human review before durable truth; a single-user rehearsal for Cloud gates.
10. **Light enforcement.** Claude Code `PreToolUse` advisories ("this file is governed by decision X", "adding a dependency needs a decision"). Advisory first.
11. **Engineering packs** that generate Inbox *questions*, never rules.
12. **Similarity beyond word sets.** Add an FTS5 index when a ranked-search consumer needs it. Add local embeddings (`sqlite-vec`, small local model, opt-in download) only if measured misses are material.
13. **Weak spots to close:** Structure timeline by commit is weak for rebased history (no per-batch commit stamp); automatic challenge from a decision review trigger; decision status chips on Canvas nodes; importing projected Decision files.
14. **Rules before a run's merge.** An optional check of a dispatch run's diff against the Rules that hold for the nodes it touches, shown on the run's page before Merge; advisory until a Decision makes it a gate.
15. **Scope exclusion in filters.** Scope pills are inclusion-only because reads take one `scope_nodes` union; `is not` on Domain / Component / Environment needs an `exclude_scope_nodes` on the scoped reads.

## Cloud (needs a Decision before any code)

Local-first, offline-capable core stays free. Cloud is one Go service (Fly.io, same region as Supabase) over Postgres (`pgx` + `sqlc`), Supabase Auth with GitHub login (Go verifies JWTs), River for jobs, SSE for live claims, RLS as a second layer only. No Edge Functions, no `supabase-js` from the Web App, no Supabase Realtime. Cloud stores the project model, commit metadata, and judgments, never source code. The domain logic and Polter codec are one Go module shared with the local binary. The Web App lives in `polterware-web`; the public Desktop repository never authorizes hosted operations.

Replication is an operation log: local `change_ops` upload and merge, no silent magical merge. Remote MCP exposes the same `polter.apply` / `polter.query` / `polter.format` for agents that cannot run a local binary (Codex cloud, Claude web, ChatGPT); prefer local when a core exists.

Build order:

- **C0 — Read-only continuity.** `polterware login` (device flow), one-way log upload, GitHub App ingest, Web App delivery feed.
- **C1 — Coordination.** Two-way sync, claims with leases, sessions, handoffs, remote MCP.
- **C2 — Decisions.** Inbox with owners (`CODEOWNERS`), projection PRs (Decisions, `AGENTS.md` blocks).
- **C3 — Judge.** Decision detector and handoff verification as GitHub Checks.
- **C4 — Gates and billing.** Required checks, calibration, plans.

Cloud must not: ship an AI-memory product or a parallel vector store, provision VPS/AWS nodes or remote execution, gate local wedge features behind a subscription, or host customer applications. Pricing is undecided; the draft is Local (free), Team (shared board, claims across people, owners, delivery feed, remote MCP), Guard (Jev checks per PR, required check).

Open: sync operation granularity, long-offline replicas, log compaction, GitLab adapter boundary at ingest, webhook replay/backfill, whether "delivered" means merged or in production for a teammate, whether Cloud owns tasks or reads them from GitHub Issues/Linear.

## Jev as judge

TypeSafe Jev returns typed judgments with probabilities (Choice, Noul, Score), cheap enough to run on every push, PR, handoff, and apply. Code finds candidates, Jev selects, a threshold decides between acting, suggesting, and sending to the Inbox. Output stays `inferred` until a person or gate confirms. Sending diffs to a third party needs per-repository opt-in and a retention policy.

Candidate checks, strongest first: decision detector and invariant guard (one request per diff); handoff verification (does the diff support what the agent said?); commit ↔ task linking without an explicit reference; duplicate or contradictory decision/question; automatic challenge when a fact meets a decision's review trigger; Inbox routing (owner, blocking, priority); Oracle packing under a token budget.

Optional later: bring-your-own LLM key enrichment (`inferred`, promoted by a human), only if a Decision allows keys in the product.

## Research and validation (hypotheses, no experiment run)

- H1: coding agents vary meaningfully across repeated runs of the same task.
- H2: typed actions, contracts, and external verification reduce unwanted variance.
- H3: explicit decisions and evidence reduce changes incompatible with architecture and requirements.
- H4: selected context capsules reduce tokens or time without material loss of success.
- H5: engineering packs reduce relevant defects in complex scenarios.

Independent hypotheses; a win on H2 does not validate H3–H5, and more restriction may lower success (report the trade-off). Minimal design: four conditions (no Polterware, structured output, typed actions, typed actions plus contracts/states/verification), fixed snapshot, task, budget, and model per block, randomized order, repeated runs. Behavioral reproducibility `BR = 1 − D`, where `D` is the mean pairwise distance between normalized action traces; define the normalization and distance before observing results and validate against independent human judges. Report separately: task success, blocked proposals versus *executed* violations, files touched, dependencies added, external calls, tokens, latency, cost. Validate the product (interviews, real use) before widening the architecture.

## GPUI Desktop

`desktop-gpui` has the frame (with Help and its topics, Switch project's popover, and the New project dialog), the Projects screen, the Inbox, Activity with its Dispatch entries, Knowledge (the Decisions, Rules, and Docs lists; a new record, and a Rule to edit, in the shared Composer; the archived docs; and a decision's or a doc's page with its Context as tokens, its Markdown body edited in place with the slash menu, the selection toolbar, code blocks, and the gutter's "+", tags, lifecycle moves, and Activity, its rail a drawer on a narrow window), and Milestones (the Roadmap; a milestone's Board, its cards dragged between stages, and its List with groups, selection, and Peek; a new milestone or card in the shared Composer; and a card's page with its tasks, gate, verification, commits, handoff, runs, agent brief, and Activity), Structure (the tree with its search and filters, a node's document with its level, title, and summary edited in place, a component's domain, kind, technology, checkout, and deploys with their checks and the deploy Composer, the records that cite it to link and unlink, and its Activity; a new domain or component in its Composer, from N, the tree's "+", and Commander's Add structure node), Canvas (the canvases with their miniatures, created, renamed, made agent-visible, archived, restored, and deleted; and the editor with its shape library, the dotted well with selection, moving, resizing, connections, labels edited in place, undo, arrange, and zoom, the inspector for a shape, a connection, or several shapes with Promote, and autosave; and Structure's Draw as canvas), the shared Composer for a card, a decision, or a doc (from C, Commander's New card, New decision, and New doc, and each page's "+"), the settings dialog over the page (a project's General, Environments, Platforms with the provider sign-in in a terminal, and Danger zone; the app's MCP server and clients, Agent setup, Hooks, AI skill, CLI reference, Shortcuts with their remaps, Appearance, Local core with its idle, auto start, and quit choices, Privacy, and Danger zone), and the scope filter's Domain, Component, and Environment in every page's filters, one selection the scoped reads share, as Electron draws them, with live updates from the store's change stream (`ChangeService.WatchChanges`), plus Commander (its groups, tiles, and hints, the open card's actions, and a run started through Polterware Dispatch from its prompt with the route pills), the macOS menus (with Check for Updates…), and `dispatch://` links that open only when Dispatch is installed, on contracts generated from `schemas/`. `tooling/release/bundle-gpui.sh` packages it as the Electron release does (the same name, bundle id, icon, core, Sparkle feed and key, ad-hoc signing, ZIP and DMG), but the release workflow still ships Electron, and since both share the feed, an installed GPUI build updates to the next Electron release. The Electron Desktop stays the shipped app until the GPUI one is the same app screen for screen (layout, components, words, and behavior, checked side by side with `tooling/parity`), then retires and the workflow ships the GPUI bundle; `tooling/bench` measures the two. Each step builds the controls it needs as general components in polterware-ui (the Solid design system's look, gpui-kit's behavior). The order: every screen first, then every screen and component made identical to Electron's, then end-to-end checks and automated tests of what they do, and only then the performance measured again:

1. What waits for a gpui release, since the apps stay on the published gpui, never a patched or forked copy. The idle frame: GPUI 0.2.2's macOS display link fires on every vsync while the window shows, even when nothing draws, so the idle CPU is above Electron's, and neither the window layer nor the app can pause it (gpui 0.3 does the same). Letter-spacing and optical size: Electron tightens text by 0.1 to 0.16px and draws Inter Variable at each size's optical size, tighter as text grows, while gpui's fonts take neither (polterware-ui ships the static text faces), so GPUI's lines run a little wider, a 24px title about 5%, and now and then wrap a word sooner. An SVG mark filled with a gradient: an icon tile's metal sheet is drawn as flat bands, without its seeded warp. Emphasis: neither app loads an italic face, so Chromium slants Inter on its own, while gpui has no synthetic oblique and draws emphasis upright.
2. What Knowledge, Milestones, Structure, Canvas, Commander, and Settings still lack, once every screen is in. The Composer: Open on the toast a created record shows, the 0.97 scale it opens from (GPUI only fades it in), and the description's 280px cap before it scrolls. The Markdown editor: a code block's long lines scroll in Electron and wrap in GPUI; the selection toolbar does not show inside a code block; the gutter's "+" has no tooltip; the page still scrolls while the slash menu is open; and a read-only body (`markdown_view`) has no Copy on its code blocks. The rail's content does not fade at its top and bottom edges, and the archive's "Nothing archived" sits about 24px higher than Electron's. Milestones: a target date is typed as `YYYY-MM-DD` (no DatePicker yet); archiving or removing cards has no Undo on its toast; the Board has no hover marker travelling between cards and lifts a card without the 1.03 scale; the List's filters have no Display menu (grouping, ordering, properties); the Composer shows no crumbs; and Peek opens only from the List's Space. The filters: no saved filters (`createSavedFilters` over `SavedFilterService`: listed at the end of the menu, applied, saved from the bar, deleted), and Commander's New card, New decision, and New doc are offered only inside a project. Structure: a record a node's document lists opens its page at once, not a peek first; the platforms show a cloud, not their brand marks (`ServiceMark`); the node and deploy Composers show no crumbs, and Add deploy is not disabled before an environment and a platform are picked (a toast says so); record rows light under the pointer without the table's sliding highlight, and a section's Link picker hangs from the `+`'s left edge, not its right; the tree's open branches and its pane's toggle last only while the app runs (Electron remembers them); the document does not scroll under a floating toolbar; the Level and Kind filter options have no icons; Shift+Enter does not break the summary's line; and the prompt cards' label is not spaced out. Settings: a search result opens its tab without scrolling to its section; Add environment's Composer has no template body and no crumbs; Platforms has no Category, CLI, or Status filter and no brand marks; the CLI reference's index does not jump to a command; Hooks' rows do not light under the pointer; an environment row expands, and the connect dialog and the confirmations open, without motion; the quit question has no "Don't ask again"; Download .md saves into Downloads and reveals the file, not a save panel; a shortcut's remap reaches Commander, C, and N, while the other chords stay as GPUI binds them; and Projects' Project settings opens the whole project scope, not only the General tab's. Canvas: Archive's toast has no Undo and Draw as canvas's toast no Open canvas (Canvas opens on the drawn canvas instead); the outline's drop is the face painted twice more, not a blur, and lines join mitered; a card's hover dims its miniature under a veil instead of brightening it; a label edited on the canvas is a fixed-size field, not text at the zoom; external components show no brand marks and icon tiles no seeded warp; the remembered canvas and the panels' widths last only while the app runs; the Composer shows no crumbs, and Promote's shows neither the record's stage line nor its error under the title (a refusal is a toast); the project reference lists entities as rows, not the sortable table; the library's and the inspector's sections sit as loosely as a detail panel's, not tighter; a library drag shows the kind's name, not its drawing; and a number stepper's value is not centered. Commander: switching between the results and a run swaps the panel's height at once, without Electron's glide, fade, and 6px rise; Keyboard shortcuts opens the Shortcuts settings tab, not Electron's shortcuts panel; and Connect integration opens Platforms without its connect dialog. The menus: Window has Minimize and Zoom, without the Bring All to Front and window list that Electron's `windowMenu` role adds.
3. End-to-end checks of every screen's behavior and the automated tests that keep it, then the performance against Electron again (`tooling/bench`).

## Dispatch still open

Polterware Dispatch lives in its own repository, [polterware/dispatch](https://github.com/polterware/dispatch). Polterware serves it through `DispatchHostService` and reads its runs through the Dispatch core; a card's runs and Activity's dispatch entries come from it. What is open on the Polterware side:

Still open:

- Turn a run's diff into Evidence on the card.
- Compare Drift with the real worktree diff.
- Treat Delivered as merged to the target branch.
- A Waves tab.
- Dispatch MCP tools.
- Context Compiler suggestions that pick a tier from run history.
- Polterware Vitals integration (host metrics).

## Git and worktrees as the change ledger

Direction: Git is the most reliable evidence of what changed, so Polterware reads it deeply and ties every change to the card, Decision, agent, and run that justified it. It is evidence and lifecycle, never a Git client.

Starting point (shipped, narrow): the scan writes a `git:HEAD` row and up to 20 `git:commit` rows, each repository clone adds `git:<name>:HEAD`, `status` prints the branch, `delivered` checks that `HEAD` is on local `main`/`master`, and `handoff` compares `touched:` and `branch:` with the local diff (see Local item 1; end-to-end verification is still pending). Everything below is not implemented.

Worktree lifecycle still open (the engine already creates one worktree per run, records path, branch, base, owner, card, and status, and keeps one writer per directory):

- Clean up: remove merged or abandoned worktrees and branches, prune stale ones on `serve` start, and never delete a worktree with uncommitted changes without a finding in the Inbox.
- Dirty main checkout is a finding, not an error: runs may start from a clean worktree while the user keeps editing.

Change evidence:

- Per run: base, head, diff summary (files, additions, deletions), touched paths, and commits, stored as Evidence on the card and linked to the AgentRun.
- Commit ↔ card linking: explicit reference first (trailer or branch name), then touched paths against the card's components. Inferred links stay `inferred` until confirmed.
- Delivered means merged: a card moves to Delivered when its branch is an ancestor of the target branch, not when an agent says so. Squash and rebase merges need patch-id or tree comparison, since ancestry is lost.
- Rebased history: stamp commits per batch so the Structure timeline survives rewrites (closes the weak spot already listed under Local).

Drift with Git:

- Declared vs real: compare the diff against what the card, its components, and active Decisions declared (touched paths outside the card's components, new dependencies without a Decision, changes in governed files).
- Cross-run conflicts: flag worktrees whose touched paths overlap before merge, with both runs and cards named.
- Stale base: warn when a worktree's base is far behind the target branch and its card is in Verify.
- Findings open the commit, diff, or worktree in the shared DetailPanel.

Review and merge in one place:

- A card in Verify shows the worktree diff, verify result, relevant Decisions, and open Inbox items; approving records who approved and moves the card on.
- Merge itself stays a Git operation (fast-forward or merge commit chosen per project); Polterware may run it for a local branch when the user confirms, and never pushes without explicit confirmation.
- Projection back to Git: commit trailers or notes linking commit to card and run, only if the user opts in.

Boundaries:

- No stage/unstage, interactive rebase, blame, or graph UI. If a screen looks like a Git client, it is out of scope.
- No hosting-provider logic in the core (PRs, checks, reviews) beyond reading evidence through the Integrations path; GitHub App ingest belongs to Cloud (C0).
- Source code is read for evidence locally and is never uploaded; Cloud stores commit metadata only.
- Shell out to the `git` binary (as the importer does) instead of embedding a Git library, unless measurements show it is a bottleneck.
- Destructive operations (`reset --hard`, force push, branch deletion with unmerged commits) are never run automatically.

Open: branch naming scheme and whether it is configurable; where worktrees live (inside the repo, a sibling directory, or app data); behavior in repositories with submodules or sparse checkouts; large monorepos and diff cost; whether abandoned runs keep their branch for a retention window; how to treat commits made outside any run.

## Larger ideas not yet decided

- **Behavioral runtime.** Project and task contracts, typed actions, and an execution state machine that mediates effects where an integration can. Enforcement is only possible where the agent host lets Polterware mediate; elsewhere it stays advisory.
- **Engineering knowledge library.** Versioned sources, principles, applicability conditions, and checks. No paper becomes policy automatically. Context capsules select facts, Decision excerpts, gaps, and source paths under a token budget.
- **Decision ownership.** Authority classes (human-only, delegated, agent-local), a delegation budget, and Decision Inbox routing. Humans choose architecture, method, and test strategy; accepted decisions and invariants bound the agent.
- **Compact agent encodings.** The brief, the layered format, and the context hook ship; encodings such as TOON are untried and must beat plain text in `token_budget_test` before they replace it. Anti-ideas: dumping the architecture into the prompt, a vector store as truth.
- **Multi-repo answers.** Cross-repository questions resolved from linked entities and evidence across projects, without cloning the other repository; federation only with Cloud.

## Product questions still open

1. Who creates the domains of the Structure: by hand, imported from code, or agent-suggested and human-approved?
2. Should the default route flip from Inbox to Work?
3. Work state per feature, per branch, or per session? (Leaning: per feature, branches linked.)

## Desktop motion backlog

Small, independent polish items, using the existing `--ease-standard`, `--ease-enter`, `--duration-fast`, and `--duration-control` tokens (see the `polter-fluid-motion` skill). Verify each against current code before starting; file paths from earlier plans may have moved.

1. Shared presence motion (`MotionPresence`, Ark-backed) and an animated `DataTable.SelectionToolbar` that stops mounting and unmounting instantly. Everything below depends on it.
2. Notification "Mark all as read" animates into the empty state instead of applying synchronously.
3. Integration setup wizard bridges the credential-method swap (Assume Role ↔ Access Key fields).
4. Inline validation errors in dialogs get restrained presence instead of an instant mount.
