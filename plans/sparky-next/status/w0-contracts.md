# w0-contracts status

- Result: done
- Branch / last commit: `sparky/w0-contracts`, tag `sparky-contracts-v1` on this status commit (HEAD)
- Acceptance criteria:
  - `cd sparky/schemas && buf lint && buf generate` clean → **pass** (`bash tooling/verify-sparky.sh`, which runs `buf lint` and `buf breaking` against `.buf/baseline.binpb`)
  - Go and TypeScript builds pass → **pass** (`go test ./...` and `go vet ./...` in `sparky/backend`; `pnpm --filter @polterware/sparky-contracts typecheck`)
  - SQL baseline applies and `PRAGMA foreign_key_check` is clean → **pass** (`packages/sparky-contracts/src/baseline.test.mjs`)
  - Every RPC listed in doc 02 exists → **pass** (`packages/sparky-contracts/src/rpc-surface.test.mjs`)
  - Each MCP tool schema accepts a sample call → **pass** (`packages/sparky-contracts/src/mcp-schemas.test.mjs`, 18 tools)
  - Verify command → **pass** (`bash tooling/verify-sparky.sh`)
- Deviations from the plan (and why):
  - `buf.yaml` excepts `RPC_REQUEST_STANDARD_NAME` and `RPC_RESPONSE_STANDARD_NAME`. Doc 02 uses the same short method names (`List`, `Get`, `Create`) on several services. Request messages are prefixed (`ListMindsRequest`) so `RPC_REQUEST_RESPONSE_UNIQUE` still holds.
  - Added settings key `links.fetchTitles` (default false). Doc 05 requires it for opt-in link titles; doc 02's typed-key list omitted it. Frozen now so Wave 1 does not need a contract change.
  - In-memory fakes live under `sparky/backend/domains/<pkg>/fake` and panic with `not implemented`. The deliverable asks for signature skeletons; doc 02 assigns real fakes to W1-DOMAIN. Owns listed `ports.go` and `types.go`; the fake packages are the skeleton half of that deliverable.
  - Extra port packages beyond the examples in doc 02: `fault`, `actor`, `clock`, `ids`, `tx`, `place`, `reminder`, `maintenance`, `agentaccess`. Every RPC in doc 02 has a Go service interface.
  - MCP `tools/list` payload (compact JSON of `{name, description, inputSchema}` for the 18 tools) is **6124 bytes** (limit 6144). Nested schedule, place, and checklist objects are named in parameter descriptions (≤ 120 characters). The strict shapes are `schedule.schema.json` and `place.schema.json` and are not part of that payload. Inlining them made the payload 11221 bytes.
  - Focus defaults 25 / 5 / 15 minutes and 4 cycles match the Swift reference (`FocusSettings.swift`). Validation bounds are 1–60 minutes and 1–12 cycles.
  - `map.online` defaults to false until the first-run disclosure turns it on.
  - `HealthService.Check` keeps `build` = 1 and `version` = 2 and adds `started_at`, `db_path`, `mcp`, and `notifier`. `.buf/baseline.binpb` was regenerated because this commit is the v1 freeze.
  - Repository `Update` treats `expectedRev == 0` as "do not check" so an MCP first write can last-write-wins. Rev still starts at 1.
  - The change stream `op` is `create`, `update`, or `delete`. Activity rows also use `complete` and `reopen`; those writes still emit an entity `update`.
