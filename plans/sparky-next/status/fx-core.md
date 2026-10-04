# fx-core status

- Result: done
- Branch / last code commit: `sparky/fx-core` / `edcd814` (the final commit records this status)
- Base: `sparky-next` at `0ef2418a864b74da6810e57338164b1cfd766577`; the existing worktree was clean and already on the requested branch.
- Review input: `/Users/erickpatrickbarcelos/codes/polterware-total/polterware-wt/integration/.orchestra/runs/rv-core/last.txt`.
- Every assigned finding was checked against the implementation before changing it. All assigned defects were confirmed; none was classified as not-a-bug.

## Findings

Paths below are relative to `sparky/backend`. Review line numbers identify the original finding; implementation locations identify the corrected code.

| Finding | Disposition | Implementation and regression evidence |
| --- | --- | --- |
| `platform/db/tx.go:53`: rollback of B can discard A's committed Watch changes | fixed | `platform/db/tx.go:43` serializes writer transactions through harvesting and publication with a context-aware gate shared by the hook buffer. `TestWriterWaitsForCommitPublication` covers commit/rollback separation, publication ordering, and nested transactions. `TestWaitingWriterHonorsCancellation` prevents blocked cancelled writers. |
| `compositions/core/search_attachment.go:37`: open Rows exhaust the reader pool during concurrent Reindex | fixed | `compositions/core/search_attachment.go:58` collects IDs and closes Rows before projecting documents. `TestSearchDocumentsReleaseRowsBeforeProjection` restricts the pool to one reader; `TestConcurrentReindexDoesNotStarveConnections` runs eight rebuilds concurrently. |
| `compositions/core/search_attachment.go:20`: stale documents overwrite a newer committed update | fixed | `compositions/core/search_attachment.go:33` wraps collection and rebuild in one writer transaction; `providers/sqlite/document.go:25` joins that transaction for the rebuild. `TestReindexCannotOverwriteConcurrentMemoryUpdate` pauses after collection and verifies that the competing update cannot commit until rebuild finishes and that the final FTS document is current. |
| `compositions/core/schedules.go:205`: SetFocusRecipe reads the schedule outside its transaction | fixed | `compositions/core/schedules.go:207` reads and writes inside `Audit.Write`. `TestFocusRecipeReadsScheduleInsideTransaction` interleaves a schedule change before the recipe's transaction begins and verifies that the newer fire time survives. |
| `compositions/core/schedules.go:135`: schedule mutations preserve Memory.Rev | fixed | Set, Clear, and SetFocusRecipe bump Memory.Rev and updated_at through `bumpMemoryRevision` in the same audit transaction. `TestScheduleAndPlaceMutationsInvalidateMemoryRevision` checks all three paths and rejects an earlier expectedRev. `TestScheduleAndPlaceRevisionRollbackWithAudit` checks atomic rollback. |
| Additional assigned scope: place mutations preserve Memory.Rev | fixed | `compositions/core/places.go:20` and `:52` bump Memory.Rev and updated_at in the Set/Clear audit transaction. The same revision and audit-rollback regression tests cover place mutations. |
| `providers/sqlite/mind.go:83`: memories moved to Inbox retain rev/updated_at | fixed | `providers/sqlite/mind.go:75` collects memories across the deleted tree and explicitly moves them to Inbox with rev+1 and updated_at before deleting the minds. `TestMindDeleteMovesAndReindexesMemoriesAtomically` covers parent/child memories, unaffected Inbox memories, stale expectedRev, and rollback. The existing domain Mind-delete audit remains in the same transaction and its commit/rollback is asserted. |
| `providers/sqlite/mind.go:83`: deleted Mind names remain in FTS | fixed | The same delete transaction reindexes all surviving moved memories before commit. `TestMindDeleteMovesAndReindexesMemoriesAtomically` searches the removed ancestor name before/after deletion and verifies FTS restoration on audit failure. |
| `providers/sqlite/tag.go:111`: tag merge preserves affected Memory.Rev | fixed | `providers/sqlite/tag.go:100` advances rev and updated_at once for each source-tagged memory during Retarget, including memories already carrying the target tag. `TestTagMergeInvalidatesAffectedMemoryRevisions` covers both overlap cases, target-only memories, stale SetTags, FTS, and rollback. |
| `platform/runstate/runstate.go:23`: a database file symlink derives a different core lock | fixed | `platform/runstate/runstate.go:24` resolves canonical database identity; locks and discovery use the canonical parent while retaining the existing run-file layout. `TestSymlinkDatabaseSharesLockAndDiscovery` covers existing and dangling relative file links; `TestCanonicalDatabaseWithMissingParentsAndDirectoryAlias` covers missing directories; `TestCoreLockRejectsSymlinkLoop` covers invalid identity. |
| `compositions/core/reminders.go:156`: cancel and Ack(done) delete one-offs without activity | fixed | `compositions/core/reminders.go:158` audits both deletion paths with the caller actor in the same transaction. `TestOneOffRemovalAndAuditCommitTogether` covers cancel/done, audit failure rollback, delivered/opened no-ops, actor preservation, and missing-reminder errors without phantom audit rows. |
| RotateToken/guard | skipped(fx-mcp ownership) | `agents.go` and `mcp.go` were not edited. |
| Attachment activity and SweepOrphans | skipped(fx-attach ownership) | `attachments.go` and attachment persistence were not edited; the search fix changes only the search portion of `search_attachment.go`. |

## Validation

- PASS: `cd sparky/backend && go vet ./... && go test ./... -race -count=1`, repeated after the final code change. All Go packages passed.
- PASS: `TestCatalogBudgetAndGolden` in that full run, including the unchanged tools/list golden and the 6000-byte budget.
- PASS: `pnpm --filter @polterware/sparky-contracts test` (4 files, 28 tests).
- PASS: `git diff sparky-next --check` and ownership review of the complete diff.
- Desktop typecheck/test were not required: `apps/sparky-desktop` was not changed.
- Contracts, generated code, AGENTS.md, and shared configuration were not changed. No pushes or tags were created.
- Quality review: reread the production and regression diffs for transaction boundaries, connection reuse, revision bypasses, Watch publication ordering, and audit rollback. Replaced the initial writer mutex with a cancellable gate to preserve cancellation behavior.

## Deviations and requests

- The search provider is outside fx-core ownership and its Rebuild opens its own transaction. A core adapter delegates rebuild writes to the owned SQLite provider, which joins the collection transaction; no provider/search edit or contract change was needed.
- `polterware` was absent from PATH. The existing binary at `/Users/erickpatrickbarcelos/codes/polterware-total/polterware/dist/polterware` successfully supplied `format`, but `status`, `query cards`, and `handoff` reported that this directory has no registered project. Work-card writes could not be performed without registering/selecting a project; no project was initialized. The requested status file records the work.
- Requests filed: none.
- Follow-ups for other workstreams: fx-mcp owns token rotation/guard; fx-attach owns attachment audit and orphan sweeping.
