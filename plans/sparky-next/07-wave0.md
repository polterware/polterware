# Wave 0 — scaffold, spikes, contracts freeze

All Wave-0 workstreams run in parallel (W0-SCAFFOLD and W0-CONTRACTS are the only ones that produce files others depend on;
spikes are throwaway code under their own directories and produce **evidence + a decision**).
Merge order into `sparky-next`: W0-SCAFFOLD → W0-CONTRACTS → spikes (any order) → W0-DS-AUDIT.
Gate **G0** at the end of this file.

---

## W0-SCAFFOLD — repo scaffolding and shared config

**Goal.** Everything later WS need exists as a compiling stub, so nobody edits shared config again.
**Reads.** `package.json`, `pnpm-workspace.yaml`, `turbo.json`, `go.work`, `tooling/*`, `apps/desktop/*`, `.github/*`.
**Owns.** Shared files listed in AGENT-PROTOCOL §2, plus all new stub directories below.
**Deliverables.**
1. Integration branch `sparky-next` from `feat/atlas-grounded-sdd` HEAD (confirm base with Erick), with `plans/sparky-next/`
   committed as its first commit (it is currently uncommitted in the main checkout), pushed.
2. Go module `sparky/backend` (`go.mod`: `github.com/polterware/polterware/sparky/backend`, go 1.27) added to `go.work`;
   directories from the README layout with `doc.go` stubs; `runtimes/cli/main.go` printing `sparky <version>` and subcommand
   stubs (`serve`, `mcp`, `status`, `doctor`).
3. `sparky/schemas/` buf module (`buf.yaml`, `buf.gen.yaml` generating Go + Connect-Go into `sparky/backend/platform/gen` and
   `bufbuild/es` into `packages/sparky-contracts/src/gen`), a placeholder `sparky/v1/health.proto`, `.buf/baseline.binpb` placeholder.
4. `packages/sparky-contracts` (package.json, `src/index.ts` re-exporting gen).
5. Stub packages with `package.json` (`@polterware/feature-sparky-<x>`, `peerDependencies: solid-js`), `tsconfig.json`,
   `vitest.config.ts`, `src/index.tsx` default-exporting a `SparkyFeature` stub: `shell, memories, minds, schedule, calendar,
   attachments, focus, review, map, settings, search`; plus `packages/frontend/map` (stub).
6. `packages/features/sparky-shell/src/feature.ts` (the `SparkyFeature` type from 06-ui.md) and the host-facing types.
7. `apps/sparky-desktop`: copy of the **structure** of `apps/desktop` (electron-vite config with aliases for every stub
   package, `electron-builder.yml` with `appId: com.polterware.sparky`, `productName: Sparky`, empty `src/main|preload|renderer`
   that opens a window showing "Sparky" using the DS), `renderer/features.ts` importing every stub.
8. `apps/sparky-notifier`: SwiftPM/Xcode skeleton that builds an empty `SparkyNotifier.app` (bundle id
   `com.polterware.sparky.notifier`).
9. `tooling/verify-sparky.sh` (runs: `buf lint` + `buf breaking` for `sparky/schemas`, `go vet` + `go test` in
   `sparky/backend`, `pnpm --filter "@polterware/*sparky*" typecheck test`, `check-sparky-boundaries.sh`, `design:lint`, and the
   existing Polterware `tooling/verify.sh` pieces that do not need network) and `tooling/check-sparky-boundaries.sh`
   (domains import rules; no electron in packages; no cross-feature deep imports; no React).
10. Root scripts: `dev:sparky`, `build:sparky-core` (`go build -o dist/sparky ./sparky/backend/runtimes/cli`), `typecheck:sparky`,
    `test:sparky`, `verify:sparky`; turbo tasks wired; `.github/workflows/sparky.yml` running `verify-sparky.sh`.
11. `plans/sparky-next/{status,requests}/.gitkeep`.
**Acceptance.** `pnpm install && bash tooling/verify-sparky.sh` is green on a clean checkout; `pnpm dev:sparky` opens the stub
window; `pnpm dev:desktop` (Polterware) still works; no Polterware product file modified.
**Verify.** `bash tooling/verify-sparky.sh && pnpm typecheck && pnpm design:lint`.