- Requests filed (plans/sparky-next/requests/...): none
- Follow-ups for other WS:
  - **W1-DOMAIN**: replace the panicking fakes for memory, mind, tag, focus, review, activity, and settings. Implement the `Service` interfaces in those packages. Review mood thresholds are constants in `domains/review`.
  - **W1-RECURRENCE**: implement `Rule.Validate`, `Next`, `At`, `Count`, and `Describe`, plus `schedule.Planner`. Load `domains/schedule/testdata/recurrence_vectors.json` (14 cases). Do not rename `Rule` fields.
  - **W1-STORE**: implement every `Repository` / `Index` / `Vectors` / `Archiver` against SQLite. `0001_baseline.sql` is read-only and identical to doc 02. WAL, `foreign_keys`, `busy_timeout`, and `synchronous` are connection pragmas, not part of the migration. Join the `tx.Manager` transaction stored in `context.Context`.
  - **W1-SEARCH / W1-ATTACH**: implement `search.Index`, `search.Vectors`, `search.Embedder`, and `attachments.Store`. Size limits are constants on `attachments` (photo 25 MB, audio 100 MB, file 50 MB, hard max 500 MB).
  - **W1-CORE**: `place`, `reminder`, `agentaccess`, and `maintenance` services are not in the W1-DOMAIN owns list. Implement those handlers here (or agree a split). Map Connect codes from `fault.Error`: `not_found`, `conflict` and `cycle` and `precondition` → `FAILED_PRECONDITION` with `PreconditionDetail`, `invalid` → `INVALID_ARGUMENT` with `BadRequest`, `exhausted` → `RESOURCE_EXHAUSTED`, `unavailable` → `UNAVAILABLE`, `already_exists` → `ALREADY_EXISTS`. Attach the current entity JSON on `rev_conflict`. `MemoryService.Get` is composed here from memory, schedule, place, and attachments using `MemoryInclude`.
  - **W1-MCP**: read `providers/mcp/schemas/*.json`. Do not edit them. Publish `tools/list` as `name`, `description`, and `inputSchema` only (no `title`, `$schema`, or `outputSchema` in that payload; there are 20 bytes of room under 6 KB). Validate `schedule` and `place` with the sibling schema files and `domains/schedule`. Batch tools accept up to 50 items even though `maxItems` was left off the wire schema to fit the budget.

## Contract summary

Package `sparky.v1`. Connect procedure names are `/sparky.v1.<Service>/<Method>`. Timestamps are `google.protobuf.Timestamp` (UTC). Calendar dates in filters and all-day inputs are `YYYY-MM-DD` strings. IDs are ULID strings. `rev` is `int64` and starts at 1. JSON names are camelCase.

Nullable SQL columns use proto3 `optional` or message presence. Empty `freq` is a one-shot. `weekday_mask` bit 0 is Sunday (`WeekdayMonday = 2`, `WeekdayWednesday = 8`). `end_type` is `never`, `until`, or `count`.

### Services

| Service | Methods |
|---------|---------|
| HealthService | Check |
| MindService | List, Get, Create, Update, Delete, Reorder |
| MemoryService | List, Get, Create, Update, Delete, SetStatus, SetPinned, Reorder, ToggleCheckItem, ReplaceChecklist, SetTags, BulkAction |
| TagService | List, Create, Update, Delete, Merge |
| ScheduleService | Set, Clear, Preview, ListOccurrences, CompleteOccurrence, ReopenOccurrence, SetFocusRecipe |
| PlaceService | Set, Clear, ListInBounds |
| AttachmentService | ImportFromPath, AddLink, List, Delete, Reorder, OpenInfo |
| FocusService | GetDefaults, SetDefaults, RecordSession, ListSessions |
| ReviewService | Get |
| SearchService | Search, Status, Reindex, ConfigureEmbedding, TestEmbedding |
| ReminderService | Plan, Ack, Snooze, ArmOneOff, CancelOneOff, Heartbeat, Status |
| ActivityService | List |
| SettingsService | Get, Set |
| AgentAccessService | GetInfo, RotateToken, SetEnabled, SetAllowFileAttach |
| ChangeService | Watch (server stream; first message has `snapshot = true`) |
| MaintenanceService | Export, Backup, Doctor |

