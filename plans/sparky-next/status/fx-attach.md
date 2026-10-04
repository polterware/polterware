# fx-attach status

- Result: partial (owned fixes complete; eager core-startup hook awaits integrator)
- Branch: `sparky/fx-attach`, based on `sparky-next` at `0ef2418a864b74da6810e57338164b1cfd766577`.
- Last implementation commit: `b7d76f1` (`sparky(fx-attach): initialize recovery once per attachment store`). The final status commit is the branch HEAD (`git log -1`).
- Review sources: `integration/.orchestra/runs/rv-attach/last.txt` and the attachments audit finding in `integration/.orchestra/runs/rv-core/last.txt`.

## Findings verified against the original code

| Finding | Disposition | Evidence and regression coverage |
| --- | --- | --- |
| IPv4-mapped IPv6 SSRF | fixed | `blocked` previously applied predicates before unmapping; it now calls `Unmap`. `TestSSRFGuardRejectsPrivateDestinations` covers `0.0.0.0`, `::`, mapped unspecified/loopback/metadata IPs, decimal/octal/hex forms and userinfo. Noncanonical numeric forms are explicitly rejected. |
| DNS validation ignores timeout/cancellation | fixed | `validatePublicURL` used `context.Background`; validation, redirects, request and dialing now share the fetch deadline. The dial closure preserves it even when HTTP transport detaches its own context. `TestTitleValidationSharesDeadline`, `TestTitleDialSharesValidationDeadline`. |
| Store symlink ancestors on import/open/delete | fixed | `MkdirAll`, string-path rename and deletion previously traversed memory-directory symlinks. Root and memory directory components now use descriptor-relative `fstatat` without following links and `openat(O_NOFOLLOW)`. Temp creation, rename, unlink and sweep stay anchored. `TestStoreRejectsSymlinkAncestorsForEveryFileOperation`, `TestAnchoredDirectorySurvivesAncestorSwap`. `AbsolutePath` validates every store component before returning its informational path. |
| Unbounded copy before MIME size check | fixed | The previous unrestricted `io.Copy` ran before size rejection. Copy now reads at most the MIME limit plus one byte, including the sniff prefix, with cancellation checks. `TestImportAppliesMIMELimitBeforeCompletingCopy` covers photo/audio/file budgets and temp cleanup. |
| WAV/WebM MIME breaks preview | fixed | Sniffed `audio/wave`/`audio/x-wav` normalize to `audio/wav`; sniffed `video/webm` normalizes to the supported audio recording container `audio/webm`. Correct `.wav`/`.webm` extensions select preview MIME and audio kind. `TestImportNormalizesPreviewMIME`; Desktop `serves normalized %s audio previews`. |
| Canceled rollback cleanup and discarded error | fixed | Cleanup previously reused the canceled request context and ignored errors. A bounded independent context preserves cleanup, and `errors.Join` surfaces both transaction and cleanup failures. `TestFailedTransactionCleanupIgnoresCancellationAndSurfacesFailure`. |
| Import/dedup races with Delete | fixed | Delete previously bypassed the import mutex. Mutations now share serialization. `TestDeleteCannotInvalidateAnImportDedupSnapshot`, existing concurrent dedup test, all run with `-race`. |
| Dedup ignores expectedRev | fixed | Hash matches returned before revision validation. Revision check and dedup list now share the transaction. `TestDedupChecksExpectedRevisionAndCleansCopy`. |
| Crash leftovers never swept | skipped (literal eager startup requires unowned `wiring.go`; request `fx-attach-1`) | Recovery is implemented and serialized with `sync.Once` before the first service publication/import; missing-file metadata deletes are audited. `TestCoreAttachmentAuditAndRecoveryBeforeImport`, `TestCoreConcurrentFirstImportsPreserveEveryFile`. Files are still not swept immediately at `OpenRuntime` without the integrator hook. |
| Missing audit for AddLink/import/delete/reorder | fixed | Metadata writes previously had no activity append. Core now supplies `Audit.Write` inside the mutation transaction; reorder delegates through the service. Dedup emits no fabricated mutation event. `TestCoreAttachmentAuditAndRecoveryBeforeImport` checks exactly two creates, one update and one delete, actor attribution, plus recovery system audit. |
| Protocol path replacement/symlink disclosure | fixed | Handler previously used `stat` and later `readFile(path)`. It now opens without following links, validates and reads the same descriptor. Darwin uses `O_NOFOLLOW_ANY` for every component; Linux anchors child opens through directory descriptors. Desktop `reads the opened descriptor when the path is swapped after validation` and ancestor rejection tests. |
| Protocol accepts multiple hard links | fixed | `realpath`/`stat` previously accepted hard-linked external files. Descriptor stats reject link counts above one before and after reading. Desktop `rejects hard links and symlinks in both storage ancestors`. |

No reviewed finding was classified not-a-bug. FetchTitle remains an opt-in helper without a current core caller; its confirmed bypass was fixed anyway.

## Validation

- PASS: `cd sparky/backend && go vet ./... && go test ./... -race -count=1`.
- PASS: `TestCatalogBudgetAndGolden` included in the full backend run (18 tools, exact golden, tools/list <= 6000 bytes).
- PASS: `pnpm --filter @polterware/sparky-desktop typecheck`.
- PASS: `pnpm --filter @polterware/sparky-desktop test` (14 files, 93 tests).
- PASS: `pnpm --filter @polterware/sparky-contracts test` (4 files, 28 tests).
- PASS: `git diff --check` and ownership inspection against `sparky-next`.
- Self-review: checked transaction/audit rollback, independent cleanup, dedup/delete linearization, singleton recovery publication, descriptor lifetime, symlink swaps, hard links and total copy budgets.
- Platform limit: runtime validation was on macOS. Linux uses descriptor-relative operations but was not executed here. Unsupported Desktop platforms fail closed for preview opening.

## Deviations and handoff

- `fx-attach-1.md` requests only the eager `OpenRuntime` attachment initialization hook in unowned core wiring. No contracts or generated files changed.
- Source files changed only inside ownership. Planning output is limited to the requested status and integrator request.
- Small local commits only; no push, tag, new unrelated branch, browser, UI preview or distribution. `AGENTS.md` was not changed.
- Polter CLI was absent from PATH; used the existing main-checkout `dist/polterware`. This worktree has no registered project, so status/cards/apply/handoff use the existing project from its registered main checkout. Work card: `sparky-fx-attach`.
