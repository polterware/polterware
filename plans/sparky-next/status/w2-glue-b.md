# w2-glue-b status
- Result: done
- Branch / last commit: `sparky/w2-glue-b` / implementation `df4850f` (`sparky(w2-glue-b): add Commander command sources with plain snippets`); this status commit follows
- Acceptance criteria:
  - Serve `/characters/<file>` in dev and packaged, allow-list only, no directory listing, no `file://` exposure, CSP not widened → **pass**. `characters-protocol.test.ts` (`allows only the named PNG and WebP files`, symlink 404); Vite public copy at `src/renderer/public/characters/`; `electron-builder.yml` extraResources `resources/characters` → `characters`; Main intercepts `file:///characters/<allowlist>` like `/sounds/` (`index.ts` `matchCharacterPath`). CSP still `img-src 'self' data: blob: sparky-file:` / `connect-src 'self'` by default.
  - `sparky://map` in `parseDeepLink` and shell `deepLinkPath` (`/map`) with tests → **pass**. `navigation-policy.test.ts` `routes the Map command to /map`; `route.test.ts` `sparky://map` → `/map`.
  - New `window.sparky.app` methods on `APP_METHODS` / `SparkyBridge` / preload / trusted-sender IPC, with validation and negative tests, no login item → **pass**. See signatures below. Tests: `ipc.test.ts` input validation (incl. `openLog("/tmp/secret.log")`, `testMcpConnection({ token })`, `setReduceTransparency("true")`); `preload/index.test.ts` channel mapping; `window-security.test.ts` `assertTrustedSender` + `setLoginItem` absent; `app-log.test.ts`, `appearance.test.ts`, `mcp-probe.test.ts`.
  - CSP: no loopback `connect-src`; Nominatim only while `map.geocoding` is true; re-evaluate on settings change → **pass**. `navigation-policy.test.ts` `adds Nominatim to connect-src only while geocoding is enabled`; `refreshMapPolicy` reads `mapOnline` and `mapGeocoding` and reloads when either changes.
  - Connect `invalid_argument` field-violation text in renderer `Error.message`, no stacks/paths → **pass**. `ipc-error.test.ts`; `ipc.test.ts` `exposes Connect invalid_argument field violations on Error.message` (`must be HH:MM`).
  - `commandSources?: CommandSourceDef[]` on `SparkyFeature`; Commander uses registered sources (debounced, stale-cancelled) with existing keyboard model; snippets as Solid text nodes; built-in search fallback; search feature untouched → **pass**. `shell.test.tsx` `renders registered Commander sources and strips FTS markers`; fallback test still opens `Buy milk` via SearchService; `snippet.test.ts`; `commander.tsx` has no `innerHTML`.
- Verification: `pnpm --filter @polterware/sparky-desktop test` pass (129); `pnpm --filter @polterware/feature-sparky-shell test` pass (97); `pnpm design:lint` pass; whole-repo `pnpm typecheck` pass; `git diff --check` pass. Browser and e2e not run.
- Deviations from the plan (and why):
  - Login item methods were not added (2026-10-04 decision: unavailable under ad-hoc signing).
  - Vibrancy-on window fill does not use `#00000000` (opacity-lint allowlist is Polterware Desktop `window.ts` only); opaque `#070707` is applied when reduce-transparency is on.
  - `testMcpConnection` returns `{ ok, toolCount?, error? }` (token never included). Settings still owns UI copy.
- Requests filed (plans/sparky-next/requests/...): none
- Follow-ups for other WS:
  - **W2-REVIEW** — load allow-listed files from `/characters/<file>` (no `file://`). Names: `me-streak-{sad,hopeful,happy,euphoric}{,@2x,@3x}.png`, `me-completed-{sad,hopeful,happy,euphoric}{,@2x,@3x}.png`, `sparky-character.webp`.
  - **W2-MAP** — `app.openDeepLink("sparky://map")` now parses. Geocoding fetch to Nominatim is allowed in CSP only after `settings.set` turns `map.geocoding` on (Main reloads the renderer).
  - **W2-SETTINGS / W2-SEARCH-UI** — call these exact `window.sparky.app` signatures (already on the typed bridge; login item is not present):
    - `openLog(): Promise<void>` — Main reveals `~/Library/Logs/Sparky/sparky.log` (size-capped rotating supervisor/core events; never token or file content) via `shell.showItemInFolder`.
    - `openDataFolder(): Promise<void>` — reveals the Main-resolved Sparky data directory (not a renderer path).
    - `getReduceTransparency(): Promise<boolean>`
    - `setReduceTransparency(enabled: boolean): Promise<boolean>` — persisted in Main `userData/appearance-preferences.json`; applies live to open windows (disables macOS vibrancy, opaque fallback).
    - `testMcpConnection(): Promise<{ ok: boolean; toolCount?: number; error?: string }>` — Main reads port/token via `AgentAccessService.GetInfo`, runs `initialize` + `tools/list` against `http://127.0.0.1:<port>/mcp`; returns only the result.
  - **W2-SEARCH-UI** — register `commandSources` on the Search feature:
    ```ts
    commandSources?: CommandSourceDef[]
    // CommandSourceDef = { id: string; search: (query: string) => Promise<{ hits: { id: string; label: string; snippet?: string; run: () => void }[] }> }
    ```
    Commander queries sources instead of `SearchService` when any source is registered. Pass plain `snippet` strings; the shell strips FTS `<b>` markers and renders text nodes.
  - Connect `invalid_argument` `Error.message` is the joined field-violation reasons (e.g. `must be HH:MM`; `must be mon or sun`).
  - W3-E2E: packaged `/characters/`, `sparky://map`, Settings Advanced/Data/MCP test, Nominatim CSP on/off, Commander source hits.
