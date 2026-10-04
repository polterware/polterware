# Decisions, assumptions, open questions

## Locked by Erick

| ID | Decision | Rationale | Reversal cost |
|----|----------|-----------|---------------|
| D1 | Code lives in the Polterware monorepo (`apps/sparky-desktop`, `sparky/backend`, `packages/features/sparky-*`). The plan lives in `plans/sparky-next/` (AGENTS.md exception, see README) | One workspace, atomic design-system changes, Turbo runs both apps | Medium |
| D2 | Go for the Sparky core, CLI and MCP. Separate Go module; **does not import `polterware/backend/**`** in v1 (copy-adapt patterns) | Light single binary, mature daemon tooling, same platform as Polterware; separate code keeps the apps independent | High |
| D3 | Execution in waves with gates (W0 spikes → W1 backend/DS/shell → W2 features → W3 hardening) | A failing reminder spike must not waste feature work | Low |
| D4 | No users: no importer, no migration, no Sparkle, no coexistence. Swift app = read-only reference | Nobody uses it | None |
| D5 | v1 scope = full list in README (core + calendar + focus + review + attachments + map + FTS (text search only; **no semantic search**) + expanded MCP + CLI/API) | Erick's choice | Medium |
| D6 | Agents run in worktrees, may commit and run everything (builds, tests, dev server, browser) | Erick's choice | None |
| D7 | Package scope stays `@polterware/*`; renaming is the last task (W3-CONVERGE, optional) | Avoids churn in Polterware now | Low |
| D8 | Electron (not Tauri), same shell pattern and versions as `apps/desktop` | Reuse the proven Polterware setup and the Chromium rendering for the shared design system | Medium |
| D9 | Same brand/theme as Polterware (one token set, one dark theme). App-level accent only through `--color-brand` override | "Same family" requirement | Low |

## Assumptions (veto any of these before Wave 0 starts)

| ID | Assumption | Default |
|----|-----------|---------|
| A1 | v1 targets macOS only (native reminders use UNUserNotificationCenter). Windows/Linux are future work | macOS 15+ (confirm the minimum in W0-NOTIFIER) |
| A2 | Location data (`places`) is stored and shown on the map, but **geofence arming is not implemented** (same as the current Mac app) | Place has `event` enter/exit stored for the future |
| A3 | Tags are in the core model (the Swift app has them) | Yes |
| A4 | MCP is local only: loopback HTTP + stdio bridge, no OAuth, no pairing, no remote | Yes; remote MCP is out of scope |
| A5 | The Swift "Me" tab becomes **Review** | Yes |
| A6 | Focus timer runs in the renderer; it arms a one-off OS notification for each phase end through the native reminders module | Yes |
| A7 | **No semantic search** (no embeddings, no vectors, no provider, no local model). Search is FTS5 only; an MCP agent may rerank results itself (Erick, 2026-10-04) | Decided |
| A9 | Map uses free tiles without an API key; provider chosen by W0-MAP. Geocoding (address search) is opt-in and disabled by default because it sends text to a third party | Yes |
| A10 | App display name "Sparky"; bundle id `com.polterware.sparky`; URL scheme `sparky://`; data dir `~/Library/Application Support/Sparky/` | Yes |
| A11 | App icon = `/Users/erickpatrickbarcelos/codes/sparky-workspace/sparky/new.icon` (8-point star, dithered gradient, dark tile) | Yes; W1-ASSETS exports the variants |
| A12 | macOS code signing: Polterware ships with `identity: null` (unsigned). Sparky starts the same; W3-PACKAGING decides Developer ID. The notifier must work ad-hoc signed (W0-NOTIFIER proves it) | Yes |

## Open questions (resolved by a spike, then recorded here)

