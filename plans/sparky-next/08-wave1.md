# Wave 1 — backend, Electron shell, design system, notifier

Starts after G0. Workstreams run in parallel against the **frozen contracts** (`sparky-contracts-v1`) and the Go ports from
W0-CONTRACTS. Where a WS needs another WS's implementation, it uses the in-memory fakes or a stub.
Merge order: W1-DS, W1-ASSETS, W1-DOMAIN, W1-RECURRENCE, W1-STORE → W1-CORE → W1-SEARCH, W1-ATTACH → W1-MCP → W1-ELECTRON →
W1-NOTIFIER. Gate **G1** at the end.

Shared reading for every Go WS: [02-data-and-contracts.md](02-data-and-contracts.md), AGENT-PROTOCOL §6, the Polterware
patterns at `backend/platform/*`, `backend/compositions/local-core/*`, `backend/providers/sqlite/*`.

---

## W1-STORE — SQLite provider

**Owns.** `sparky/backend/providers/sqlite/**` (except the baseline SQL, owned by W0-CONTRACTS — read-only for you),
`sparky/backend/platform/db/**`, `sparky/backend/platform/changes/**`.
**Goal.** Implement every repository port against SQLite; migrations runner; the change recorder.
**Do.**
- `platform/db`: `Open(path)` (WAL, `foreign_keys`, `busy_timeout`, `synchronous=NORMAL`, one writer connection + a small reader
  pool), forward-only `Migrate`, `schema_migrations`.
- Repositories for minds, memories (+ check items, tags, completions, schedule, place, attachments metadata), tags, focus
  sessions, activity, settings, embeddings; all mutations within a `Tx` helper that also applies the FTS update through the
  `search.Index` port when present.
- `platform/changes`: copy the Polterware idea (pre-update + commit hooks → once per committed transaction emit
  `[]Change{entity,id,op}`), a bounded in-process broadcaster with `seq` numbers and replay of the last 1 000 changes for
  `Watch(since_seq)`.
- Keyset pagination for lists; no `OFFSET` for user lists.
**Acceptance.** Repository contract tests (a shared suite in `tests/repotest` run against the SQLite impl and the in-memory
fakes — W1-DOMAIN supplies the fakes); `foreign_key_check` clean after random operation fuzz; Mind delete cascades children and
nulls memory `mind_id`; change events are emitted only after commit and never on rollback; concurrent reader during a write
does not block (WAL test).
**Verify.** `cd sparky/backend && go vet ./providers/sqlite/... ./platform/... && go test ./providers/sqlite/... ./platform/... -race -count=1`.

## W1-DOMAIN — domain services (pure)

**Owns.** `sparky/backend/domains/{memory,mind,tag,focus,review,activity,settings}/**` (implementations + fakes + tests).
**Goal.** All business rules, no I/O.
**Do.** Services for: Mind tree (cycle detection, recursive delete result, reorder), Memory (create/update/patch semantics with
`null` clears, `expected_rev`, status rules incl. recurring completion and `auto_complete_on_checklist`, pin, priority, bulk
actions, checklist replace with identity preservation, tags set/merge), Tag, Focus (defaults validation 1–60 / 1–12, session
recording), Review (activity per day and period **morning 06–12 / afternoon 12–18 / evening 18–22 / night otherwise**, streak,
totals, 12-month contribution days, mood thresholds: streak 0 sad, 1–6 hopeful, 7–29 happy, ≥30 euphoric; completed 0 sad, 1–24
hopeful, 25–99 happy, ≥100 euphoric), Activity (append + summaries), Settings (typed keys with validation and defaults),
name lookup helpers for MCP (case-insensitive, "did you mean"). Every mutating method takes an `Actor` and writes `activity`.
Also the in-memory fakes of every repository port.
**Acceptance.** ≥ 90 % statement coverage in `domains/{memory,mind,tag,review}`; table tests for each rule above; Review
reproduces the Swift thresholds (read `Views/Settings/MeMetrics.swift`, `MeCharacterMood.swift`); `check-sparky-boundaries.sh` green.
**Verify.** `cd sparky/backend && go test ./domains/... -race -count=1 -cover`.

## W1-RECURRENCE — schedule engine and reminder planner

