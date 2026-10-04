// Package search implements the FTS5 index over the core SQLite database.
package search

import (
	"context"
	"database/sql"
	"errors"
	"fmt"
	"regexp"
	"strings"
	"time"
	"unicode"

	domain "github.com/polterware/polterware/sparky/backend/domains/search"
)

type dbtx interface {
	ExecContext(context.Context, string, ...any) (sql.Result, error)
	QueryContext(context.Context, string, ...any) (*sql.Rows, error)
	QueryRowContext(context.Context, string, ...any) *sql.Row
}

// Index uses the caller-owned connection so callers can share their write transaction.
type Index struct{ db dbtx }

var _ domain.Index = (*Index)(nil)

func New(db dbtx) *Index { return &Index{db: db} }

func (i *Index) Upsert(ctx context.Context, doc domain.Document) error {
	if strings.TrimSpace(doc.MemoryID) == "" {
		return errors.New("search: memory id is required")
	}
	if _, err := i.db.ExecContext(ctx, `DELETE FROM memory_fts WHERE memory_id = ?`, doc.MemoryID); err != nil {
		return fmt.Errorf("search delete existing index row: %w", err)
	}
	_, err := i.db.ExecContext(ctx, `INSERT INTO memory_fts(memory_id,title,note,checklist,tags,mind) VALUES(?,?,?,?,?,?)`,
		doc.MemoryID, doc.Title, StripMarkdown(doc.Note), doc.Checklist, doc.Tags, doc.Mind)
	if err != nil {
		return fmt.Errorf("search insert index row: %w", err)
	}
	return nil
}

func (i *Index) Delete(ctx context.Context, memoryID string) error {
	if _, err := i.db.ExecContext(ctx, `DELETE FROM memory_fts WHERE memory_id = ?`, memoryID); err != nil {
		return fmt.Errorf("search delete index row: %w", err)
	}
	return nil
}

func (i *Index) Search(ctx context.Context, query string, filter domain.Filter) ([]domain.Hit, error) {
	match := BuildQuery(query)
	if match == "" {
		return []domain.Hit{}, nil
	}
	limit := filter.Limit
	if limit <= 0 {
		limit = 50
	}
	if limit > 500 {
		limit = 500
	}
	var where strings.Builder
	args := []any{match}
	where.WriteString(`memory_fts MATCH ?`)
	if filter.Status != "" {
		where.WriteString(` AND m.status = ?`)
		args = append(args, filter.Status)
	}
	if filter.MindID != nil && *filter.MindID != "" {
		if filter.IncludeDescendants {
			where.WriteString(` AND m.mind_id IN (WITH RECURSIVE tree(id) AS (SELECT id FROM minds WHERE id = ? UNION ALL SELECT minds.id FROM minds JOIN tree ON minds.parent_id=tree.id) SELECT id FROM tree)`)
		} else {
			where.WriteString(` AND m.mind_id = ?`)
		}
		args = append(args, *filter.MindID)
	}
	if len(filter.TagIDs) != 0 {
		marks := make([]string, len(filter.TagIDs))
		for n, id := range filter.TagIDs {
			marks[n] = "?"
			args = append(args, id)
		}
		where.WriteString(` AND EXISTS (SELECT 1 FROM memory_tags mt WHERE mt.memory_id=m.id AND mt.tag_id IN (` + strings.Join(marks, ",") + `))`)
	}
	appendTime := func(column string, op string, value *time.Time) {
		if value != nil {
			where.WriteString(` AND m.` + column + op + `?`)
			args = append(args, value.UTC().Format(time.RFC3339Nano))
		}
	}
	appendTime("due_at", ">=", filter.DueFrom)
	appendTime("due_at", "<=", filter.DueTo)
	appendTime("updated_at", ">=", filter.UpdatedFrom)
	appendTime("updated_at", "<=", filter.UpdatedTo)
	args = append(args, limit)
	sqlQuery := `SELECT memory_fts.memory_id, memory_fts.title, bm25(memory_fts,10.0,1.0,2.0,3.0,1.5), ` +
		`snippet(memory_fts,1,'<b>','</b>','…',12), ` +
		`snippet(memory_fts,2,'<b>','</b>','…',12), snippet(memory_fts,3,'<b>','</b>','…',12), ` +
		`snippet(memory_fts,4,'<b>','</b>','…',12), snippet(memory_fts,5,'<b>','</b>','…',12) ` +
		`FROM memory_fts JOIN memories m ON m.id=memory_fts.memory_id WHERE ` + where.String() + ` ORDER BY bm25(memory_fts,10.0,1.0,2.0,3.0,1.5), memory_fts.memory_id LIMIT ?`
	rows, err := i.db.QueryContext(ctx, sqlQuery, args...)
	if err != nil {
		return nil, fmt.Errorf("search query: %w", err)
	}
	defer rows.Close()
	var hits []domain.Hit
	for rows.Next() {
		var hit domain.Hit
		var snippets [5]string
		if err := rows.Scan(&hit.MemoryID, &hit.Title, &hit.Score, &snippets[0], &snippets[1], &snippets[2], &snippets[3], &snippets[4]); err != nil {
			return nil, fmt.Errorf("search scan result: %w", err)
		}
		for n, name := range []string{"title", "note", "checklist", "tags", "mind"} {
			if strings.Contains(snippets[n], "<b>") {
				hit.MatchedIn = append(hit.MatchedIn, name)
				if hit.Snippet == "" {
					hit.Snippet = snippets[n]
				}
			}
		}
		hits = append(hits, hit)
	}
	if err := rows.Err(); err != nil {
		return nil, fmt.Errorf("search iterate results: %w", err)
	}
	return hits, nil
}

