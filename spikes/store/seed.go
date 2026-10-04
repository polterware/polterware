package store

import (
	"context"
	"database/sql"
	"fmt"
	"os"
	"time"
)

// Words used to build FTS documents. Several carry Portuguese diacritics so
// remove_diacritics can be checked on the same corpus as the timings.
var Words = []string{
	"projeto", "reunião", "café", "memória", "agenda", "hoje", "amanhã",
	"focus", "review", "calendar", "attachment", "inbox", "mind", "sparky",
	"compras", "médico", "relatório", "deploy", "bug", "sprint",
	"não", "ação", "código", "tarefa", "lembrete",
}

// Corpus is a seeded baseline database.
type Corpus struct {
	DB        *sql.DB
	Path      string
	Memories  int
	MindID    string
	TagID     string
	Insert    time.Duration
	SizeBytes int64
}

// BuildCorpus creates a temp database, applies the baseline, and inserts n memories
// plus their FTS rows, one tag link, and two checklist rows.
func BuildCorpus(ctx context.Context, n int) (*Corpus, error) {
	f, err := os.CreateTemp("", "sparky-store-*.sqlite")
	if err != nil {
		return nil, fmt.Errorf("temp db: %w", err)
	}
	path := f.Name()
	if err := f.Close(); err != nil {
		return nil, fmt.Errorf("close temp db: %w", err)
	}
	db, err := Open(ctx, path)
	if err != nil {
		return nil, err
	}
	db.SetMaxOpenConns(1)
	if err := ApplyBaseline(ctx, db); err != nil {
		_ = db.Close()
		return nil, err
	}

	const mindCount = 20
	const tagCount = 30
	minds := make([]string, mindCount)
	for i := range minds {
		minds[i] = id(1_000 + i)
	}
	tags := make([]string, tagCount)
	for i := range tags {
		tags[i] = id(2_000 + i)
	}
	now := time.Date(2026, 1, 1, 0, 0, 0, 0, time.UTC).Format("2006-01-02T15:04:05.000Z")
	tx, err := db.BeginTx(ctx, nil)
	if err != nil {
		_ = db.Close()
		return nil, fmt.Errorf("begin seed: %w", err)
	}
	for i, mind := range minds {
		parent := any(nil)
		if i > 0 {
			parent = minds[i%4]
			if parent == mind {
				parent = nil
			}
		}
		if _, err := tx.ExecContext(ctx, `
			INSERT INTO minds (id, parent_id, name, color, icon, sort_order, created_at, updated_at, rev)
			VALUES (?, ?, ?, '#445566', 'brain', ?, ?, ?, 1)`,
			mind, parent, fmt.Sprintf("Mind %s", Words[i%len(Words)]), i, now, now); err != nil {
			_ = tx.Rollback()
			_ = db.Close()
			return nil, fmt.Errorf("insert mind: %w", err)
		}
	}
	for i, tag := range tags {
		if _, err := tx.ExecContext(ctx, `
			INSERT INTO tags (id, name, color, created_at, updated_at, rev)
			VALUES (?, ?, '#778899', ?, ?, 1)`,
			tag, fmt.Sprintf("tag-%s-%d", Words[i%len(Words)], i), now, now); err != nil {
			_ = tx.Rollback()
			_ = db.Close()
			return nil, fmt.Errorf("insert tag: %w", err)
		}
	}
	if err := tx.Commit(); err != nil {
		_ = db.Close()
		return nil, fmt.Errorf("commit dims: %w", err)
	}

	start := time.Now()
	if err := insertMemories(ctx, db, n, minds, tags); err != nil {
		_ = db.Close()
		return nil, err
	}
	elapsed := time.Since(start)
	if _, err := db.ExecContext(ctx, `PRAGMA wal_checkpoint(TRUNCATE)`); err != nil {
		_ = db.Close()
		return nil, fmt.Errorf("checkpoint: %w", err)
	}
	info, err := os.Stat(path)
	if err != nil {
		_ = db.Close()
		return nil, fmt.Errorf("stat db: %w", err)
	}
	return &Corpus{
		DB:        db,
		Path:      path,
		Memories:  n,
		MindID:    minds[0],
		TagID:     tags[0],
		Insert:    elapsed,
		SizeBytes: info.Size(),
	}, nil
}

func insertMemories(ctx context.Context, db *sql.DB, n int, minds, tags []string) error {
	const batch = 250
	for start := 0; start < n; start += batch {
		end := start + batch
		if end > n {
			end = n
		}
		tx, err := db.BeginTx(ctx, nil)
		if err != nil {
			return fmt.Errorf("begin memories: %w", err)
		}
		if err := insertMemoryBatch(ctx, tx, start, end, minds, tags); err != nil {
			_ = tx.Rollback()
			return err
		}
		if err := tx.Commit(); err != nil {
			return fmt.Errorf("commit memories: %w", err)
		}
	}
	return nil
}

