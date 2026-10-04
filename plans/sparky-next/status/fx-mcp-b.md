# fx-mcp-b status

- Result: done.
- Branch / last implementation commit: `sparky/fx-mcp-b` / `a5bc387` (this status is committed separately).
- Base: `sparky-next` at `42eb506`; fx-core, fx-mcp, and fx-attach were already merged. The existing worktree and branch matched the assignment.
- Reproduced before editing: `cd sparky/backend && go test ./tests/mcp/ -run TestCoreMCPPlaceWritesUseSessionActor -count=1` failed with `memory changed since this session last read it (rev 3)`.

## Change and audit

- `update_memories`: each item now joins the existing core writer transaction for the field update, checklist, tags, schedule/place Set/Clear, links, and final memory snapshot. Session observations are staged and only the final revision is published after commit. A failed relation write rolls back the item without advancing its session revision. Field-only updates without a transaction adapter retain the domain's exact return value; compound updates fail before writing if the adapter is unavailable.
- `create_memories` with schedule/place: already captured the final revision inside that same transaction and published it only after commit; retained and covered with real SQLite regressions.
- `set_memory_status`: the domain returns its final memory revision from the mutation transaction, including recurring occurrence completion; retained and covered.
- `toggle_check_item`: the positive checked revision plus the domain's single transactional increment is exact, including title lookup; retained and covered.
- `add_attachments`: links and file imports check but do not increment Memory.Rev. They now register each successful write's checked revision, including the first write to an unread memory, and return it without a post-commit read. A subsequent failure does not discard observations for already committed attachments.
- `delete_mind`: collects affected memories, deletes the tree, and captures final moved-memory revisions in one writer transaction. Publishes those revisions after commit and forgets all deleted mind IDs. Previously observed memories with an unseen UI edit retain their stale expectation, so deletion cannot silently absorb that edit. The response includes `memoryRevisions` for the registered memories.
- `create_mind` / `update_mind`: already return/register exact transactional revisions. `delete_memories` checks the remembered revision transactionally and forgets the deleted ID; retained and covered.
- Link insertion no longer acquires the file-import mutex while joining a writer transaction. This avoids a lock inversion with concurrent file import; metadata insertion remains serialized by the transaction.

## Acceptance criteria and validation

| Criterion | Evidence | Result |
| --- | --- | --- |
| Consecutive own writes register/return final revisions, while outside UI writes conflict | `TestCoreMCPUpdatePathsRegisterFinalRevision`: fields; schedule/place Set/Clear; replace/add/remove tags; checklist; links; combined relations | PASS |
| Creation with schedule/place supports immediate follow-up writes and preserves UI conflicts | `TestCoreMCPCreateExtrasRegisterFinalRevision`: schedule, place, both | PASS |
| Status, recurring status, toggle, attachments preserve session revisions | `TestCoreMCPSingleWritePathsKeepSessionRevision`: includes multiple links, repeated file import, and first attachment write without a prior read | PASS |
| Mind deletion advances own moved-memory revisions without absorbing outside edits | `TestCoreMCPMindDeleteRegistersMovedMemoryRevisions`: descendant memory, prior unseen UI memory edit, subsequent UI edit, and UI mind rename | PASS |
| Memory deletion after own write succeeds; deletion after outside write conflicts | `TestCoreMCPMemoryDeleteAfterOwnWriteAndOutsideWrite` | PASS |
| No post-commit revision race | `TestCompoundWritesNeverObservePostCommitUIRevision`, updated `TestTagsNeverObservePostCommitUIRevision`, `TestFirstAttachmentWriteNeverObservesPostCommitUIRevision`; existing toggle race regressions | PASS |
| Failed compound updates preserve persisted and session state | `TestCoreMCPUpdateFailureKeepsCommittedSessionRevision`: schedule, place, and second-link failure | PASS |
| Compound links avoid file-import lock inversion | `TestCompoundLinkWriteDoesNotWaitForFileImportMutex` | PASS |
| Original integration failure resolved and MCP suites race-clean | `cd sparky/backend && go test ./tests/mcp/ ./providers/mcp/ -race -count=1` | PASS |
| Go verification before final commit | `cd sparky/backend && GOFLAGS=-p=1 go vet ./... && GOFLAGS=-p=1 go test ./... -race -count=1` | PASS |
| Frozen contracts unaffected | `pnpm --filter @polterware/sparky-contracts test`: 4 files, 28 tests | PASS |
| tools/list <= 6 KB | `TestCatalogBudgetAndGolden`: 5964 bytes, unchanged schemas and golden catalog | PASS |
| Owned paths and whitespace | `git diff sparky-next...HEAD --check`, `git diff --check`; only assigned MCP/domain paths and this status | PASS |

## Deviations and requests

- Reused the existing core transaction adapter instead of adding new domain return types: the final snapshot runs inside the same writer transaction, with no opportunity for a concurrent UI write between mutation and capture. No schemas, generated code, frozen contracts, or core source changed.
- The exact initial full gate passed vet, then failed test compilation/linking with `no space left on device`. Re-running both commands with process-local `GOFLAGS=-p=1` passed the complete race suite. No files were removed to reclaim space.
- `polterware` is unavailable (`command not found`, exit 127). Status, format, and card-query commands were attempted; Work-card recording is unavailable. Handoff is attempted at session end.
- No AGENTS.md changes, pushes, tags, or additional branches. All commits are local.
- Requests filed: none.
- Follow-ups for other workstreams: none.
