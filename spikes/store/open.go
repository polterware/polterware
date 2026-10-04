package store

import (
	"context"
	"database/sql"
	_ "embed"
	"fmt"
	"strings"

	_ "modernc.org/sqlite"
)

//go:embed schema.sql
var baselineSQL string

const (
	plainDriver = "sqlite"
	hookDriver  = "sparky-spike-hooks"
)

// Open opens a file database with the Polterware/Sparky pragmas (WAL, foreign keys, busy timeout).
func Open(ctx context.Context, path string) (*sql.DB, error) {
	return open(ctx, plainDriver, path)
}

// OpenHooks opens a database whose connections record entity-level change events.
func OpenHooks(ctx context.Context, path string) (*sql.DB, error) {
	return open(ctx, hookDriver, path)
}

func open(ctx context.Context, driver, path string) (*sql.DB, error) {
	dsn := fmt.Sprintf("file:%s?_pragma=busy_timeout(5000)&_pragma=foreign_keys(1)", path)
	db, err := sql.Open(driver, dsn)
	if err != nil {
		return nil, fmt.Errorf("open sqlite: %w", err)
	}
	if _, err := db.ExecContext(ctx, `PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL; PRAGMA busy_timeout = 5000; PRAGMA synchronous = NORMAL;`); err != nil {
		_ = db.Close()
		return nil, fmt.Errorf("set pragmas: %w", err)
	}
	return db, nil
}

// ApplyBaseline applies the doc-02 DDL. Statements are executed one by one so a
// failure names the statement that SQLite rejected.
func ApplyBaseline(ctx context.Context, db *sql.DB) error {
	for i, stmt := range splitSQL(baselineSQL) {
		if _, err := db.ExecContext(ctx, stmt); err != nil {
			preview := stmt
			if len(preview) > 180 {
				preview = preview[:180]
			}
			return fmt.Errorf("baseline statement %d: %w (%s)", i+1, err, strings.TrimSpace(preview))
		}
	}
	return nil
}

func splitSQL(script string) []string {
	parts := strings.Split(script, ";")
	out := make([]string, 0, len(parts))
	for _, part := range parts {
		stmt := strings.TrimSpace(part)
		if stmt == "" {
			continue
		}
		out = append(out, stmt)
	}
	return out
}