func insertMemoryBatch(ctx context.Context, tx *sql.Tx, start, end int, minds, tags []string) error {
	memStmt, err := tx.PrepareContext(ctx, `
		INSERT INTO memories (
			id, mind_id, title, note, status, pinned, priority, due_at, due_all_day,
			sort_order, auto_complete_on_checklist, completed_at, created_at, updated_at, rev
		) VALUES (?, ?, ?, ?, ?, ?, ?, ?, 0, ?, 0, ?, ?, ?, 1)`)
	if err != nil {
		return fmt.Errorf("prepare memory: %w", err)
	}
	defer memStmt.Close()
	ftsStmt, err := tx.PrepareContext(ctx, `
		INSERT INTO memory_fts (memory_id, title, note, checklist, tags, mind)
		VALUES (?, ?, ?, ?, ?, ?)`)
	if err != nil {
		return fmt.Errorf("prepare fts: %w", err)
	}
	defer ftsStmt.Close()
	tagStmt, err := tx.PrepareContext(ctx, `INSERT INTO memory_tags (memory_id, tag_id) VALUES (?, ?)`)
	if err != nil {
		return fmt.Errorf("prepare tag link: %w", err)
	}
	defer tagStmt.Close()
	checkStmt, err := tx.PrepareContext(ctx, `
		INSERT INTO check_items (id, memory_id, title, detail, done, sort_order, completed_at, created_at, updated_at)
		VALUES (?, ?, ?, ?, 0, ?, NULL, ?, ?)`)
	if err != nil {
		return fmt.Errorf("prepare check: %w", err)
	}
	defer checkStmt.Close()

	for i := start; i < end; i++ {
		memID := id(100_000 + i)
		mind := minds[i%len(minds)]
		tag := tags[i%len(tags)]
		title := fmt.Sprintf("%s %s %d", Words[i%len(Words)], Words[(i/3)%len(Words)], i)
		note := noteFor(i)
		status := "active"
		var completed any
		if i%5 == 0 {
			status = "completed"
			completed = stamp(i)
		}
		pinned := 0
		if i%10 == 0 {
			pinned = 1
		}
		var due any
		if i%3 == 0 {
			due = stamp(i)
		}
		var priority any
		if i%4 != 0 {
			priority = (i % 3) + 1
		}
		created := stamp(i)
		checklist := fmt.Sprintf("%s item\ndetalhe %d", Words[(i+1)%len(Words)], i)
		mindName := fmt.Sprintf("Mind %s", Words[(i%len(minds))%len(Words)])
		tagName := fmt.Sprintf("tag-%s-%d", Words[(i%len(tags))%len(Words)], i%len(tags))
		if _, err := memStmt.ExecContext(ctx, memID, mind, title, note, status, pinned, priority, due, i, completed, created, created); err != nil {
			return fmt.Errorf("insert memory %d: %w", i, err)
		}
		if _, err := ftsStmt.ExecContext(ctx, memID, title, note, checklist, tagName, mindName); err != nil {
			return fmt.Errorf("insert fts %d: %w", i, err)
		}
		if _, err := tagStmt.ExecContext(ctx, memID, tag); err != nil {
			return fmt.Errorf("insert memory tag %d: %w", i, err)
		}
		for c := 0; c < 2; c++ {
			if _, err := checkStmt.ExecContext(ctx, id(5_000_000+i*2+c), memID, fmt.Sprintf("%s item %d", Words[(i+c)%len(Words)], c), "detail", c, created, created); err != nil {
				return fmt.Errorf("insert check %d: %w", i, err)
			}
		}
	}
	return nil
}

func noteFor(i int) string {
	a := Words[i%len(Words)]
	b := Words[(i+7)%len(Words)]
	c := Words[(i+13)%len(Words)]
	return fmt.Sprintf("%s sobre %s e %s. Nota %d para busca textual no Sparky.", a, b, c, i)
}

func stamp(i int) string {
	return time.Date(2026, 1, 1, 0, 0, 0, 0, time.UTC).Add(time.Duration(i) * time.Second).Format("2006-01-02T15:04:05.000Z")
}

func id(n int) string {
	const alpha = "0123456789ABCDEFGHJKMNPQRSTVWXYZ"
	var buf [26]byte
	for i := 25; i >= 0; i-- {
		buf[i] = alpha[n%32]
		n /= 32
	}
	return string(buf[:])
}
