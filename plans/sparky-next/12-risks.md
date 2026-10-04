# Risks and fallbacks

| ID | Risk | Likelihood / impact | Detected by | Mitigation / fallback |
|----|------|---------------------|-------------|------------------------|
| R1 | Notifier cannot run reliably ad-hoc/unsigned, or macOS blocks the helper identity/permission prompt | Medium / **High** (core promise) | W0-NOTIFIER | Try agent vs loginItem; try Developer ID (ask Erick); **Plan B** in 03 (Electron-posted notifications while Sparky runs in the background) and say so in the UI and docs |
| R2 | macOS pending-notification limit is lower than the planned budget | Medium / Medium | W0-NOTIFIER | Set `Budget` to the measured limit; keep nearest-first global planning; re-plan on wake/delivery |
| R3 | Official Go MCP SDK lacks something needed (streamable HTTP details, structured output, progress) | Low–Medium / Medium | W0-MCP | Hand-rolled JSON-RPC like Polterware (known to work); isolate behind `providers/mcp` interfaces |
| R4 | `modernc.org/sqlite` lacks FTS5 or hook APIs | Low / Medium | W0-STORE (passed) | `ncruces/go-sqlite3` (WASM) |
| R5 | Design-system refactor regresses Polterware | Medium / **High** | W1-DS non-regression | Provider-with-defaults (no behavior change), screenshot diffs, small commits, revert-friendly commits, run Polterware tests every commit |
| R6 | Map provider terms forbid app use or tiles unreliable | Medium / Low–Medium | W0-MAP | Leaflet + self-hosted/bundled PMTiles; or ship the map as an optional online feature with the disclosure |
| R7 | Recurrence edge cases (DST, month ends) wrong | Medium / Medium | W1-RECURRENCE vectors + fuzz | Vectors in 03 are normative; compare with Swift for simple cases; add cases when bugs appear |
| R8 | Parallel agents collide in shared files or contracts | Medium / Medium | Merge conflicts, G-gates | Scaffold creates every stub and alias; requests flow for shared files; contracts frozen at a tag |
| R9 | Electron auto-update needs signing on macOS | High / Low–Medium | W3-PACKAGING | Document manual updates; ask Erick about Developer ID; keep updater disabled until signing exists |
| R10 | Scope is large (full feature parity + new backend features) | High / Medium | Wave gates | Gates make progress shippable per wave; features inside Wave 2 are independent and can slip individually without blocking release (decide with Erick) |
| R11 | `go 1.27` / `TypeScript 7` / `pnpm 12` toolchains drift between agent machines | Low / Low | verify scripts | Pin in `go.mod`/`package.json` (`packageManager`), CI uses the same versions |
| R12 | Window closed + core alive confuses users about "is Sparky running?" | Medium / Low | W3-POLISH | Setting + clear status chip + menu item "Quit Sparky completely" that stops the core and notifier registration prompt |
| R13 | Privacy: map tiles leak the viewport | Medium / Medium | W2-MAP | `map.online` setting, explicit disclosure, network-isolation test (no requests while disabled) |

## Plan-B switch procedure (reminders)

If G0 fails on O1: (1) record the evidence in [00-decisions.md](00-decisions.md); (2) promote 03 §Fallback to the main path;
(3) W1-NOTIFIER is replaced by **W1-DELIVERY-ELECTRON** (Electron Main timer + `Notification` fed by `ReminderService.Plan`,
catch-up on wake with a "missed reminders" list in the UI); (4) UI status chip wording changes to "Reminders need Sparky running";
(5) everything else is unchanged because the core planner and RPC surface are identical.