**Owns.** `sparky/backend/domains/schedule/**`.
**Goal.** Implement [03-recurrence-and-reminders.md](03-recurrence-and-reminders.md) exactly.
**Do.** `Rule` (Validate/Next/At/Count/Describe), `planner.go` (nearest-first global budget, horizon, completed-skipping,
all-day time, body/title shaping), occurrence listing for the calendar (`ListOccurrences` logic: range, filters, completion
state, overdue flag) as a pure function over memories + completions, `Preview`. Load `testdata/recurrence_vectors.json`;
write the golden comparison against the Swift reference for the cases listed in 03 and document intentional differences in
the package `doc.go`.
**Acceptance.** All 14 vectors pass; fuzz test (strictly increasing, `Next` consistency, no panic on random rules/zones incl.
`Pacific/Auckland`, `Asia/Kolkata`, `America/Sao_Paulo`, `Europe/London` across DST edges); benchmarks: `At` for 1 year of hourly
< 1 ms; planner for 5 000 memories < 50 ms.
**Verify.** `cd sparky/backend && go test ./domains/schedule/... -race -count=1 -fuzz=. -fuzztime=20s`.

## W1-CORE — the core server

**Owns.** `sparky/backend/compositions/core/**`, `sparky/backend/runtimes/cli/{main,serve,core,status,doctor}.go`,
`sparky/backend/platform/{runstate,ids,token,listen,clock,coreclient}/**`.
**Goal.** The `sparky` binary's `serve` runs the core end to end (with MCP handlers wired by W1-MCP later).
**Do.** Port the Polterware patterns (own code, no imports): `runstate` (`core.json`, `core.lock`, `core.log`, 0600), token mint,
loopback listener, Connect handlers for every service in 02 delegating to domain services (error mapping per 02), bearer
interceptor (constant-time), `ChangeService.Watch` stream, `HealthService`, build id handshake (`coreclient.BuildID`), graceful
shutdown, log rotation, `doctor` and `status` commands, `ReminderService` handlers (`Plan/Ack/Snooze/ArmOneOff/CancelOneOff/Heartbeat/Status`, planner from W1-RECURRENCE),
`MaintenanceService` (Export zip, Backup via `VACUUM INTO`). Wire SQLite repos, search index, attachments, embeddings by their
ports (they can be stubbed until their WS merge; use build-time registration so merges don't conflict).
**Acceptance.** `dist/sparky serve` starts, writes `core.json`, second instance exits 3 with the Polterware-style message;
integration tests (`tests/integration`) drive every RPC over a real loopback server against a temp DB; unauthenticated calls
fail; a write from one client is observed on `Watch` within 100 ms; rev-conflict returns `FAILED_PRECONDITION` with the entity.
**Verify.** `pnpm build:sparky-core && cd sparky/backend && go test ./compositions/... ./runtimes/... ./tests/integration/... -race -count=1`.

## W1-MCP — MCP server

**Owns.** `sparky/backend/providers/mcp/**` (except `schemas/*.json`, W0-CONTRACTS), `sparky/backend/runtimes/cli/mcp.go`,
`sparky/backend/tests/mcp/**`.
**Goal.** Implement [04-mcp.md](04-mcp.md) with the SDK/transport decided in O2.
**Do.** 18 tools mapped to the same domain services as the UI (actor `mcp:<client>`), per-session read-rev tracking for the
conflict rule, name resolution, schedule input parsing via `domains/schedule`, batch semantics, size-budget enforcement, stdio
bridge (`sparky mcp`) that finds/starts the core, HTTP at `/mcp` with the security rules in 01, resources/prompts if cheap.
**Acceptance.** All tests listed in 04 pass; `tools/list` ≤ 6 KB; golden transcripts committed; a recorded session with a real
Claude Code client creating, searching and completing memories; Inspector smoke documented.
**Verify.** `cd sparky/backend && go test ./providers/mcp/... ./tests/mcp/... -race -count=1`.

## W1-SEARCH — FTS5 and semantic search

**Owns.** `sparky/backend/domains/search/**`, `sparky/backend/providers/search/**`, `sparky/backend/providers/embeddings/**`.
**Goal.** Implement the search section of [05-search-attachments-map.md](05-search-attachments-map.md).
**Do.** `search.Index` implementation (FTS5 upsert/delete/rebuild; coordinates with W1-STORE through the port), query builder
(tokenizer-safe, prefix, bm25 weights, snippets), embedding workers + providers (`ollama`, `openai-compatible`; HTTP client with
timeouts/backoff; key received at runtime, never persisted), brute-force vector search with heap top-K, RRF fusion, status
reporting, `Reindex`, filters applied pre-fusion.
**Acceptance.** Fixtures in Portuguese/English (diacritics, prefix, phrases, operators escaped); budgets from W0-STORE; with a
fake embedding provider (deterministic hash embeddings) hybrid ranking tests pass; provider failure degrades to `fts` with
`modeUsed`; no secret ever appears in logs (test with a canary key).
**Verify.** `cd sparky/backend && go test ./domains/search/... ./providers/search/... ./providers/embeddings/... -race -count=1`.

## W1-ATTACH — attachments store

**Owns.** `sparky/backend/domains/attachments/**`, `sparky/backend/providers/attachments/**`.
**Goal.** Implement the attachments section of 05.
**Do.** `ImportFromPath` (regular files only, symlink rejection, copy + sha256 + mime sniff + limits), `AddLink` (http/https only;
optional title fetch with SSRF guard, 5 s/1 MB), delete (files + rows), orphan sweep, `OpenInfo`, path construction from ids
only, MIME allowlist for previews, export hooks for `MaintenanceService`.
**Acceptance.** Tests for: traversal attempts (`../`, absolute paths in names), symlink, oversize, duplicate hash, concurrent
imports, SSRF guard (127.0.0.1, 10.x, 169.254.x, `localhost`, DNS rebinding to private IP), crash-safety (temp file + rename).
**Verify.** `cd sparky/backend && go test ./domains/attachments/... ./providers/attachments/... -race -count=1`.

## W1-ELECTRON — Electron shell (Main + preload)

**Owns.** `apps/sparky-desktop/src/main/**`, `apps/sparky-desktop/src/preload/**`, `apps/sparky-desktop/src/shared/**`,
`apps/sparky-desktop/electron-builder.yml`, `apps/sparky-desktop/package.json`, `apps/sparky-desktop/resources/**` **except**
icons (W1-ASSETS).
**Reads.** `apps/desktop/src/main/*` (supervisor, ipc, updater, window, menu, security tests).
**Goal.** A secure shell that ensures the core, bridges it to the renderer, and handles deep links, updates and background mode.
**Do.** Port (own code): `process-supervisor` (`ensureCore`, build handshake, detached spawn, `SPARKY_CORE_*` overrides),
`core/client.ts` (Connect-ES clients from `@polterware/sparky-contracts`), `change-watcher.ts`, typed IPC with schema-validated
channels, preload `window.sparky` (namespaces in 06-ui.md; **no** endpoint/token exposed), window creation with vibrancy
per DESIGN.md exception, single-instance lock + `sparky://` protocol registration and routing, native dialogs
(`pickFiles`, save zip), `sparky-file://` protocol restricted to `<data>/attachments`, microphone permission helper,
`app.setLoginItemSettings` registration of the notifier (per O1), keep-running-in-background behavior, `electron-updater`
wiring (disabled until packaging), application menu, Help menu entries, CSP management (map hosts toggled by setting),
notifier launch/health status for the UI, `electron-builder` config bundling `dist/sparky` and the notifier.
**Acceptance.** Dev boot: window appears, core spawns once (kill the window — core stays), reconnect works after core restart;
the Polterware security tests ported (`window-security`, `navigation-policy`, `ipc`) pass; `window.sparky.memories.list()` works
in the stub renderer against a real core; deep link `open sparky://memory/<id>` reaches the renderer router; no Node API
reachable from the renderer (test).
**Verify.** `pnpm --filter @polterware/sparky-desktop typecheck test` and `pnpm dev:sparky` smoke recorded in status.

## W1-DS — design-system decoupling and new primitives

**Owns.** `packages/frontend/design-system/**`, `packages/frontend/state/**`, `packages/configuration/tailwind/**` (additive
tokens only), `apps/gallery/**`, and the minimal Polterware call-site edits needed for the provider/props changes
(`apps/desktop/src/renderer/**`, `packages/features/shell`, `packages/features/knowledge`, `packages/features/architecture`,
`packages/features/settings`, `packages/features/integrations`, `packages/features/commander`) — **edits there are limited to
passing the new props/provider; no visual or behavioral change**.
**Reads.** `plans/sparky-next/status/w0-ds-audit.md`, [06-ui.md](06-ui.md) §Design-system work, DESIGN.md.
**Do.** Items 1–5 of the DS section in 06-ui.md, in this order: (1) provider + storage prefix, (2) generalize product-flavored
components, (3) new primitives, (4) gallery, (5) non-regression. Update DESIGN.md first when a rule changes (AGENTS.md rule),
keep `check-ui-opacity` green.
**Acceptance.** Polterware: `pnpm typecheck`, `pnpm test`, `pnpm design:lint` green and a manual `pnpm dev:desktop` smoke with
before/after screenshots of Inbox, Work board, Atlas, Settings showing no unintended change; every new primitive has tests
(keyboard, focus, controlled/uncontrolled, reduced motion) and a gallery entry; no `rgba`/opacity violations.
**Verify.** `pnpm typecheck && pnpm test && pnpm design:lint && pnpm --filter @polterware/gallery build`.

## W1-ASSETS — icon, logo, characters, sounds

**Owns.** `apps/sparky-desktop/resources/**` (icons/logo/characters/sounds only), `apps/sparky-notifier/Resources/**`.
**Reads.** `/Users/erickpatrickbarcelos/codes/sparky-workspace/sparky/new.icon` (`icon.json`, `Assets/Star 1.png`), Polterware's
`apps/desktop/resources/{app-icons,logo,icon.icns}`, and the Swift asset catalogs and sounds under
`/Users/erickpatrickbarcelos/codes/sparky-workspace/sparky/sparky/` (find `*.xcassets`, `*.caf|*.wav|*.aiff|*.mp3`; character images
named `MeStreak*`/`MeCompleted*` or similar; `sparky-mcp/src/assets/sparky-character.webp`).
**Goal.** Complete brand/asset set in Polterware's format.
**Do.** Render the `.icon` (Icon Composer / `actool`) to 1024 px PNG; produce `app-icons/{main,black,white}.png`,
`logo/{black,white}.png`, `icon.icns` (all sizes 16–1024) and a legibility sheet at 16/32/64/128 px on dark and light; check
clipping at the layer scale; export mood characters to WebP/PNG; convert notification and focus sounds to formats usable by
`UNNotificationSound` (CAF/AIFF ≤ 30 s) and by Web Audio (WebM/MP3), with an inventory table of names → files.
**Acceptance.** Files exist with the same naming as Polterware; `icon.icns` opens in Preview and shows at all sizes; the
legibility sheet is committed for Erick's review (he decides about the optional green dot); inventory table in the status file.
**Verify.** `bash -c 'iconutil --version || true'` plus file-existence checks; `branding-assets.test.ts` ported by W1-ELECTRON passes.

## W1-NOTIFIER — production notifier

**Owns.** `apps/sparky-notifier/**` (after W0-NOTIFIER merged).
**Goal.** Implement the notifier behavior in 03 on the proven registration approach.
**Do.** Real Connect/HTTP client to the core (token from `core.json`), `ChangeService.Watch` subscription with reconnect, plan
diff (id + content hash), categories/actions, snooze, focus-phase notifications (`ReminderService` one-off items), re-plan
triggers (start, change, wake, tz change, delivery, 6 h timer), `ensureCore` at login, heartbeat/`Ack`, permission-denied
handling, logs, packaging script that embeds the app into the Electron bundle and signs it (ad-hoc/identity per A12).
**Acceptance.** The W0 acceptance list again, now against the **real** core with real data: arming 200 memories respects
`Budget`; completing an occurrence from the notification updates the UI live; editing a schedule in the UI re-arms within 2 s;
tz change re-plans; with Electron closed everything still works; unit tests for the diff logic (Swift Testing).
**Verify.** `xcodebuild test` (or `swift test`) for the notifier + the scripted end-to-end in `apps/sparky-notifier/scripts/e2e.sh`.

---

## Gate G1 (integrator)

- [ ] `bash tooling/verify-sparky.sh` green on `sparky-next`.
- [ ] `pnpm typecheck && pnpm test && pnpm design:lint` green (Polterware non-regression).
- [ ] MCP smoke transcript with Claude Code attached to a dev core; `tools/list` size within budget.
- [ ] `pnpm dev:sparky` shows the shell window connected to the real core (stub features), core survives window close.
- [ ] Notifier E2E against the real core passes (Electron closed).
- [ ] Gallery reviewed by Erick (screenshots in status files).
