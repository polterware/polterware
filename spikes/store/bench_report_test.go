package store

import (
	"context"
	"database/sql"
	"fmt"
	"math"
	"os"
	"runtime"
	"sort"
	"testing"
	"time"
)

const topK = 20

func TestBenchmarkReport(t *testing.T) {
	ctx := context.Background()
	fmt.Printf("host goos=%s goarch=%s cpu=%d go=%s\n", runtime.GOOS, runtime.GOARCH, runtime.NumCPU(), runtime.Version())

	for _, n := range []int{10_000, 50_000} {
		fmt.Printf("seeding %d memories\n", n)
		corpus, err := BuildCorpus(ctx, n)
		if err != nil {
			t.Fatal(err)
		}
		t.Cleanup(func() {
			_ = corpus.DB.Close()
			_ = os.Remove(corpus.Path)
		})
		ns := corpus.Insert.Nanoseconds() / int64(n)
		fmt.Printf("INSERT n=%d elapsed=%s ns/op=%d rows/s=%.0f db_bytes=%d\n",
			n, corpus.Insert.Round(time.Millisecond), ns, float64(n)/corpus.Insert.Seconds(), corpus.SizeBytes)

		fts := measureN(80, func(i int) error {
			word := Words[i%len(Words)]
			if i%2 == 0 {
				word = word + " " + Words[(i+3)%len(Words)]
			}
			return ftsOnce(ctx, corpus.DB, word)
		})
		printLat(fmt.Sprintf("FTS query n=%d", n), fts)

		listMind := measureN(80, func(int) error {
			return listOnce(ctx, corpus.DB, `
				SELECT m.id FROM memories m
				WHERE m.status = 'active' AND m.mind_id = ? AND m.pinned = 0
				ORDER BY m.updated_at DESC LIMIT 50`, corpus.MindID)
		})
		printLat(fmt.Sprintf("list status+mind+updated n=%d", n), listMind)

		listTag := measureN(80, func(int) error {
			return listOnce(ctx, corpus.DB, `
				SELECT m.id FROM memories m
				JOIN memory_tags mt ON mt.memory_id = m.id
				WHERE m.status = 'active' AND mt.tag_id = ? AND m.due_at IS NOT NULL
				ORDER BY m.updated_at DESC LIMIT 50`, corpus.TagID)
		})
		printLat(fmt.Sprintf("list status+tag+due n=%d", n), listTag)

		if n == 10_000 {
			p95 := percentile(sortedCopy(fts), 0.95)
			verdict := "PASS"
			if p95 >= 30*time.Millisecond {
				verdict = "FAIL"
			}
			fmt.Printf("BUDGET FTS p95 @10k %s %s (limit 30ms)\n", verdict, p95.Round(time.Microsecond))
		}
	}

	for _, dims := range []int{384, 768} {
		for _, n := range []int{10_000, 20_000, 50_000} {
			fmt.Printf("generating vectors n=%d dims=%d\n", n, dims)
			data := Generate(n, dims, uint64(n*1000+dims))
			single := measureN(40, func(i int) error {
				base := (i % n) * dims
				hits := TopK(data, dims, topK, data[base:base+dims])
				if len(hits) != topK || hits[0].Score < 0.99 {
					return fmt.Errorf("bad topk len=%d", len(hits))
				}
				return nil
			})
			printLat(fmt.Sprintf("vector topk single n=%d dims=%d k=%d", n, dims, topK), single)
			parallel := measureN(40, func(i int) error {
				base := (i % n) * dims
				hits := TopKParallel(data, dims, topK, 4, data[base:base+dims])
				if len(hits) != topK || hits[0].Score < 0.99 {
					return fmt.Errorf("bad parallel topk len=%d", len(hits))
				}
				return nil
			})
			printLat(fmt.Sprintf("vector topk x4 n=%d dims=%d k=%d", n, dims, topK), parallel)
			if n == 20_000 && dims == 768 {
				limit := 50 * time.Millisecond
				for _, row := range []struct {
					name string
					lats []time.Duration
				}{{"single", single}, {"x4", parallel}} {
					p95 := percentile(sortedCopy(row.lats), 0.95)
					verdict := "PASS"
					if p95 >= limit {
						verdict = "FAIL"
					}
					fmt.Printf("BUDGET vector %s p95 @20k×768 %s %s (limit 50ms)\n", row.name, verdict, p95.Round(time.Microsecond))
				}
			}
		}
	}

	fmt.Println("blob scan 20k x 768")
	blobLat, err := blobScan(ctx)
	if err != nil {
		t.Fatal(err)
	}
	printLat("vector topk sqlite blob scan n=20000 dims=768", blobLat)

	probe, err := ProbeOllama(ctx)
	if err != nil {
		t.Fatal(err)
	}
	fmt.Printf("OLLAMA skipped=%v %s\n", probe.Skipped, probe.Note)
}

