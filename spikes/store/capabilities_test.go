package store

import (
	"context"
	"database/sql"
	"fmt"
	"os"
	"sync"
	"testing"
	"time"
)

func TestFTSCapabilities(t *testing.T) {
	ctx := context.Background()
	db := openTemp(t, ctx, false)
	db.SetMaxOpenConns(1)
	if err := ApplyBaseline(ctx, db); err != nil {
		t.Fatalf("apply baseline: %v", err)
	}

	var version string
	if err := db.QueryRowContext(ctx, `SELECT sqlite_version()`).Scan(&version); err != nil {
		t.Fatal(err)
	}
	var fts5 int
	if err := db.QueryRowContext(ctx, `SELECT sqlite_compileoption_used('ENABLE_FTS5')`).Scan(&fts5); err != nil {
		t.Fatal(err)
	}
	if fts5 != 1 {
		t.Fatalf("ENABLE_FTS5 not compiled in (sqlite %s)", version)
	}
	t.Logf("sqlite %s ENABLE_FTS5=1", version)

	var createSQL string
	if err := db.QueryRowContext(ctx, `SELECT sql FROM sqlite_master WHERE name = 'memory_fts'`).Scan(&createSQL); err != nil {
		t.Fatal(err)
	}
	if !containsAll(createSQL, "prefix", "unicode61", "remove_diacritics") {
		t.Fatalf("fts ddl missing tokenizer options: %s", createSQL)
	}

	now := "2026-10-03T21:30:00.000Z"
	mind := id(1)
	if _, err := db.ExecContext(ctx, `
		INSERT INTO minds (id, parent_id, name, color, icon, sort_order, created_at, updated_at, rev)
		VALUES (?, NULL, 'Pessoal', NULL, NULL, 0, ?, ?, 1)`, mind, now, now); err != nil {
		t.Fatal(err)
	}
	mem := id(2)
	if _, err := db.ExecContext(ctx, `
		INSERT INTO memories (id, mind_id, title, note, status, pinned, priority, due_at, due_all_day, sort_order, created_at, updated_at, rev)
		VALUES (?, ?, 'café da manhã', 'reunião sobre o código e a memória do projeto', 'active', 0, 1, NULL, 0, 0, ?, ?, 1)`,
		mem, mind, now, now); err != nil {
		t.Fatal(err)
	}
	if _, err := db.ExecContext(ctx, `
		INSERT INTO memory_fts (memory_id, title, note, checklist, tags, mind)
		VALUES (?, 'café da manhã', 'reunião sobre o código e a memória do projeto', 'comprar pão', 'casa', 'Pessoal')`, mem); err != nil {
		t.Fatalf("fts insert: %v", err)
	}

	query := FTSQuery("cafe")
	if query != `"cafe"*` {
		t.Fatalf("FTSQuery = %s", query)
	}
	var gotID, snippet string
	var rank float64
	err := db.QueryRowContext(ctx, `
		SELECT memory_id,
		       bm25(memory_fts, 10.0, 1.0, 2.0, 3.0, 1.5),
		       snippet(memory_fts, 1, '<b>', '</b>', '…', 8)
		FROM memory_fts
		WHERE memory_fts MATCH ?
		ORDER BY bm25(memory_fts, 10.0, 1.0, 2.0, 3.0, 1.5)
		LIMIT 1`, query).Scan(&gotID, &rank, &snippet)
	if err != nil {
		t.Fatalf("bm25/snippet query %q: %v", query, err)
	}
	if gotID != mem {
		t.Fatalf("diacritics: cafe found %s, want %s", gotID, mem)
	}
	if snippet == "" {
		t.Fatal("empty snippet")
	}
	t.Logf("diacritics hit rank=%g snippet=%s", rank, snippet)

	var prefixHit string
	if err := db.QueryRowContext(ctx, `
		SELECT memory_id FROM memory_fts WHERE memory_fts MATCH ? LIMIT 1`, FTSQuery("reu")).Scan(&prefixHit); err != nil {
		t.Fatalf("prefix reu: %v", err)
	}
	if prefixHit != mem {
		t.Fatalf("prefix hit %s", prefixHit)
	}

	rows, err := db.QueryContext(ctx, `EXPLAIN QUERY PLAN SELECT memory_id FROM memory_fts WHERE memory_fts MATCH '"ca"*'`)
	if err != nil {
		t.Fatal(err)
	}
	defer rows.Close()
	var plan string
	for rows.Next() {
		var id, parent, notused int
		var detail string
		if err := rows.Scan(&id, &parent, &notused, &detail); err != nil {
			t.Fatal(err)
		}
		plan += detail + "; "
	}
	t.Logf("prefix plan: %s", plan)
}

