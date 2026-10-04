package sqlite

import (
	"context"
	"database/sql"
	"fmt"
	"strings"

	"github.com/polterware/polterware/sparky/backend/domains/search"
)

// DocumentSource projects a memory's indexed text for search.Index.
type DocumentSource struct {
	store *Store
}

func (s *Store) Documents() DocumentSource { return DocumentSource{store: s} }

func (d DocumentSource) Document(ctx context.Context, memoryID string) (search.Document, error) {
	return d.store.document(ctx, memoryID)
}

func (s *Store) document(ctx context.Context, memoryID string) (search.Document, error) {
	q := s.q(ctx, false)
	var title, note string
	var mindID sql.NullString
	if err := q.QueryRowContext(ctx, `SELECT title, note, mind_id FROM memories WHERE id=?`, memoryID).
		Scan(&title, &note, &mindID); err != nil {
		return search.Document{}, mapSQL(err, "memory", memoryID)
	}
	path := ""
	if mindID.Valid {
		p, err := s.mindPath(ctx, mindID.String)
		if err != nil {
			return search.Document{}, err
		}
		path = p
	}
	checklist, err := s.checklistText(ctx, memoryID)
	if err != nil {
		return search.Document{}, err
	}
	tags, err := s.tagText(ctx, memoryID)
	if err != nil {
		return search.Document{}, err
	}
	return search.Document{
		MemoryID:  memoryID,
		Title:     title,
		Note:      stripMarkdown(note),
		Checklist: checklist,
		Tags:      tags,
		Mind:      path,
	}, nil
}

func (s *Store) mindPath(ctx context.Context, mindID string) (string, error) {
	rows, err := s.q(ctx, false).QueryContext(ctx, `
		WITH RECURSIVE chain AS (
			SELECT id, parent_id, name, 0 AS depth FROM minds WHERE id = ?
			UNION ALL
			SELECT m.id, m.parent_id, m.name, c.depth + 1
			FROM minds m JOIN chain c ON c.parent_id = m.id
		)
		SELECT name FROM chain ORDER BY depth DESC`, mindID)
	if err != nil {
		return "", err
	}
	defer rows.Close()
	var names []string
	for rows.Next() {
		var name string
		if err := rows.Scan(&name); err != nil {
			return "", err
		}
		names = append(names, name)
	}
	return strings.Join(names, " / "), rows.Err()
}

func (s *Store) checklistText(ctx context.Context, memoryID string) (string, error) {
	rows, err := s.q(ctx, false).QueryContext(ctx, `
		SELECT title, detail FROM check_items WHERE memory_id=? ORDER BY sort_order, id`, memoryID)
	if err != nil {
		return "", err
	}
	defer rows.Close()
	var parts []string
	for rows.Next() {
		var title, detail string
		if err := rows.Scan(&title, &detail); err != nil {
			return "", err
		}
		if detail != "" {
			parts = append(parts, title+"\n"+detail)
		} else {
			parts = append(parts, title)
		}
	}
	return strings.Join(parts, "\n"), rows.Err()
}

func (s *Store) tagText(ctx context.Context, memoryID string) (string, error) {
	rows, err := s.q(ctx, false).QueryContext(ctx, `
		SELECT t.name FROM memory_tags mt JOIN tags t ON t.id = mt.tag_id
		WHERE mt.memory_id=? ORDER BY t.name COLLATE NOCASE`, memoryID)
	if err != nil {
		return "", err
	}
	defer rows.Close()
	var names []string
	for rows.Next() {
		var name string
		if err := rows.Scan(&name); err != nil {
			return "", err
		}
		names = append(names, name)
	}
	return strings.Join(names, " "), rows.Err()
}

func (s *Store) syncIndex(ctx context.Context, memoryID string, deleted bool) error {
	if s.index == nil {
		return nil
	}
	if deleted {
		return s.index.Delete(ctx, memoryID)
	}
	doc, err := s.document(ctx, memoryID)
	if err != nil {
		return err
	}
	return s.index.Upsert(ctx, doc)
}

func stripMarkdown(s string) string {
	s = strings.ReplaceAll(s, "\r\n", "\n")
	var b strings.Builder
	inFence := false
	for _, line := range strings.Split(s, "\n") {
		trim := strings.TrimSpace(line)
		if strings.HasPrefix(trim, "```") {
			inFence = !inFence
			continue
		}
		if inFence {
			b.WriteString(trim)
			b.WriteByte('\n')
			continue
		}
		line = strings.TrimLeft(line, "#>* \t")
		line = strings.ReplaceAll(line, "**", "")
		line = strings.ReplaceAll(line, "__", "")
		line = strings.ReplaceAll(line, "*", "")
		line = strings.ReplaceAll(line, "_", "")
		line = strings.ReplaceAll(line, "`", "")
		b.WriteString(line)
		b.WriteByte('\n')
	}
	return strings.TrimSpace(b.String())
}

func (s *Store) reindexMindMemories(ctx context.Context, mindID string) error {
	if s.index == nil {
		return nil
	}
	rows, err := s.q(ctx, false).QueryContext(ctx, `
		WITH RECURSIVE tree AS (
			SELECT id FROM minds WHERE id = ?
			UNION ALL
			SELECT m.id FROM minds m JOIN tree t ON m.parent_id = t.id
		)
		SELECT id FROM memories WHERE mind_id IN (SELECT id FROM tree)`, mindID)
	if err != nil {
		return err
	}
	defer rows.Close()
	var ids []string
	for rows.Next() {
		var id string
		if err := rows.Scan(&id); err != nil {
			return err
		}
		ids = append(ids, id)
	}
	if err := rows.Err(); err != nil {
		return err
	}
	for _, id := range ids {
		if err := s.syncIndex(ctx, id, false); err != nil {
			return fmt.Errorf("reindex %s: %w", id, err)
		}
	}
	return nil
}
