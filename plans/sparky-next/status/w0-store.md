# w0-store status

- Result: done
- Branch / last commit: `sparky/w0-store` (see `git log -1` after push)
- Acceptance criteria:
  - Benchmarks table with ns/op and p95 in this file → **pass** (tables below; host Apple M1 Pro, darwin/arm64, 8 cores, go1.27.1)
  - FTS p95 < 30 ms at 10k memories → **pass** (p95 5.168 ms, mean 2,869,141 ns/op)
  - Vector top-K < 50 ms at 20k×768 → **pass** (single-thread p95 19.063 ms; 4 goroutines p95 5.141 ms; k=20)
  - Statement on missing capabilities and fallbacks (`ncruces/go-sqlite3` WASM, `sqlite-vec`) → **pass** (no capability missing; fallback not required)
  - O3 and A8 confirmation → **pass** (decision below; append the resolutions-log line from this file, do not treat it as already written to `00-decisions.md`)
  - Evidence command → **pass** (`cd spikes/store && GOWORK=off go vet ./... && GOWORK=off go test -count=1 -timeout 10m -v .`, exit 0)
- Deviations from the plan (and why):
  - Baseline SQL is the DDL in `02-data-and-contracts.md`, copied to `spikes/store/schema.sql`. `sparky/backend/providers/sqlite/migrations/0001_baseline.sql` is not on `sparky-next` yet (W0-CONTRACTS).
  - The spike module is not in `go.work` (`go.work` is owned by W0-SCAFFOLD). Tests run with `GOWORK=off`.
  - `00-decisions.md` was not edited (not in Owns). The integrator appends the resolution line below.
  - Ollama at `127.0.0.1:11434` answered `/api/tags` with zero models, so no embedding call ran. Skipped with a note, as the plan allows when a local model is not available.
- Requests filed (plans/sparky-next/requests/...): none
- Follow-ups for other WS:
  - Integrator: append this line to the resolutions log in `00-decisions.md`:
    `2026-10-03 O3: modernc.org/sqlite v1.59.0 (SQLite 3.53.4) has FTS5 (bm25, snippet, prefix), WAL with concurrent readers, and PreUpdate/Commit hooks that emit entity-level changes. In-memory brute-force top-20 at 20k×768 is p95 19.1 ms on one Apple M1 Pro core (budget 50 ms). A8 confirmed. Keep vectors in process memory; a per-query BLOB scan was p95 95 ms. sqlite-vec is not required. (W0-STORE, plans/sparky-next/status/w0-store.md)`
  - **W1-SEARCH**: load `embeddings.vector` into a `[]float32` cache and run the dot-product scan there. Do not `SELECT` the BLOBs on every query. At 50k×768 one core is p95 46.9 ms (under 50 ms, little headroom); 4 goroutines bring that to p95 12.6 ms.
  - **W1-STORE**: `list` with status + tag + due is p95 37.1 ms at 50k (no budget in this spike). The mind/status/`updated_at` list is p95 25.7 ms at 50k.
  - **W0-CONTRACTS**: spike applied the doc-02 DDL successfully, including the FTS5 virtual table and `STRICT` tables.

## Decision (O3, A8)

`modernc.org/sqlite` **v1.59.0** (same version as `backend/go.mod`) embeds **SQLite 3.53.4** with `ENABLE_FTS5`. Confirmed on this machine:

- `bm25(memory_fts, 10.0, 1.0, 2.0, 3.0, 1.5)` and `snippet()` work. `cafe` matches `café` (`unicode61 remove_diacritics 2`). Prefix `reu` matches `reunião`. `EXPLAIN QUERY PLAN` for a 2-character prefix uses `SCAN memory_fts VIRTUAL TABLE INDEX`.
- `journal_mode=wal`. Four readers kept the pre-commit snapshot while a writer held an open transaction, then saw the commit. A 1.5 s run with one writer and four readers returned no errors (`TestWALOneWriterManyReaders`).
- `RegisterPreUpdateHook` + `RegisterCommitHook` + `RegisterRollbackHook` (same shape as `backend/platform/db/sqlite/hooks.go`) emit `{entity, id, op}` on commit. Rollback emits nothing (`TestEntityChangeHooks`).

**A8 stands.** Brute-force cosine (dot product of L2-normalized float32 vectors, min-heap top-20) meets the 50 ms budget at 20k×768 without `sqlite-vec`. No missing SQLite capability, so `ncruces/go-sqlite3` is not needed.