func (i *Index) Rebuild(ctx context.Context, docs []domain.Document) error {
	tx, ok := i.db.(interface {
		BeginTx(context.Context, *sql.TxOptions) (*sql.Tx, error)
	})
	if !ok {
		return errors.New("search: rebuild requires a transactional database handle")
	}
	dbtx, err := tx.BeginTx(ctx, nil)
	if err != nil {
		return fmt.Errorf("search begin rebuild: %w", err)
	}
	defer dbtx.Rollback()
	index := New(dbtx)
	if _, err := dbtx.ExecContext(ctx, `DELETE FROM memory_fts`); err != nil {
		return fmt.Errorf("search clear index: %w", err)
	}
	for _, doc := range docs {
		if err := index.Upsert(ctx, doc); err != nil {
			return err
		}
	}
	if err := dbtx.Commit(); err != nil {
		return fmt.Errorf("search commit rebuild: %w", err)
	}
	return nil
}

func (i *Index) Count(ctx context.Context) (int, error) {
	var count int
	if err := i.db.QueryRowContext(ctx, `SELECT count(*) FROM memory_fts`).Scan(&count); err != nil {
		return 0, fmt.Errorf("search count index: %w", err)
	}
	return count, nil
}

var words = regexp.MustCompile(`[\p{L}\p{N}_]+`)

// BuildQuery quotes all user terms, making FTS operators ordinary searchable words.
// An explicitly quoted query remains a phrase; otherwise the final term is a prefix.
func BuildQuery(input string) string {
	input = strings.TrimSpace(input)
	if len(input) >= 2 && input[0] == '"' && input[len(input)-1] == '"' {
		phrase := strings.TrimSpace(input[1 : len(input)-1])
		if phrase == "" {
			return ""
		}
		return `"` + strings.ReplaceAll(phrase, `"`, `""`) + `"`
	}
	terms := words.FindAllString(input, -1)
	for n := range terms {
		terms[n] = `"` + strings.ReplaceAll(terms[n], `"`, `""`) + `"`
	}
	if len(terms) == 0 {
		return ""
	}
	terms[len(terms)-1] += "*"
	return strings.Join(terms, " AND ")
}

var markdownMarkup = regexp.MustCompile(`<[^>]*>|!?\[([^\]]*)\]\([^)]*\)|[*_` + "`" + `~>#]`)

// StripMarkdown removes common Markdown structure while retaining visible link labels and text.
func StripMarkdown(markdown string) string {
	markdown = strings.ReplaceAll(markdown, "\r\n", "\n")
	markdown = markdownMarkup.ReplaceAllString(markdown, "$1")
	markdown = strings.Map(func(r rune) rune {
		if unicode.IsControl(r) && r != '\n' && r != '\t' {
			return -1
		}
		return r
	}, markdown)
	return markdown
}
