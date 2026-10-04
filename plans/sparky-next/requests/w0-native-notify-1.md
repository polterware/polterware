# W0-NATIVE-NOTIFY request 1: unblock the existing SQL baseline gate

- Owner: W0-CONTRACTS-FIX / integrator.
- Blocked branch: `sparky/w0-native-notify`, based on `18e9281`.
- Why: `bash tooling/verify-sparky.sh` exits 1 in `packages/sparky-contracts/src/baseline.test.mjs`, `SQL baseline > matches the specification fence`. The SQL baseline still contains `embeddings`; the specification fence has already removed it per resolution A7. All of these paths are outside W0-NATIVE-NOTIFY's ownership. No changes to contracts were made in this workstream.
- Evidence: `spikes/native-notify/evidence/verify-sparky.txt`, especially the Expected/Received diff. Go tests, Go vet and Sparky TypeScript checks passed before the failing contract test. The baseline applies successfully; its specification equality fails.
- Requested change: merge the authorized W0-CONTRACTS-FIX workstream, including its complete contract/tag/baseline work. The immediate SQL change needed for this failure is shown below; it is not a substitute for that workstream's complete scope. Handle the contract freeze per the protocol.

```diff
--- a/sparky/backend/providers/sqlite/migrations/0001_baseline.sql
+++ b/sparky/backend/providers/sqlite/migrations/0001_baseline.sql
@@
 CREATE VIRTUAL TABLE memory_fts USING fts5(
   memory_id UNINDEXED, title, note, checklist, tags, mind,
   tokenize = 'unicode61 remove_diacritics 2', prefix = '2 3'
 );
-CREATE TABLE embeddings (
-  memory_id TEXT PRIMARY KEY REFERENCES memories(id) ON DELETE CASCADE,
-  model TEXT NOT NULL, dims INTEGER NOT NULL, vector BLOB NOT NULL,   -- float32 little-endian, L2-normalized
-  text_hash TEXT NOT NULL, updated_at TEXT NOT NULL
-) STRICT;
```

- Resume: after the integrator merges the fix, rebase this branch onto `sparky-next` and rerun the gate. Continue the native acceptance runs and propose O1-final in a separate request for the integrator to append to `00-decisions.md`. Do not edit the decision log from this workstream.
