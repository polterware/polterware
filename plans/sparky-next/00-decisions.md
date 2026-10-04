# Decisions, assumptions, open questions

## Locked by Erick

| ID | Decision | Rationale | Reversal cost |
|----|----------|-----------|---------------|
| D1 | Code lives in the Polterware monorepo (`apps/sparky-desktop`, `sparky/backend`, `packages/features/sparky-*`). The plan lives in `plans/sparky-next/` (AGENTS.md exception, see README) | One workspace, atomic design-system changes, Turbo runs both apps | Medium |
| D2 | Go for the Sparky core, CLI and MCP. Separate Go module; **does not import `polterware/backend/**`** in v1 (copy-adapt patterns) | Light single binary, mature daemon tooling, same platform as Polterware; separate code keeps the apps independent | High |
| D3 | Execution in waves with gates (W0 spikes → W1 backend/DS/shell → W2 features → W3 hardening) | A failing reminder spike must not waste feature work | Low |
| D4 | No users: no importer, no migration, no Sparkle, no coexistence. Swift app = read-only reference | Nobody uses it | None |
| D5 | v1 scope = full list in README (core + calendar + focus + review + attachments + map + FTS + semantic + expanded MCP + CLI/API) | Erick's choice | Medium |
| D6 | Agents run in worktrees, may commit and run everything (builds, tests, dev server, browser) | Erick's choice | None |
| D7 | Package scope stays `@polterware/*`; renaming is the last task (W3-CONVERGE, optional) | Avoids churn in Polterware now | Low |
| D8 | Electron (not Tauri), same shell pattern and versions as `apps/desktop` | Reuse the proven Polterware setup and the Chromium rendering for the shared design system | Medium |
| D9 | Same brand/theme as Polterware (one token set, one dark theme). App-level accent only through `--color-brand` override | "Same family" requirement | Low |

## Assumptions (veto any of these before Wave 0 starts)

| ID | Assumption | Default |
|----|-----------|---------|
| A1 | v1 targets macOS only (the notifier is Swift/macOS). Windows/Linux are future work | macOS 15+ (confirm the minimum in W0-NOTIFIER) |
| A2 | Location data (`places`) is stored and shown on the map, but **geofence arming is not implemented** (same as the current Mac app) | Place has `event` enter/exit stored for the future |
| A3 | Tags are in the core model (the Swift app has them) | Yes |
| A4 | MCP is local only: loopback HTTP + stdio bridge, no OAuth, no pairing, no remote | Yes; remote MCP is out of scope |
| A5 | The Swift "Me" tab becomes **Review** | Yes |
| A6 | Focus timer runs in the renderer; it arms a one-off OS notification for each phase end through the notifier | Yes |
| A7 | Semantic search is **off by default**; it needs an embedding provider the user configures: an **OpenAI-compatible remote endpoint only**. No Ollama, no local model, no model in the app (Erick: too heavy) | Decided |
| A8 | Brute-force cosine search over float32 BLOBs in Go (no `sqlite-vec`) | Yes; revisit only if W0-STORE measures > 50 ms at 20k memories |
| A9 | Map uses free tiles without an API key; provider chosen by W0-MAP. Geocoding (address search) is opt-in and disabled by default because it sends text to a third party | Yes |
| A10 | App display name "Sparky"; bundle ids `com.polterware.sparky`, `com.polterware.sparky.notifier`; URL scheme `sparky://`; data dir `~/Library/Application Support/Sparky/` | Yes |
| A11 | App icon = `/Users/erickpatrickbarcelos/codes/sparky-workspace/sparky/new.icon` (8-point star, dithered gradient, dark tile) | Yes; W1-ASSETS exports the variants |
| A12 | macOS code signing: Polterware ships with `identity: null` (unsigned). Sparky starts the same; W3-PACKAGING decides Developer ID. The notifier must work ad-hoc signed (W0-NOTIFIER proves it) | Yes |

## Open questions (resolved by a spike, then recorded here)

| ID | Question | Resolved by | Deadline |
|----|----------|-------------|----------|
| O1 | Is `SMAppService.agent` (launchd, KeepAlive) or `.loginItem` the right helper registration? Is the pending-notification limit on macOS 26 really 64? Does it work ad-hoc signed? | W0-NOTIFIER | G0 |
| O2 | Official Go MCP SDK vs hand-rolled JSON-RPC (Polterware's MCP is hand-rolled) | W0-MCP | G0 |
| O3 | Does `modernc.org/sqlite` ship FTS5 and the commit/preupdate hooks we need? Brute-force vector search latency at 10k / 50k rows | W0-STORE | G0 |
| O4 | Map provider and library: MapLibre GL + OpenFreeMap vs Leaflet + raster vs PMTiles; terms of use; CSP | W0-MAP | G0 |
| O5 | Which design-system files are Polterware-specific vs shared; exact coupling list | W0-DS-AUDIT | G0 |

Record each resolution as a dated line appended below.

## Resolutions log

(empty — workstreams append: `YYYY-MM-DD O<n>: <decision> (WS, evidence link)`)