---

## W0-CONTRACTS — freeze the data model, RPC surface and MCP schemas

**Goal.** Turn [02-data-and-contracts.md](02-data-and-contracts.md), [03-recurrence-and-reminders.md](03-recurrence-and-reminders.md)
and [04-mcp.md](04-mcp.md) into code-level contracts that W1 workstreams build against in parallel.
**Reads.** docs 02/03/04; Swift `Model/**` and `sparky-mcp/docs/CONTRACT.md` for intent; `schemas/polterware/local/v1/*.proto`
for style.
**Owns.** `sparky/schemas/**`, `sparky/backend/platform/gen/**`, `packages/sparky-contracts/**`,
`sparky/backend/providers/sqlite/migrations/0001_baseline.sql`, Go **interfaces/ports and domain types only** in
`sparky/backend/domains/*/ports.go` + `types.go` (no implementations), `sparky/backend/providers/mcp/schemas/*.json`
(tool input/output JSON schemas), `sparky/backend/domains/schedule/testdata/recurrence_vectors.json`.
**Deliverables.** Protos for every service in 02; generated Go + TS compile; `0001_baseline.sql` identical to 02 (fix errors
via a note in the status file); port interfaces + in-memory fake **skeletons** (signatures only); MCP tool schemas (18 tools)
with the token-budget descriptions; vectors file with the 14 cases in 03; a short `CONTRACTS.md`-style summary inside the
status file (not a new doc). Tag `sparky-contracts-v1`.
**Acceptance.** `cd sparky/schemas && buf lint && buf generate` clean; Go and TS builds pass; applying the SQL baseline to an
empty SQLite succeeds and a `PRAGMA foreign_key_check` is clean; every RPC listed in 02 exists; each MCP tool schema validates
against a sample call in a unit test.
**Verify.** `bash tooling/verify-sparky.sh`.

---

## W0-NOTIFIER — spike S1: OS-delivered reminders with every Sparky process closed

**Goal.** Prove the reminder architecture or trigger Plan B. **Resolves O1.**
**Owns.** `apps/sparky-notifier/**` (may restructure the skeleton), `spikes/notifier/**` (throwaway Go stub core + scripts).
**Do.** Build `SparkyNotifier.app`: authorization request, arm a notification from a CLI argument or from a stub core `Plan`
response, actions Done/Snooze/Open with a category, `sparky://` open handler stub, logging. Try both registrations
(`SMAppService.agent` and `.loginItem`) and an Electron stub that registers it via `app.setLoginItemSettings`. Measure the
pending-notification limit by arming 100 and counting. Test ad-hoc signature (`codesign -s -`) and unsigned-by-Electron
(`identity: null`) packaging.
**Acceptance (all evidence in the status file, with commands and screenshots/logs).**
1. A notification armed for T+2 min is delivered after `pkill -9` of the Electron stub, the stub core and the notifier itself
   (only launchd/OS alive).
2. Notification **Done** action reaches the stub core while Electron is closed.
3. Click opens the Electron stub at `sparky://memory/<id>` (cold start and warm).
4. The helper restarts after `kill -9` (agent) or documents why loginItem cannot; helper starts at login (manual reboot test
   documented).
5. The macOS permission prompt names **Sparky** (helper identity); behavior with permission denied is handled.
6. Pending limit measured on the host macOS version; the **minimum macOS** for SMAppService/UNUserNotificationCenter options is stated.
7. Works ad-hoc signed; states exactly what needs a Developer ID (if anything).
**Output decision (append to 00-decisions.md O1).** Registration type, budget value (`Budget`), minimum macOS, signing needs, or
"Plan B" with the evidence.

---

## W0-MCP — spike S2: MCP SDK choice and transport behavior

