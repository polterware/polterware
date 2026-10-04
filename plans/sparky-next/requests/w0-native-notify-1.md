# W0-NATIVE-NOTIFY request 1: resolved by integrator gate waiver

- Status: **resolved for W0-NATIVE-NOTIFY**, 2026-10-04.
- Original request: repair/merge the unrelated SQL baseline mismatch before resuming native runtime acceptance.
- Integrator decision: the failure in `packages/sparky-contracts/src/baseline.test.mjs` is known and assigned to W0-CONTRACTS-FIX. Wave-0 spikes are judged by their own evidence; record the failing gate and continue. No rebase, contract edit or shared-file mutation was needed to complete this workstream.
- Gate remains failing: `bash tooling/verify-sparky.sh`; `SQL baseline > matches the specification fence`, baseline includes the removed `embeddings` table. Latest evidence: `spikes/native-notify/evidence/verify-sparky-resume.txt` (25 contract tests pass, one fails).
- Native continuation: **completed**; both engine implementations pass their runtime acceptance. See `plans/sparky-next/status/w0-native-notify.md` and `spikes/native-notify/evidence/runtime.json`.
- Ownership follow-up, not a runtime blocker: integrator appends the O1-final resolution proposed in the status file to `00-decisions.md`; W0-CONTRACTS-FIX repairs the external gate. This workstream recommends Objective-C++ N-API, local ad-hoc signing, no added notification entitlement, and keeping native delivery rather than the timer fallback.
- Nothing was pushed or tagged. No other workstream's files were edited.