func ftsOnce(ctx context.Context, db *sql.DB, user string) error {
	q := FTSQuery(user)
	rows, err := db.QueryContext(ctx, `
		SELECT memory_id, bm25(memory_fts, 10.0, 1.0, 2.0, 3.0, 1.5), snippet(memory_fts, 1, '<b>', '</b>', '…', 8)
		FROM memory_fts WHERE memory_fts MATCH ?
		ORDER BY bm25(memory_fts, 10.0, 1.0, 2.0, 3.0, 1.5) LIMIT 20`, q)
	if err != nil {
		return err
	}
	defer rows.Close()
	n := 0
	for rows.Next() {
		var id, snip string
		var rank float64
		if err := rows.Scan(&id, &rank, &snip); err != nil {
			return err
		}
		n++
	}
	if n == 0 {
		return fmt.Errorf("fts %q returned no rows", q)
	}
	return rows.Err()
}

func listOnce(ctx context.Context, db *sql.DB, query string, arg string) error {
	rows, err := db.QueryContext(ctx, query, arg)
	if err != nil {
		return err
	}
	defer rows.Close()
	for rows.Next() {
		var id string
		if err := rows.Scan(&id); err != nil {
			return err
		}
	}
	return rows.Err()
}

func blobScan(ctx context.Context) ([]time.Duration, error) {
	const n, dims = 20_000, 768
	data := Generate(n, dims, 99)
	f, err := os.CreateTemp("", "sparky-vec-*.sqlite")
	if err != nil {
		return nil, err
	}
	path := f.Name()
	if err := f.Close(); err != nil {
		return nil, err
	}
	defer os.Remove(path)
	db, err := Open(ctx, path)
	if err != nil {
		return nil, err
	}
	defer db.Close()
	db.SetMaxOpenConns(1)
	if err := ApplyBaseline(ctx, db); err != nil {
		return nil, err
	}
	tx, err := db.BeginTx(ctx, nil)
	if err != nil {
		return nil, err
	}
	now := "2026-10-03T21:30:00.000Z"
	for i := 0; i < n; i++ {
		if i > 0 && i%500 == 0 {
			if err := tx.Commit(); err != nil {
				return nil, err
			}
			tx, err = db.BeginTx(ctx, nil)
			if err != nil {
				return nil, err
			}
		}
		mem := id(100_000 + i)
		if _, err := tx.ExecContext(ctx, `
			INSERT INTO memories (id, mind_id, title, note, status, created_at, updated_at)
			VALUES (?, NULL, ?, '', 'active', ?, ?)`, mem, fmt.Sprintf("v %d", i), now, now); err != nil {
			_ = tx.Rollback()
			return nil, err
		}
		blob := Float32LE(data[i*dims : (i+1)*dims])
		if _, err := tx.ExecContext(ctx, `
			INSERT INTO embeddings (memory_id, model, dims, vector, text_hash, updated_at)
			VALUES (?, 'probe', ?, ?, 'h', ?)`, mem, dims, blob, now); err != nil {
			_ = tx.Rollback()
			return nil, err
		}
	}
	if err := tx.Commit(); err != nil {
		return nil, err
	}
	query := append([]float32(nil), data[:dims]...)
	return measureN(10, func(int) error {
		rows, err := db.QueryContext(ctx, `SELECT vector FROM embeddings`)
		if err != nil {
			return err
		}
		defer rows.Close()
		best := float32(-1)
		var scratch [768]float32
		for rows.Next() {
			var blob []byte
			if err := rows.Scan(&blob); err != nil {
				return err
			}
			if len(blob) != dims*4 {
				return fmt.Errorf("blob len %d", len(blob))
			}
			for d := 0; d < dims; d++ {
				off := d * 4
				bits := uint32(blob[off]) | uint32(blob[off+1])<<8 | uint32(blob[off+2])<<16 | uint32(blob[off+3])<<24
				scratch[d] = math.Float32frombits(bits)
			}
			score := dot(scratch[:], query)
			if score > best {
				best = score
			}
		}
		if err := rows.Err(); err != nil {
			return err
		}
		if best < 0.99 {
			return fmt.Errorf("blob best %g", best)
		}
		return nil
	}), nil
}

func measureN(n int, fn func(i int) error) []time.Duration {
	for i := 0; i < 5; i++ {
		if err := fn(i); err != nil {
			panic(err)
		}
	}
	lats := make([]time.Duration, n)
	for i := 0; i < n; i++ {
		start := time.Now()
		if err := fn(i + 5); err != nil {
			panic(err)
		}
		lats[i] = time.Since(start)
	}
	return lats
}

func printLat(name string, lats []time.Duration) {
	sorted := sortedCopy(lats)
	var sum time.Duration
	for _, d := range sorted {
		sum += d
	}
	mean := time.Duration(int64(sum) / int64(len(sorted)))
	fmt.Printf("LAT %s samples=%d ns/op=%d p50=%s p95=%s\n",
		name, len(sorted), mean.Nanoseconds(),
		percentile(sorted, 0.50).Round(time.Microsecond),
		percentile(sorted, 0.95).Round(time.Microsecond))
}

func sortedCopy(lats []time.Duration) []time.Duration {
	sorted := append([]time.Duration(nil), lats...)
	sort.Slice(sorted, func(i, j int) bool { return sorted[i] < sorted[j] })
	return sorted
}

func percentile(sorted []time.Duration, p float64) time.Duration {
	if len(sorted) == 0 {
		return 0
	}
	idx := int(math.Ceil(p*float64(len(sorted)))) - 1
	if idx < 0 {
		idx = 0
	}
	if idx >= len(sorted) {
		idx = len(sorted) - 1
	}
	return sorted[idx]
}