**Goal.** Decide official Go SDK vs hand-rolled. **Resolves O2.**
**Owns.** `spikes/mcp/**`.
**Do.** Implement 3 tools (`get_current_time`, `list_memories`, `create_memories` against an in-memory fake) twice: with the
official `github.com/modelcontextprotocol/go-sdk` (check the current version and API on the module docs) and hand-rolled
JSON-RPC like `backend/providers/mcp`. Serve streamable HTTP on loopback with bearer + Host/Origin checks, plus a stdio proxy mode.
Test with MCP Inspector and a real Claude Code (`claude mcp add …`) and Cursor config.
**Acceptance.** Table in the status file comparing: spec coverage (tools/resources/prompts/structured output/progress),
streamable HTTP + session handling, error mapping, payload size of `tools/list`, lines of code, test ergonomics, maintenance
risk; a recommendation. Both transports pass the Inspector. Anti-DNS-rebinding checks demonstrated with `curl` (bad Host, bad Origin).
**Output decision.** O2 + the exact dependency/version (or the hand-rolled design).

---

## W0-STORE — spike S3: SQLite capabilities

**Goal.** Verify the database assumptions. **Resolves O3.**
**Owns.** `spikes/store/**`.
**Do.** With `modernc.org/sqlite` (same version as `backend/go.mod`): confirm FTS5 + `bm25` + `snippet` + `prefix`; WAL with
one writer + several readers; `PreUpdate`/`Commit` hooks (copy the idea from `backend/platform/db/sqlite/hooks.go`) emitting
entity-level change events; load the baseline SQL from W0-CONTRACTS if merged, otherwise the DDL in doc 02. Benchmarks on
synthetic data: 10k and 50k memories — FTS query p50/p95, `list_memories` with filters, insert throughput, DB size; (vector and embedding benchmarks are **out of scope**: there is no semantic search).
**Acceptance.** Benchmarks table (ns/op and p95) in the status file; pass/fail against budgets (FTS p95 < 30 ms @10k; list_memories budgets); statement on any missing capability and the fallback (`ncruces/go-sqlite3` WASM).
**Output decision.** O3.

---

## W0-MAP — spike S4: free map in Electron + Solid

**Goal.** Pick the map stack. **Resolves O4.**
**Owns.** `spikes/map/**`.
**Do.** In a throwaway Electron+Solid window (dark theme): MapLibre GL JS + OpenFreeMap, Leaflet + OSM raster, and (if feasible)
PMTiles. For each: marker + radius circle, pin drop, drag, cluster 500 markers, dark style, offline behavior, CSP needed,
bundle size, attribution requirement, **terms of use for an installed desktop app (cite URL and date)**. Evaluate geocoding
options and their terms (Nominatim policy, others) without enabling any by default.
**Acceptance.** Recommendation + evidence table; a working minimal `MapView` API prototype matching 05; screenshot of the dark
map; explicit statement of what the user's viewport leaks and how the setting controls it.
**Output decision.** O4 (library, provider, attribution text, CSP entries).

---

## W0-DS-AUDIT — design-system coupling audit (read-only)

**Goal.** Make W1-DS exact. **Resolves O5.**
**Owns.** `plans/sparky-next/status/w0-ds-audit.md` only (no code).
**Do.** Read every file in `packages/frontend/design-system/src` and `packages/configuration/tailwind/base.css`. Produce: (a)
table of components → Polterware-specific? (strings, keys, imports of `@polterware/frontend-state`, domain nouns); (b) the exact
list of the files that mention "polterware" and what each reference is; (c) tokens used by Sparky's screens that don't exist
yet; (d) which Ark UI parts exist for the new primitives (Checkbox, DatePicker, TimePicker, NumberInput, Slider, ColorPicker,
TreeView) in the pinned `@ark-ui/solid` version; (e) test coverage per component; (f) a proposed ordered refactor list for
W1-DS with risk notes for Polterware.
**Acceptance.** Status file contains (a)–(f) with file paths and line references; W1-DS can start without reading the
source again.

---

## Gate G0 (the integrator checks, then tags `sparky-wave0-done`)

- [ ] W0-SCAFFOLD verify green; Polterware dev still works.
- [ ] W0-CONTRACTS merged, `sparky-contracts-v1` tagged.
- [ ] O1 resolved with passing evidence (or Plan B adopted and recorded; if Plan B, [03](03-recurrence-and-reminders.md) §Fallback is promoted to the main path before Wave 1).
- [ ] O2, O3, O4, O5 resolved and appended to the resolutions log in [00-decisions.md](00-decisions.md).
- [ ] Erick reviewed the gallery-less summary and the contracts, and approved Wave 1.
