# MCP

MCP is a flagship feature. Goals: correct, fast, light, **cheap in tokens**. Reference for the old surface (read-only):
`/Users/erickpatrickbarcelos/codes/sparky-workspace/sparky-mcp/docs/CONTRACT.md` and `src/mcp.ts`. Polterware's MCP
(`backend/providers/mcp/*`) is a hand-rolled JSON-RPC server; W0-MCP decides whether Sparky uses the official Go SDK instead.

## Server

- Name `sparky`, version = core version. Declares `tools`, `resources`, `prompts` capabilities.
- Transports: **stdio** (`sparky mcp`, proxies to the core) and **streamable HTTP** at `http://127.0.0.1:47600/mcp` (bearer
  `mcp-token`, Host/Origin checks, see [01-architecture.md](01-architecture.md)). Stateless where possible; session ids only
  if the SDK needs them.
- Every tool call is attributed: actor `mcp:<clientInfo.name>` in `activity` and the UI ("changed by Claude Code").
- Writes are **synchronous** and return the resulting entity (no queue, no `get_command_status`).
- Concurrency: tools that mutate an existing entity accept `expectedRev`. If omitted, the server uses the `rev` the **same
  session** last read for that entity (tracked per session); if the entity changed since then it returns a `conflict` error
  with the current summary instead of overwriting. First write without a prior read = last-write-wins.
- Disabled by default? **No** — enabled by default but only reachable on loopback with a token; the UI can switch it off
  (`agents.mcpEnabled`).

## Token-efficiency rules (tests enforce these)

1. Tool descriptions ≤ 300 characters; parameter descriptions ≤ 120. Total `tools/list` payload ≤ 6 KB.
2. Default `limit` 20 (max 100). Always return `total` and `nextCursor`.
3. List results are **summaries**: `{id, title, status, mindId/mind, due, pinned, tags[], schedule:"every Mon,Wed 09:00"|null,
   snippet(≤120 chars), rev}`. Full note/checklist only via `get_memory`.
4. `get_memory` takes `include` (`note`, `checklist`, `schedule`, `place`, `attachments`, `completions`); default
   `note,checklist,schedule`.
5. Responses use compact JSON (no pretty printing, omit null/empty fields) plus a one-line human `text` content; use
   `structuredContent` when the SDK supports it.
6. Batch writes (`create_memories`, `update_memories`, `delete_memories`) accept up to 50 items and return per-item results
   (`ok|error`), so an agent never loops 50 calls.
7. Errors are one actionable sentence ("Mind 'Wrok' not found. Valid: Work, Home, Health").
8. Name-based lookup for Minds/tags everywhere (`mind: "Work"`), with case-insensitive match and a "did you mean" error.
9. Size budgets (golden tests): `list_memories` page of 20 ≤ 3.5 KB; `get_memory` typical ≤ 1.5 KB; `list_minds` for 30 Minds ≤ 1.2 KB.

## Tools (18: 9 read, 9 write)

Read
| Tool | Params | Notes |
|------|--------|-------|
| `get_current_time` | – | now, tz, weekday, ISO week |
| `list_minds` | `parent?` | compact tree `name(id) > children` |
| `list_tags` | – | name, color, count |
| `list_memories` | `mind?`, `includeDescendants?`, `tag?`, `status?`, `pinned?`, `dueFrom?`, `dueTo?`, `updatedFrom?`, `updatedTo?`, `hasSchedule?`, `query?` (FTS), `order?`, `limit?`, `cursor?` | summaries |
| `search_memories` | `query`, `mind?`, `tag?`, `limit?` | FTS5 ranked, with `score`, `matchedIn` and highlighted `snippet`; text search only (an agent may rerank the results) |
| `get_memory` | `id`, `include?` | full entity |
| `list_occurrences` | `from`, `to`, `mind?`, `includeCompleted?`, `limit?` | "what is on my plate": occurrences with completion state, overdue flagged |
| `get_review` | `period?` (`week` default, `month`, `year`) | totals, streak, per-day counts, focus minutes |
| `list_activity` | `limit?`, `actor?` | recent changes (who/what) |

Write
| Tool | Params | Notes |
|------|--------|-------|
| `create_memories` | `items[]`: `title`, `note?`, `mind?`, `tags?`, `due?`, `pinned?`, `checklist?:[string \| {title,detail}]`, `schedule?`, `place?`, `links?:[{url,title?}]` | batch ≤ 50 |
| `update_memories` | `items[]`: `id`, `expectedRev?`, any creatable field as patch (`null` clears) | `checklist` replaces; `tags` replaces; `addTags`/`removeTags` available |
| `set_memory_status` | `id`, `status`, `occurrence?` | `occurrence` completes one occurrence of a recurring memory |
| `toggle_check_item` | `memoryId`, `itemId` or `itemTitle`, `occurrence?` | |
| `delete_memories` | `ids[]`, `confirm: true` | refuses without `confirm` and says what would be deleted |
| `create_mind` / `update_mind` | `name`, `parent?`, `color?`, `icon?`, `sortOrder?` | |
| `delete_mind` | `id` or `name`, `confirm: true` | description states: children are deleted, memories move to Inbox |
| `add_attachments` | `memoryId`, `links?[]`, `files?[{path}]` | `files` only when `agents.allowFileAttach` is on; otherwise error explains where to enable it |

Schedule input (shared by create/update): `{at: ISO|"YYYY-MM-DD" (all-day), tz?, repeat?: {freq, every?, weekdays?: ["mon",…],
until?|count?}, focus?: {workMin, shortBreakMin, longBreakMin, cycles, autoContinue}}`. The server validates with
`domains/schedule` and returns the next 3 occurrences in the result so the agent can confirm intent.

Not exposed: starting/stopping a focus session (the timer lives in the window; honest limitation in the tool description of
`get_review`), reading attachment bytes (metadata only; `read_attachment` for ≤ 64 KB text files is **optional P2**), Settings.

## Resources and prompts (P2, only if W0-MCP shows the SDK makes it cheap)

Resources: `sparky://minds`, `sparky://memory/{id}`, `sparky://today`. Prompts: `plan_day`, `weekly_review`, `capture` (turn free
text into `create_memories` args). Keep each prompt ≤ 1 KB.

## Client setup (the UI's Agents > Access page generates these)

```bash
# Claude Code — stdio (recommended)
claude mcp add sparky -- /Applications/Sparky.app/Contents/Resources/sparky mcp
# Claude Code — HTTP
claude mcp add --transport http sparky http://127.0.0.1:47600/mcp --header "Authorization: Bearer <mcp-token>"
```
Cursor/Claude Desktop JSON snippets: same two shapes (`command` + `args`, or `url` + `headers`). The page shows the current
port/token and a "Test connection" button that performs `initialize` + `tools/list` and reports honestly.

## Tests (W1-MCP owns)

- Contract tests per tool against in-memory services (golden JSON transcripts under `tests/mcp/golden/`).
- Size-budget tests from the token rules above.
- Security tests: wrong/missing token, bad Host, bad Origin, oversized body, path traversal in `add_attachments`, `confirm` missing.
- Smoke with the MCP Inspector (`npx @modelcontextprotocol/inspector`) documented in the WS status file; **plus** a real
  `claude mcp add` run against a dev core, recording the transcript.
- Concurrency test: UI-style write between an agent's read and write yields `conflict`, not a silent overwrite.
