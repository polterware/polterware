# Search, attachments, map

## Full-text search (FTS5)

- Table `memory_fts` (see [02-data-and-contracts.md](02-data-and-contracts.md)). One row per memory: `title`, `note` (Markdown
  stripped to text), `checklist` (item titles + details joined by newline), `tags` (space-joined), `mind` (Mind name path).
- Maintained **inside the same transaction** as the write by `providers/sqlite` (service calls `Index.Upsert(memory)` /
  `Index.Delete(id)`); a `Reindex` RPC rebuilds everything (used after upgrades and by `sparky doctor --fix`).
- Query: user text → tokenizer-safe FTS query (quote terms, add `*` to the last term for prefix match, drop FTS operators unless
  the query is wrapped in quotes). Ranking: `bm25(memory_fts, 10.0, 1.0, 2.0, 3.0, 1.5)` (title weighted highest). Return
  `snippet()` highlights for the UI.
- Acceptance: p95 < 30 ms at 10k memories; diacritics-insensitive (`cafe` finds `café`); Portuguese and English fixtures.

## Semantic search (opt-in)

- **Off by default.** No model ships in the app, nothing is sent anywhere until the user configures a provider
  ("Execution honesty": the Search settings page shows `not configured | indexing n/m | ready | provider unreachable`).
- Provider interface (`providers/embeddings`): `Embed(ctx, []string) ([][]float32, error)`, `Dims()`, `Name()`.
  Implementations: `ollama` (default URL `http://127.0.0.1:11434`, model chosen by the user, e.g. `nomic-embed-text`) and
  `openai-compatible` (base URL + API key stored in the macOS Keychain via Electron Main `safeStorage`, passed to the core
  at runtime, **never** written to SQLite). A `test` RPC embeds a fixed sentence and reports dims/latency.
- Text embedded per memory: `title + "\n" + note(plain, first 2 000 chars) + "\n" + checklist titles`. Stored in
  `embeddings(vector float32 LE, L2-normalized, text_hash)`. A background worker (bounded concurrency 2, batch 16, retry with
  backoff) embeds queued memories; `text_hash` prevents re-embedding unchanged content; changing the model drops and rebuilds.
- Search: embed the query, **brute-force dot product in Go** over all vectors (A8), top-K with a min-heap. Budget: ≤ 50 ms at 20k
  memories × 768 dims (W0-STORE measures; if it fails, switch to `sqlite-vec` via a WASM build or an ANN index and record O3).
- **Hybrid** = Reciprocal Rank Fusion of FTS and semantic lists (`k = 60`), filters applied before fusion. If semantic is not
  ready, `hybrid` silently degrades to `fts` and the response says `modeUsed: "fts"`.
- Privacy: memory text leaves the machine only for a non-loopback `openai-compatible` provider, and the settings page states it.

## Attachments

Kinds: `photo`, `audio`, `file`, `link`. Files live under `<data>/attachments/<memory_id>/<attachment_id>.<ext>`; the DB row
holds metadata and `sha256`.

- Import path: renderer asks Main to open a native dialog or receives a drag-drop; Main passes the **absolute source path** to
  `AttachmentService.ImportFromPath`; the core copies (never moves) the file, computes sha256 + size + mime (by content
  sniffing, extension as hint), enforces limits (photo ≤ 25 MB, audio ≤ 100 MB, file ≤ 50 MB; configurable, hard max 500 MB),
  and rejects symlinks and non-regular files.
- Audio recording: renderer uses `MediaRecorder`, streams chunks to Main, Main writes a temp file then imports it. Microphone
  permission is requested only on first use (`systemPreferences.askForMediaAccess('microphone')`); handle denied/unavailable.
- Photos: previews generated in the renderer (`createImageBitmap`, canvas) from `OpenInfo` paths via a restricted
  `sparky-file://` protocol handler in Main that only serves files under `<data>/attachments`. No `file://` access from the renderer.
- Links: `AddLink(url, title?)`. URL must be `http(s)`. Title fetching (page `<title>`) is an **opt-in** setting
  (`links.fetchTitles`, default off) because it contacts third-party sites; the fetch runs in the core with a 5 s timeout, 1 MB cap,
  no cookies, and refuses private/loopback addresses (SSRF guard).
- Deleting a memory/attachment removes files (best effort) in the same service call; a startup sweep removes orphan files and
  rows without files (reported by `doctor`).
- Export: `MaintenanceService.Export` writes `sparky-export-<date>.zip` with `data.json` + `attachments/`; `Backup` writes a
  `VACUUM INTO` copy (never copy the live `-wal` file).

## Map (free, no API key)

Purpose: show memories that have a `place`, and let the user pick a place. **No geofencing** (A2).

- Candidates (W0-MAP chooses and records O4, with terms-of-use evidence): **MapLibre GL JS + OpenFreeMap vector tiles** (no
  key; fits the dark theme with a custom style) as the preferred option; **Leaflet + OSM raster tiles** as the simple
  fallback (note: the OSM tile policy forbids heavy use and requires attribution and a valid identifying User-Agent/Referer,
  which an Electron app must configure); **PMTiles** (Protomaps) bundled or downloaded per region as the fully offline option.
- Implementation shape: a framework-agnostic `MapView` wrapper in `packages/frontend/map` (new package, built by W2-MAP) with
  props `center, zoom, markers[{id, lat, lng, radius?}], selectedId, onSelect, onPick({lat,lng})`; styles use design tokens.
- Online tiles are a **setting** (`map.online`, default on after the first-run disclosure: "Map tiles are downloaded from
  <provider>; your viewport is visible to them"). When off or offline the map shows an honest empty state with the list of places.
- CSP allows `connect-src`/`img-src` only for the chosen tile host(s) and only while the setting is on (Main sets the CSP).
- Address search (geocoding) is **opt-in** and disabled by default (provider TBD by W0-MAP; Nominatim requires ≤ 1 req/s, a
  real User-Agent and attribution; no autocomplete-as-you-type against it). Without it, users drop a pin or enter lat/lng.
- Place editor: drop pin, drag, radius slider (25–5000 m), event enter/exit, name. Same component inside the memory detail and
  on the Map route.
- Marker clustering when > 200 markers (MapLibre built-in or `supercluster`).
