# fx-mcp status

- Result: done within assigned ownership; the schedule/place revision bump remains assigned to fx-core.
- Branch: `sparky/fx-mcp`, based on `sparky-next` at `0ef2418`.
- Last implementation commit: `dbb0f04` (the status is committed separately).
- Reviews: integration `.orchestra/runs/rv-mcp/last.txt` and the first finding of `.orchestra/runs/rv-core/last.txt`.
- Every reported defect was verified against this checkout before editing. No finding was classified as not-a-bug.

## Findings

References below identify the original review locations, before these changes.

| Finding | State | Verification and regression evidence |
| --- | --- | --- |
| RotateToken leaves the mounted guard stale (`compositions/core/agents.go:53`, `compositions/core/mcp.go:85`; also rv-core finding 1) | fixed | The guard previously captured the initial token digest. Core now supplies the atomically replaced token file as its live authorization source, including the fallback mount. `TestRotateTokenImmediatelyChangesMountedGuard` rotates three times and concurrently verifies immediate old-token 401 and new-token acceptance on the same running core. |
| Explicit expectedRev <= 0 bypasses remembered revisions (`providers/mcp/session.go:36`) | fixed | Domain revision zero deliberately disables checks. MCP now rejects explicit nonpositive revisions for memory writes, batches, mind writes, attachments, and delete revision maps. `TestSessionRejectsExplicitNonpositiveRevision` and `TestExplicitNonpositiveRevisionsCannotBypassSession`. |
| Search/resources omit session revision tracking (`providers/mcp/tools_read.go:207`) | fixed | Ranked search is hydrated in the core transaction; fallback search observes only returned hits. Resources observe the memory or mind snapshots they return, including today. `TestSearchAndResourcesTrackReturnedRevision`, `TestSearchRegistersRevisionOfHydratedSnapshot`, `TestMindResourceTracksReturnedRevision`, and `TestTodayResourceTracksReturnedMemorySnapshot`. |
| Toggle observes a later UI revision through its post-commit Get (`providers/mcp/session.go:81`) | fixed | Toggle pins a positive expected revision and returns/registers exactly that revision plus the domain's single transactional increment. No post-commit Get remains. Internal title resolution preserves an earlier session read. `TestToggleReturnsOwnRevisionWithoutObservingLaterUIWrite`, `TestToggleToolReturnsAndRegistersRevision`, and `TestToggleTitleLookupDoesNotAdvancePreviouslyReadRevision`. |
| SetTags leaves the response/session at the earlier revision (`providers/mcp/tools_write.go:285`) | fixed | SetTags returns/registers its exact checked revision plus one. `TestEmptyArraysReplaceAndTagsReturnFinalRevision` and `TestTagsNeverObservePostCommitUIRevision` cover follow-up writes and an injected intervening UI write. |
| Schedule/place mutations do not bump Memory.Rev (`providers/mcp/tools_write.go:598`) | skipped (fx-core ownership) | Confirmed: `scheduleRuntime.Set` audits the schedule upsert without updating memory, and `placeService.Set/Clear` audit only place upsert/delete. These Memory.Rev changes are explicitly assigned to fx-core; this branch changes only the MCP actor adapter for Place. |
| File attach accepts symlink directory components (`providers/mcp/files.go:21`) | fixed | Every original path component is checked with Lstat. Traversal remains rejected; paths must be absolute, regular files under the size limit. `TestAttachPathRejectsEverySymlinkComponent` includes directory/file symlinks, a symlink followed by a dot component, traversal, and relative paths. |
| create_memories persists before validating schedule and can partially apply an item (`providers/mcp/tools_write.go:93`) | fixed | Schedule/place/link inputs are validated before persistence. The complete item joins one existing core transaction, and revisions are published to the session only after commit. `TestCreateValidatesBeforePersistenceAndRollsBackEachItem` covers validation and injected stage failures/retries. `TestCreateRollsBackMemorySchedulePlaceAndEarlierLinks` proves rollback of all relations and earlier links, with an independent successful sibling. |
| checklist:[] / tags:[] do not replace relations with empty (`providers/mcp/tools_write.go:208`) | fixed | A nil slice means omitted; a present empty array replaces the relation. `TestEmptyArraysReplaceAndTagsReturnFinalRevision` checks both omission and empty replacement. |
| Place audit receives the UI actor instead of the MCP actor (`providers/mcp/tools_write.go:619`) | fixed | MCP Places explicitly accepts the session actor; the core adapter places it in the existing caller context for Set/Clear. `TestCoreMCPPlaceWritesUseSessionActor` checks real SQLite audit events for both operations. |
| Stdio proxy truncates responses at 4 MiB (`providers/mcp/proxy.go:69`) | fixed | Response limit is separate (8 MiB), reads use limit+1, and overflow becomes a valid JSON-RPC error with the original request ID. `TestProxyLargeResponsesRemainValidJSON` covers 5 MiB JSON/SSE success and oversized JSON/SSE errors. |
| Closed/expired sessions retain observed state (`providers/mcp/server.go:109`) | fixed | The provider waits for each SDK session's termination and removes its revision map. Requests without a transport do not share anonymous state. `TestSessionStateRemovedOnCloseAndExpiration` exercises HTTP DELETE and actual SDK idle expiration. |

## Validation

- PASS: `cd sparky/backend && go vet ./... && go test ./... -race -count=1`.
- PASS: targeted MCP/provider regression suites with `-race -count=1`.
- PASS: `TestCatalogBudgetAndGolden` with `-race`: tools/list is **5964 bytes**, below the stricter 6000-byte test budget. Catalog schemas and golden catalog are unchanged.
- PASS: `pnpm --filter @polterware/sparky-contracts test`: 4 files, 28 tests.
- PASS: `git diff --check`; only owned source/test paths and this requested status file changed.
- Desktop typecheck/tests: not applicable; `apps/sparky-desktop` was not changed.
- Quality review: re-read the complete diff for races, hidden revision advancement, transaction rollback, and authentication bypasses. The injected toggle/tag UI races and the title-lookup regression cover the resulting safeguards.

## Deviations and requests

- No contract changes or request files; no changes to AGENTS.md, generated code, schemas, or other workstream source.
- Existing branch/worktree already matched the requested branch from sparky-next; no additional branch was created.
- Live rotation uses the existing atomic token-file publication as the authorization source rather than introducing a second in-memory credential cache.
- `runtimes/cli/mcp.go` needed no edit: it reads the token and delegates stdio responses to the fixed provider proxy.
- Polterware CLI is unavailable (`command not found`, exit 127). Status, format, card query, and handoff were attempted; Work-card recording could not run.
- All commits are local. Nothing was pushed or tagged.
- Follow-up: fx-core must implement the explicitly excluded schedule/place Memory.Rev bump.