func TestWALOneWriterManyReaders(t *testing.T) {
	ctx := context.Background()
	db := openTemp(t, ctx, false)
	db.SetMaxOpenConns(8)
	if err := ApplyBaseline(ctx, db); err != nil {
		t.Fatal(err)
	}
	var mode string
	if err := db.QueryRowContext(ctx, `PRAGMA journal_mode`).Scan(&mode); err != nil {
		t.Fatal(err)
	}
	if mode != "wal" {
		t.Fatalf("journal_mode = %s", mode)
	}

	now := "2026-10-03T21:30:00.000Z"
	if _, err := db.ExecContext(ctx, `
		INSERT INTO memories (id, mind_id, title, note, status, created_at, updated_at)
		VALUES (?, NULL, 'seed', '', 'active', ?, ?)`, id(10), now, now); err != nil {
		t.Fatal(err)
	}

	writer, err := db.Conn(ctx)
	if err != nil {
		t.Fatal(err)
	}
	tx, err := writer.BeginTx(ctx, nil)
	if err != nil {
		t.Fatal(err)
	}
	if _, err := tx.ExecContext(ctx, `
		INSERT INTO memories (id, mind_id, title, note, status, created_at, updated_at)
		VALUES (?, NULL, 'uncommitted', '', 'active', ?, ?)`, id(11), now, now); err != nil {
		t.Fatal(err)
	}

	readers := make([]*sql.Conn, 4)
	for i := range readers {
		c, err := db.Conn(ctx)
		if err != nil {
			t.Fatal(err)
		}
		readers[i] = c
		var n int
		if err := c.QueryRowContext(ctx, `SELECT count(*) FROM memories`).Scan(&n); err != nil {
			t.Fatalf("reader %d during write: %v", i, err)
		}
		if n != 1 {
			t.Fatalf("reader %d saw %d rows before commit", i, n)
		}
	}
	if err := tx.Commit(); err != nil {
		t.Fatal(err)
	}
	for i, c := range readers {
		var n int
		if err := c.QueryRowContext(ctx, `SELECT count(*) FROM memories`).Scan(&n); err != nil {
			t.Fatal(err)
		}
		if n != 2 {
			t.Fatalf("reader %d saw %d rows after commit", i, n)
		}
	}
	if err := writer.Close(); err != nil {
		t.Fatal(err)
	}
	for _, c := range readers {
		if err := c.Close(); err != nil {
			t.Fatal(err)
		}
	}

	runCtx, cancel := context.WithTimeout(ctx, 1500*time.Millisecond)
	defer cancel()
	var wg sync.WaitGroup
	errCh := make(chan error, 8)
	wg.Add(1)
	go func() {
		defer wg.Done()
		for n := 0; runCtx.Err() == nil; n++ {
			_, err := db.ExecContext(context.Background(), `
				INSERT INTO memories (id, mind_id, title, note, status, created_at, updated_at)
				VALUES (?, NULL, 'live', '', 'active', ?, ?)`, id(10_000+n), now, now)
			if err != nil && runCtx.Err() == nil {
				errCh <- fmt.Errorf("writer: %w", err)
				return
			}
		}
	}()
	for r := 0; r < 4; r++ {
		wg.Add(1)
		go func() {
			defer wg.Done()
			for runCtx.Err() == nil {
				var n int
				err := db.QueryRowContext(context.Background(), `SELECT count(*) FROM memories`).Scan(&n)
				if err != nil && runCtx.Err() == nil {
					errCh <- fmt.Errorf("reader: %w", err)
					return
				}
			}
		}()
	}
	wg.Wait()
	close(errCh)
	for err := range errCh {
		t.Error(err)
	}
}

