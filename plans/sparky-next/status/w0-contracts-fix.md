# w0-contracts-fix status

- Result: blocked
- Branch / last commit: `sparky/w0-contracts-fix` / HEAD (the commit containing this status)
- Acceptance criteria:
  - No remaining search-related keyword matches: **fail** on the unchanged checkout. `grep -rIi -E "embedding|semantic|hybrid|vector" sparky/ packages/sparky-contracts/ --exclude-dir=node_modules --exclude-dir=spikes --exclude='*recurrence*vector*'` returns existing contracts and `domains/search/fake/index.go`. Removing the latter requires ownership outside this workstream.
  - `bash tooling/verify-sparky.sh`: **fail** on the unchanged checkout. Buf lint/breaking, Go tests/vet, and all Sparky TypeScript typechecks pass. Contract tests report 25 passed and 1 failed: `SQL baseline > matches the specification fence` because the SQL still contains the table removed from doc 02. The script stops there; subsequent checks are unverified.
  - New annotated local `sparky-contracts-v1.1` tag: **not created**, because the corrected contracts are not ready to freeze. Existing `sparky-contracts-v1` was not moved or deleted.
- Deviations from the plan (and why):
  - Implementation stopped before changing contracts: the required removal would leave the out-of-ownership search fake uncompilable. AGENT-PROTOCOL sections 2 and 5 require a request and stopping when acceptance cannot be met within Owns.
  - `pnpm install` succeeded; no tracked dependency files changed.
  - `polterware status` was attempted but failed with `command not found`. The CLI is not available on PATH or inside this worktree; format/query/apply/handoff could not be run without introducing an unrelated binary/build or using another checkout. Work is recorded in the explicitly authorized plan status/request files.
- Requests filed: `plans/sparky-next/requests/w0-contracts-fix-1.md` (exact deletion of the obsolete search fake skeleton, or ownership extension).
- Follow-ups for other WS:
  - Integrator: resolve the search fake ownership request, then resume W0-CONTRACTS-FIX to remove the obsolete SQL/proto/domain/MCP/settings contracts, update tests, regenerate Go/TS, intentionally refresh the Buf baseline, verify, and create the local v1.1 tag.
  - The recurrence fixture and its contract test use the word excluded by the filename exception. Apply that exception to recurrence fixture/test filenames when checking acceptance; these are recurrence cases, not search contracts.
  - No source files were changed, no branches were created, and nothing was pushed.
