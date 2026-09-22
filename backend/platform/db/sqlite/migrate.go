package sqlite

import (
	"database/sql"
	"embed"
	"fmt"
	"sort"
	"strings"
	"time"
)

//go:embed migrations/*.sql
var migrationFS embed.FS

func Migrate(db *sql.DB) error {
	if _, err := db.Exec(`CREATE TABLE IF NOT EXISTS schema_migrations (
		version INTEGER PRIMARY KEY,
		name TEXT NOT NULL,
		applied_at TEXT NOT NULL
	)`); err != nil {
		return err
	}
	// Rebuild an incompatible legacy baseline before applying pending SQL.
	// Pending migrations may assume the current first-version tables exist.
	if err := ensureBaselineCompatible(db); err != nil {
		return err
	}
	return applyPendingMigrations(db)
}

func applyPendingMigrations(db *sql.DB) error {
	entries, err := migrationFS.ReadDir("migrations")
	if err != nil {
		return err
	}
	var names []string
	for _, e := range entries {
		if !e.IsDir() && strings.HasSuffix(e.Name(), ".sql") {
			names = append(names, e.Name())
		}
	}
	sort.Strings(names)
	for _, name := range names {
		var ver int
		if _, err := fmt.Sscanf(name, "%d_", &ver); err != nil {
			return fmt.Errorf("migration name %s: %w", name, err)
		}
		var exists int
		if err := db.QueryRow(`SELECT COUNT(1) FROM schema_migrations WHERE version=?`, ver).Scan(&exists); err != nil {
			return err
		}
		if exists > 0 {
			continue
		}
		body, err := migrationFS.ReadFile("migrations/" + name)
		if err != nil {
			return err
		}
		tx, err := db.Begin()
		if err != nil {
			return err
		}
		if _, err := tx.Exec(string(body)); err != nil {
			_ = tx.Rollback()
			return fmt.Errorf("apply %s: %w", name, err)
		}
		if _, err := tx.Exec(`INSERT INTO schema_migrations(version, name, applied_at) VALUES (?,?,?)`,
			ver, name, time.Now().UTC().Format(time.RFC3339)); err != nil {
			_ = tx.Rollback()
			return err
		}
		if err := tx.Commit(); err != nil {
			return err
		}
	}
	return nil
}

// ensureBaselineCompatible rebuilds the database when an older projects table is
// present without archive columns. Polterware has a single clean baseline and
// does not migrate legacy local databases in place.
func ensureBaselineCompatible(db *sql.DB) error {
	compatible, err := projectsBaselineCompatible(db)
	if err != nil {
		return err
	}
	if compatible {
		return nil
	}
	if err := dropAllTables(db); err != nil {
		return fmt.Errorf("rebuild incompatible baseline: %w", err)
	}
	if _, err := db.Exec(`CREATE TABLE IF NOT EXISTS schema_migrations (
		version INTEGER PRIMARY KEY,
		name TEXT NOT NULL,
		applied_at TEXT NOT NULL
	)`); err != nil {
		return err
	}
	return applyPendingMigrations(db)
}

func projectsBaselineCompatible(db *sql.DB) (bool, error) {
	var tableCount int
	if err := db.QueryRow(`SELECT COUNT(1) FROM sqlite_master WHERE type='table' AND name='projects'`).Scan(&tableCount); err != nil {
		return false, err
	}
	if tableCount == 0 {
		return true, nil
	}
	hasStatus, err := tableHasColumn(db, "projects", "status")
	if err != nil {
		return false, err
	}
	hasArchivedAt, err := tableHasColumn(db, "projects", "archived_at")
	if err != nil {
		return false, err
	}
	return hasStatus && hasArchivedAt, nil
}

func tableHasColumn(db *sql.DB, table, column string) (bool, error) {
	rows, err := db.Query(`PRAGMA table_info(` + table + `)`)
	if err != nil {
		return false, err
	}
	defer rows.Close()
	for rows.Next() {
		var cid int
		var name, typ string
		var notnull, pk int
		var dflt sql.NullString
		if err := rows.Scan(&cid, &name, &typ, &notnull, &dflt, &pk); err != nil {
			return false, err
		}
		if name == column {
			return true, nil
		}
	}
	return false, rows.Err()
}

func dropAllTables(db *sql.DB) error {
	rows, err := db.Query(`SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'`)
	if err != nil {
		return err
	}
	var names []string
	for rows.Next() {
		var name string
		if err := rows.Scan(&name); err != nil {
			_ = rows.Close()
			return err
		}
		names = append(names, name)
	}
	if err := rows.Close(); err != nil {
		return err
	}
	if err := rows.Err(); err != nil {
		return err
	}
	tx, err := db.Begin()
	if err != nil {
		return err
	}
	if _, err := tx.Exec(`PRAGMA foreign_keys = OFF`); err != nil {
		_ = tx.Rollback()
		return err
	}
	for _, name := range names {
		if _, err := tx.Exec(`DROP TABLE IF EXISTS "` + strings.ReplaceAll(name, `"`, `""`) + `"`); err != nil {
			_ = tx.Rollback()
			return err
		}
	}
	if _, err := tx.Exec(`PRAGMA foreign_keys = ON`); err != nil {
		_ = tx.Rollback()
		return err
	}
	return tx.Commit()
}

func AppliedVersions(db *sql.DB) ([]int, error) {
	rows, err := db.Query(`SELECT version FROM schema_migrations ORDER BY version`)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	var out []int
	for rows.Next() {
		var v int
		if err := rows.Scan(&v); err != nil {
			return nil, err
		}
		out = append(out, v)
	}
	return out, rows.Err()
}