func TestEntityChangeHooks(t *testing.T) {
	ctx := context.Background()
	db := openTemp(t, ctx, true)
	db.SetMaxOpenConns(1)
	if err := ApplyBaseline(ctx, db); err != nil {
		t.Fatal(err)
	}

	var mu sync.Mutex
	var batches [][]Change
	SetChangeRecorder(func(changes []Change) {
		cp := append([]Change(nil), changes...)
		mu.Lock()
		batches = append(batches, cp)
		mu.Unlock()
	})
	t.Cleanup(func() { SetChangeRecorder(nil) })

	now := "2026-10-03T21:30:00.000Z"
	mem := id(42)
	tx, err := db.BeginTx(ctx, nil)
	if err != nil {
		t.Fatal(err)
	}
	if _, err := tx.ExecContext(ctx, `
		INSERT INTO memories (id, mind_id, title, note, status, created_at, updated_at)
		VALUES (?, NULL, 'um', '', 'active', ?, ?)`, mem, now, now); err != nil {
		t.Fatal(err)
	}
	if _, err := tx.ExecContext(ctx, `UPDATE memories SET title = 'dois', rev = 2 WHERE id = ?`, mem); err != nil {
		t.Fatal(err)
	}
	if err := tx.Rollback(); err != nil {
		t.Fatal(err)
	}
	mu.Lock()
	if len(batches) != 0 {
		t.Fatalf("rollback emitted %#v", batches)
	}
	mu.Unlock()

	tx, err = db.BeginTx(ctx, nil)
	if err != nil {
		t.Fatal(err)
	}
	if _, err := tx.ExecContext(ctx, `
		INSERT INTO memories (id, mind_id, title, note, status, created_at, updated_at)
		VALUES (?, NULL, 'um', '', 'active', ?, ?)`, mem, now, now); err != nil {
		t.Fatal(err)
	}
	other := id(43)
	if _, err := tx.ExecContext(ctx, `
		INSERT INTO memories (id, mind_id, title, note, status, created_at, updated_at)
		VALUES (?, NULL, 'outro', '', 'active', ?, ?)`, other, now, now); err != nil {
		t.Fatal(err)
	}
	if err := tx.Commit(); err != nil {
		t.Fatal(err)
	}
	if _, err := db.ExecContext(ctx, `UPDATE memories SET title = 'tres', rev = 2, updated_at = ? WHERE id = ?`, now, mem); err != nil {
		t.Fatal(err)
	}
	if _, err := db.ExecContext(ctx, `DELETE FROM memories WHERE id = ?`, other); err != nil {
		t.Fatal(err)
	}

	mu.Lock()
	defer mu.Unlock()
	if len(batches) != 3 {
		t.Fatalf("batches = %d (%#v)", len(batches), batches)
	}
	wantCreate := []Change{{Entity: "memories", ID: mem, Op: "create"}, {Entity: "memories", ID: other, Op: "create"}}
	if !sameChanges(batches[0], wantCreate) {
		t.Fatalf("create batch = %#v", batches[0])
	}
	if !sameChanges(batches[1], []Change{{Entity: "memories", ID: mem, Op: "update"}}) {
		t.Fatalf("update batch = %#v", batches[1])
	}
	if !sameChanges(batches[2], []Change{{Entity: "memories", ID: other, Op: "delete"}}) {
		t.Fatalf("delete batch = %#v", batches[2])
	}
}

func TestOllamaProbe(t *testing.T) {
	probe, err := ProbeOllama(context.Background())
	if err != nil {
		t.Fatal(err)
	}
	if probe.Skipped {
		t.Logf("SKIP: %s", probe.Note)
		return
	}
	t.Logf("PASS: %s", probe.Note)
}

func openTemp(t *testing.T, ctx context.Context, hooks bool) *sql.DB {
	t.Helper()
	f, err := os.CreateTemp("", "sparky-store-*.sqlite")
	if err != nil {
		t.Fatal(err)
	}
	path := f.Name()
	if err := f.Close(); err != nil {
		t.Fatal(err)
	}
	t.Cleanup(func() { _ = os.Remove(path) })
	var db *sql.DB
	if hooks {
		db, err = OpenHooks(ctx, path)
	} else {
		db, err = Open(ctx, path)
	}
	if err != nil {
		t.Fatal(err)
	}
	t.Cleanup(func() { _ = db.Close() })
	return db
}

func containsAll(s string, parts ...string) bool {
	for _, p := range parts {
		if indexOf(s, p) < 0 {
			return false
		}
	}
	return true
}

func indexOf(s, part string) int {
	for i := 0; i+len(part) <= len(s); i++ {
		if s[i:i+len(part)] == part {
			return i
		}
	}
	return -1
}

func sameChanges(got, want []Change) bool {
	if len(got) != len(want) {
		return false
	}
	for i := range want {
		if got[i] != want[i] {
			return false
		}
	}
	return true
}