`MemoryService.List` filters: `mind_id`, `include_descendants`, `inbox_only`, `status`, `pinned`, `tag_ids`, `due_from`, `due_to`, `updated_from`, `updated_to`, `has_schedule`, `has_place`, `query`, `order` (`updated`/`due`/`manual`/`created`), `limit` (default 50, max 500), `cursor`. The response is summaries plus `next_cursor` and `total`.

`Memory.Get` include flags: `note`, `checklist`, `schedule`, `place`, `attachments`, `completions`, `tags`.

Bulk `action`: `complete`, `reopen`, `pin`, `unpin`, `move`, `tag`, `untag`, `delete`.

Update patches use `optional` fields plus `clear_*` booleans. A set optional writes the value. `clear_mind_id`, `clear_priority`, `clear_due_at`, `clear_parent`, `clear_color`, and `clear_icon` write NULL.

### Errors

`sparky.v1.BadRequest` / `FieldViolation` for `INVALID_ARGUMENT`. `sparky.v1.PreconditionDetail` for `FAILED_PRECONDITION` (`reason` is `rev_conflict`, `mind_cycle`, or `schedule_in_the_past`). On `rev_conflict`, `current_json` is the current entity and `current_rev` is its rev.

### Settings keys

`appearance.theme` (`system`/`dark`/`light`, default `dark`), `notifications.sound`, `notifications.allDayTime` (default `09:00`), `notifications.snoozeMinutes` (default 10), `focus.work`, `focus.shortBreak`, `focus.longBreak`, `focus.cycles`, `focus.autoContinue`, `focus.sounds`, `focus.notifications`, `map.online` (default false), `map.provider`, `search.semantic.enabled` (default false), `search.semantic.provider`, `search.semantic.model`, `search.semantic.baseUrl`, `agents.mcpPort` (default 47600), `agents.mcpEnabled` (default true), `agents.allowFileAttach` (default false), `app.keepRunningInBackground` (default true), `links.fetchTitles` (default false). Embedding API keys are not stored.

`Get` returns the effective document with defaults filled in. `Set` writes only the listed keys.

### Domain ports

Repositories take `context.Context` and join the `tx.Manager` transaction. `clock.Clock` and `ids.Generator` are the time and ULID ports. `fault.Error` is the domain error. `actor.Actor.Raw` is `ui`, `cli`, `mcp:<clientName>`, or `system`.

`schedule.Rule` is the pure recurrence value. Methods are intentionally not declared yet (W1-RECURRENCE adds the bodies). `ReminderItem.ID` is `rem:<memoryID>:<occurrenceUnixMs>` or `one:<id>`. Categories: `reminder`, `reminder-focus`, `focus-phase`.

### MCP

18 tools in `sparky/backend/providers/mcp/schemas/<tool>.json` (`inputSchema` and `outputSchema`). Read: `get_current_time`, `list_minds`, `list_tags`, `list_memories`, `search_memories`, `get_memory`, `list_occurrences`, `get_review`, `list_activity`. Write: `create_memories`, `update_memories`, `set_memory_status`, `toggle_check_item`, `delete_memories`, `create_mind`, `update_mind`, `delete_mind`, `add_attachments`. `null` clears on update. `confirm: true` is required to delete. `delete_mind` says child minds are deleted and memories move to Inbox. `get_review` says it cannot start a timer.

### SQL

`sparky/backend/providers/sqlite/migrations/0001_baseline.sql` is byte-identical to the `sql` fence in `plans/sparky-next/02-data-and-contracts.md`. Applying it to an empty database succeeds. A mind delete sets `memories.mind_id` to NULL. `PRAGMA foreign_key_check` is empty.

### Recurrence vectors

`sparky/backend/domains/schedule/testdata/recurrence_vectors.json` has cases 1–14. Case 13 `expect` is null (`Next` is strictly after the anchor). Case 14 is all-day with notification time `09:00`. Times are RFC 3339 with milliseconds. Weekly mask 10 is Monday+Wednesday.