Constraint for the implementation: the budget holds for vectors already in memory. Reloading 20k×768 BLOBs from SQLite on every query was p95 **95.247 ms** (over the budget). W1-SEARCH should cache the matrix.

At 50k×768, one thread is still under 50 ms (p95 46.916 ms). Four goroutines are the comfortable path at that size.

## Method

- Insert time includes the memory row, the FTS row, one tag link, and two checklist rows, in transactions of 250. `ns/op` is elapsed divided by memory count. DB size is the main file after `PRAGMA wal_checkpoint(TRUNCATE)`.
- Query timings drop 5 warmup calls, then take the sample count below. `ns/op` is the mean. p50/p95 use nearest-rank on the sorted samples: index `ceil(p*n)-1`.
- FTS queries alternate one term and two terms from the fixture vocabulary (Portuguese diacritics included). The last term is a prefix. Ranking uses the doc-05 `bm25` weights and `snippet` on the title column.
- Vector queries use an existing row as the query (top score about 1) so a wrong heap fails the test. Parallel timings include spawning 4 goroutines per query.

## Insert throughput and database size

| memories | ns/op | rows/s | elapsed | db bytes |
|---:|---:|---:|---:|---:|
| 10,000 | 51,873 | 19,278 | 519 ms | 15,618,048 |
| 50,000 | 62,670 | 15,957 | 3.134 s | 78,000,128 |

## FTS and list_memories

| workload | samples | ns/op | p50 | p95 | budget |
|---|---:|---:|---:|---:|---|
| FTS query, 10k | 80 | 2,869,141 | 2.886 ms | 5.168 ms | **pass** (< 30 ms) |
| FTS query, 50k | 80 | 13,519,383 | 13.224 ms | 25.130 ms | no budget (still < 30 ms) |
| list status+mind+updated, 10k | 80 | 4,360,001 | 4.124 ms | 5.077 ms | no budget |
| list status+mind+updated, 50k | 80 | 23,893,939 | 23.801 ms | 25.698 ms | no budget |
| list status+tag+due, 10k | 80 | 6,437,882 | 6.352 ms | 7.041 ms | no budget |
| list status+tag+due, 50k | 80 | 36,047,795 | 36.097 ms | 37.125 ms | no budget |

## Vector top-K (k=20)

| rows | dims | mode | samples | ns/op | p50 | p95 | budget |
|---:|---:|---|---:|---:|---:|---:|---|
| 10,000 | 384 | 1 thread | 40 | 4,364,857 | 4.337 ms | 4.477 ms | |
| 10,000 | 384 | 4 goroutines | 40 | 1,285,154 | 1.228 ms | 1.576 ms | |
| 20,000 | 384 | 1 thread | 40 | 8,726,746 | 8.709 ms | 8.886 ms | |
| 20,000 | 384 | 4 goroutines | 40 | 2,400,541 | 2.379 ms | 2.501 ms | |
| 50,000 | 384 | 1 thread | 40 | 21,617,509 | 21.533 ms | 21.949 ms | |
| 50,000 | 384 | 4 goroutines | 40 | 5,802,268 | 5.747 ms | 5.937 ms | |
| 10,000 | 768 | 1 thread | 40 | 9,410,638 | 9.257 ms | 9.490 ms | |
| 10,000 | 768 | 4 goroutines | 40 | 2,542,874 | 2.521 ms | 2.659 ms | |
| 20,000 | 768 | 1 thread | 40 | 18,501,989 | 18.465 ms | 19.063 ms | **pass** (< 50 ms) |
| 20,000 | 768 | 4 goroutines | 40 | 4,992,312 | 4.982 ms | 5.141 ms | **pass** (< 50 ms) |
| 50,000 | 768 | 1 thread | 40 | 46,153,246 | 46.096 ms | 46.916 ms | under 50 ms, thin headroom |
| 50,000 | 768 | 4 goroutines | 40 | 12,428,590 | 12.329 ms | 12.560 ms | |
| 20,000 | 768 | SQLite BLOB reload each query | 10 | 92,646,074 | 92.314 ms | 95.247 ms | over 50 ms; not the A8 path |

## Ollama

Skipped. `GET http://127.0.0.1:11434/api/tags` returned 200 with an empty model list, so there was no embedding model to call. Not required for O3.
