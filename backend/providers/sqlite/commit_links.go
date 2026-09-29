package sqlite

import (
	"database/sql"
	"errors"
	"strings"
	"time"

	"github.com/polterware/polterware/backend/domains/work"
	"github.com/polterware/polterware/backend/platform/ids"
)

// ReplaceCommitLinks deletes every link for one commit evidence row and writes
// the replacement set. Commits outside the current scan window are left alone.
func (s *Store) ReplaceCommitLinks(projectID, evidenceID string, links []work.CommitLink, now time.Time) error {
	tx, err := s.DB.Begin()
	if err != nil {
		return err
	}
	defer func() { _ = tx.Rollback() }()
	if _, err := tx.Exec(`DELETE FROM work_commit_links WHERE project_id=? AND evidence_id=?`, projectID, evidenceID); err != nil {
		return err
	}
	stamp := now.UTC().Format(time.RFC3339)
	seen := map[string]bool{}
	for _, link := range links {
		if link.TaskID == "" || (link.Method != work.LinkExplicit && link.Method != work.LinkSuggested) {
			continue
		}
		key := link.TaskID + "\x00" + link.Method
		if seen[key] {
			continue
		}
		seen[key] = true
		id := link.ID
		if id == "" {
			id = ids.NewULID()
		}
		if _, err := tx.Exec(`INSERT INTO work_commit_links(id,project_id,evidence_id,task_id,method,created_at) VALUES(?,?,?,?,?,?)`,
			id, projectID, evidenceID, link.TaskID, link.Method, stamp); err != nil {
			return err
		}
	}
	return tx.Commit()
}

// ListCommitLinks returns every commit-to-task link for a project.
func (s *Store) ListCommitLinks(projectID string) ([]work.CommitLink, error) {
	rows, err := s.DB.Query(`SELECT id, project_id, evidence_id, task_id, method, created_at
		FROM work_commit_links WHERE project_id=? ORDER BY created_at, id`, projectID)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	var out []work.CommitLink
	for rows.Next() {
		var link work.CommitLink
		var created string
		if err := rows.Scan(&link.ID, &link.ProjectID, &link.EvidenceID, &link.TaskID, &link.Method, &created); err != nil {
			return nil, err
		}
		link.CreatedAt, _ = time.Parse(time.RFC3339, created)
		out = append(out, link)
	}
	return out, rows.Err()
}

// SearchWorkTasks returns task ids ranked by BM25 against the work full-text
// index. The caller keeps at most one as a suggestion.
func (s *Store) SearchWorkTasks(projectID, subject string, limit int) ([]string, error) {
	if limit <= 0 {
		limit = 5
	}
	// FTS5 treats "-" as a column filter, so a subject like "zzzz-unrelated-token"
	// has to be split into words before the query is built.
	query := sanitizeFTSQuery(strings.ReplaceAll(subject, "-", " "))
	if query == "" {
		return nil, nil
	}
	rows, err := s.DB.Query(`SELECT task_id FROM work_fts
		WHERE work_fts MATCH ? AND project_id=?
		ORDER BY bm25(work_fts) LIMIT ?`, query, projectID, limit)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	var ids []string
	for rows.Next() {
		var id string
		if err := rows.Scan(&id); err != nil {
			return nil, err
		}
		ids = append(ids, id)
	}
	return ids, rows.Err()
}

func reindexWorkCardFTS(tx *sql.Tx, cardID string) error {
	var projectID, slug, title, intent, status string
	err := tx.QueryRow(`SELECT project_id, slug, title, intent, status FROM work_cards WHERE id=?`, cardID).
		Scan(&projectID, &slug, &title, &intent, &status)
	if errors.Is(err, sql.ErrNoRows) {
		return nil
	}
	if err != nil {
		return err
	}
	rows, err := tx.Query(`SELECT id, title, state FROM work_tasks WHERE card_id=?`, cardID)
	if err != nil {
		return err
	}
	defer rows.Close()
	type taskRow struct {
		id, title, state string
	}
	var tasks []taskRow
	for rows.Next() {
		var task taskRow
		if err := rows.Scan(&task.id, &task.title, &task.state); err != nil {
			return err
		}
		tasks = append(tasks, task)
	}
	if err := rows.Err(); err != nil {
		return err
	}
	if err := rows.Close(); err != nil {
		return err
	}
	for _, task := range tasks {
		if _, err := tx.Exec(`DELETE FROM work_fts WHERE task_id=?`, task.id); err != nil {
			return err
		}
		if status != work.CardActive || task.state == work.TaskDropped {
			continue
		}
		document := strings.TrimSpace(task.title + " " + title + " " + intent)
		if _, err := tx.Exec(`INSERT INTO work_fts(task_id, project_id, card_slug, title) VALUES(?,?,?,?)`,
			task.id, projectID, slug, document); err != nil {
			return err
		}
	}
	return nil
}