| ID | Question | Resolved by | Deadline |
|----|----------|-------------|----------|
| O1 | ~~SMAppService helper~~ **Resolved 2026-10-04**: native module in Electron Main; final form (N-API vs Swift+koffi) and relaunch behavior by W0-NATIVE-NOTIFY | W0-NATIVE-NOTIFY | G0 |
| O2 | Official Go MCP SDK vs hand-rolled JSON-RPC (Polterware's MCP is hand-rolled) | W0-MCP | G0 |
| O3 | Does `modernc.org/sqlite` ship FTS5 and the commit/preupdate hooks we need? (vector latency was also measured; no longer needed) | W0-STORE | G0 |
| O4 | Map provider and library: MapLibre GL + OpenFreeMap vs Leaflet + raster vs PMTiles; terms of use; CSP | W0-MAP | G0 |
| O5 | Which design-system files are Polterware-specific vs shared; exact coupling list | W0-DS-AUDIT | G0 |

Record each resolution as a dated line appended below.

## Resolutions log

- 2026-10-03 O1 (W0-NOTIFIER, `status/w0-notifier.md`, partial): OS-delivered reminders **work** with every Sparky process killed and ad-hoc signing (delivered, Done reached the core, `sparky://` opened Electron cold/warm). Pending limit on macOS 26.6.2 accepts >= 100 (Budget stays 60); shipping minimum macOS 15. **A separate launchd helper does not stay alive**: `SMAppService.agent`/`loginItem` start but launchd refuses to exec with `OS_REASON_CODESIGNING`/`EX_CONFIG` (ad-hoc and Apple Development), so keep-alive and start-at-login were not observed.
- 2026-10-04 O1 decision (Erick approved): **drop the helper app as the primary path.** Deliver reminders from a small **native module inside Electron Main** that calls `UNUserNotificationCenter` under Sparky.app's own bundle identity (same mechanism as the current Swift app, no launchd, no Developer ID). Reminders already armed fire with everything closed; re-arming happens when Sparky (or the core via MCP/CLI) runs; simple rules use native repeating triggers so they never run out. Honest UI wording: "Reminders are refreshed when Sparky runs". Developer ID + an always-alive helper stays an optional later upgrade. Verified by spike W0-NATIVE-NOTIFY (below).
- 2026-10-04 O4: **MapLibre GL JS + OpenFreeMap** (`https://tiles.openfreemap.org/styles/dark`) for online tiles; **supercluster** for marker clustering; **PMTiles** for optional offline/regional tiles (Protomaps basemap, user-provided or bundled region file); Leaflet+OSM raster rejected for v1 default due to OSM tile policy limits on desktop apps. Geocoding off by default; Nominatim acceptable only as opt-in manual search (≤1 req/s). CSP when `map.online`: `connect-src https://tiles.openfreemap.org`; `worker-src 'self' blob:`; `img-src 'self' data: blob:`. Attribution via MapLibre control (OpenMapTiles/OSM). (W0-MAP, `plans/sparky-next/status/w0-map.md`, `spikes/map/evidence/dark-map-maplibre.png`)
- 2026-10-03 O2: Use github.com/modelcontextprotocol/go-sdk v1.8.0, stateful streamable HTTP, with a Sparky bearer + Host:port + Origin wrapper; compact InputSchema and no output schemas to keep `tools/list` within 6 KB. (W0-MCP, `status/w0-mcp.md`)
- 2026-10-03 O3: modernc.org/sqlite v1.59.0 (SQLite 3.53.4) has FTS5 (bm25, snippet, prefix), WAL with concurrent readers, and PreUpdate/Commit hooks that emit entity-level changes; FTS p95 5.2 ms at 10k memories. Semantic/vector search was later removed from scope (A7). (W0-STORE, `status/w0-store.md`)
- 2026-10-03 O5: Design-system coupling is three production imports of @polterware/frontend-state (table.tsx double-click calls detailPanelStore.expand; detail-panel.tsx hardcodes shortcut ids shell.toggle_detail and shell.maximize_detail; panels/resizable.tsx hardcodes shell.toggle_sidebar). Product default copy is PageOffline plus the vendor map DITHER_BRAND_ACCENTS. No design-system file hardcodes a polterware.* storage key. @ark-ui/solid@5.39.2 ships Checkbox, DatePicker (date only), NumberInput, Slider, ColorPicker, TreeView, SegmentGroup, Progress.Circle; it has **no TimePicker** (W1-DS builds one). (W0-DS-AUDIT, `status/w0-ds-audit.md`)
- 2026-10-04 O1-final (W0-NATIVE-NOTIFY, `status/w0-native-notify.md`, done): **Objective-C++ N-API module in Electron Main** (Swift+koffi works too and stays as an alternative; N-API chosen: direct thread-safe callbacks, no 20 ms poll, no FFI dependency, smaller bundle). Verified on macOS 26.6.2 / Electron 44.4.5, **ad-hoc signed, no Developer ID and no extra entitlements**: one-shot and native repeating (`UNCalendarNotificationTrigger`) notifications are delivered after `kill -9` of everything; real Notification Center **Done/Snooze with Electron closed** applied in ~340/320 ms (cold relaunch, delegate registered before Electron readiness, authenticated core call, re-plan, Electron exits 10 s after the action); hidden relaunch `open -g -j -a <app> --args --background-replan` keeps the other app frontmost, opens no window, no Dock icon; permission prompt reads "Sparky" Notifications. Background Electron during a hidden re-plan: Main ~120 MB RSS, ~215-221 MB with child processes (for ~10 s). Fallback timer path **not** adopted. Not verified: physical sleep/wake and real timezone change (hooks wired via injected events); a real reboot. Production work: W1-NATIVE-NOTIFY.
- 2026-10-04 G0 approval (Erick): Wave 1 approved; remote `sparky/w0-*` branches and tag kept; opacity-lint allowlist widening from W0-SCAFFOLD kept. Perf guardrails adopted: Electron quits when the last window closes (armed native reminders still fire; actions relaunch hidden), per-memory cap on calendar occurrence expansion, lazy planner with debounce, lazy-loaded images in the UI.
- 2026-10-04 Contract v1.2 (Erick approved): `SearchMemoriesRequest` gains optional `due_from`, `due_to`, `updated_from`, `updated_to` (Timestamp, fields 8-11; additive, `buf breaking` clean, baseline refreshed). Local tag `sparky-contracts-v1.2`.
- 2026-10-04 MCP optimistic concurrency (Erick approved): optional `expected_rev` on `set_memory_status`, `toggle_check_item`, `delete_memories`, `delete_mind` and `add_attachments` (per-id map for batches); `Delete` and `ToggleCheckItem` use cases take the expected revision and check it atomically in the domain. Applied in a follow-up batch after W1-MCP-b merges.
- 2026-10-04 Quality mode (Erick): one independent Codex review (gpt-6.1-sol, read-only) of W1-CORE, W1-MCP and W1-ATTACH was run; its findings were fixed (fx-core, fx-mcp, fx-attach, fx-mcp-b). The second review by another engine was skipped at Erick's request. Blind spot: findings come from a single reviewer using static reading only.
- 2026-10-04 Contract v1.4 (Erick approved): `ReminderItem` gains `ReminderDateComponents repeat = 8` (tz, optional weekday/hour/minute/second; additive, `buf breaking` clean, baseline refreshed) so open-ended interval-1 minutely/hourly/daily/weekly rules are armed as native repeating triggers. Core `Plan` serializes the matcher; Main refuses a `:repeat`/`:wdN` id without it. Closes `requests/w1-native-notify-1.md`. Local tag `sparky-contracts-v1.4`.
- 2026-10-04 G1 (Erick): live native-reminder E2E (delivery with Electron closed, Done/Snooze from the notification) is deferred to W3-E2E, to run against a functional packaged build (notification permission granted to the real Sparky.app identity). Not a G1 blocker.
- 2026-10-04 G1 approved (Erick). Wave 2 opens; first batch W2-SHELL, W2-MEMORIES, W2-MINDS.
- 2026-10-04 Contract v1.5 (Erick approved): `Settings.calendar_week_start = 23` ("mon" default | "sun", key `calendar.weekStart`) and `Occurrence.tz = 9` (IANA zone of the schedule, filled by ListOccurrences). Additive, `buf breaking` clean, baseline refreshed. Closes requests w2-calendar-2 and w2-review-3. Local tag `sparky-contracts-v1.5`.
