package sqlite

import (
	"context"
	"database/sql"
	"fmt"
	"strings"
	"time"

	"github.com/polterware/polterware/sparky/backend/domains/fault"
	"github.com/polterware/polterware/sparky/backend/domains/memory"
)

const memoryCols = `id, mind_id, title, note, status, pinned, priority, due_at, due_all_day, sort_order, auto_complete_on_checklist, completed_at, created_at, updated_at, rev`

func (r Memories) Get(ctx context.Context, id string) (memory.Memory, error) {
	m, err := scanMemory(r.q(ctx, false).QueryRowContext(ctx, `SELECT `+memoryCols+` FROM memories WHERE id=?`, id))
	if err != nil {
		return memory.Memory{}, mapSQL(err, "memory", id)
	}
	return m, nil
}

func (r Memories) Insert(ctx context.Context, m memory.Memory) error {
	return r.Within(ctx, func(ctx context.Context) error {
		_, err := r.q(ctx, true).ExecContext(ctx, `
			INSERT INTO memories(id, mind_id, title, note, status, pinned, priority, due_at, due_all_day, sort_order, auto_complete_on_checklist, completed_at, created_at, updated_at, rev)
			VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)`,
			m.ID, nullString(m.MindID), m.Title, m.Note, m.Status, boolInt(m.Pinned), nullInt(m.Priority),
			nullTime(m.DueAt), boolInt(m.DueAllDay), m.SortOrder, boolInt(m.AutoCompleteOnChecklist),
			nullTime(m.CompletedAt), formatTime(m.CreatedAt), formatTime(m.UpdatedAt), m.Rev)
		if err != nil {
			return mapSQL(err, "memory", m.ID)
		}
		return r.syncIndex(ctx, m.ID, false)
	})
}

func (r Memories) Update(ctx context.Context, expectedRev int64, m memory.Memory) error {
	return r.Within(ctx, func(ctx context.Context) error {
		current, err := r.Get(ctx, m.ID)
		if err != nil {
			return err
		}
		if expectedRev != 0 && current.Rev != expectedRev {
			return updateConflict("memory", m.ID, current.Rev, current)
		}
		res, err := r.q(ctx, true).ExecContext(ctx, `
			UPDATE memories SET mind_id=?, title=?, note=?, status=?, pinned=?, priority=?, due_at=?, due_all_day=?, sort_order=?, auto_complete_on_checklist=?, completed_at=?, updated_at=?, rev=?
			WHERE id=?`,
			nullString(m.MindID), m.Title, m.Note, m.Status, boolInt(m.Pinned), nullInt(m.Priority),
			nullTime(m.DueAt), boolInt(m.DueAllDay), m.SortOrder, boolInt(m.AutoCompleteOnChecklist),
			nullTime(m.CompletedAt), formatTime(m.UpdatedAt), m.Rev, m.ID)
		if err != nil {
			return err
		}
		n, _ := res.RowsAffected()
		if n == 0 {
			return fault.NotFound("memory", m.ID)
		}
		return r.syncIndex(ctx, m.ID, false)
	})
}

func (r Memories) Delete(ctx context.Context, expectedRev int64, id string) error {
	return r.Within(ctx, func(ctx context.Context) error {
		current, err := r.Get(ctx, id)
		if err != nil {
			return err
		}
		if expectedRev != 0 && current.Rev != expectedRev {
			return updateConflict("memory", id, current.Rev, current)
		}
		if err := r.syncIndex(ctx, id, true); err != nil {
			return err
		}
		_, err = r.q(ctx, true).ExecContext(ctx, `DELETE FROM memories WHERE id=?`, id)
		return err
	})
}

func (r Memories) ListCheckItems(ctx context.Context, memoryID string) ([]memory.CheckItem, error) {
	rows, err := r.q(ctx, false).QueryContext(ctx, `
		SELECT id, memory_id, title, detail, done, sort_order, completed_at, created_at, updated_at
		FROM check_items WHERE memory_id=? ORDER BY sort_order, id`, memoryID)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	out := []memory.CheckItem{}
	for rows.Next() {
		item, err := scanCheckItem(rows)
		if err != nil {
			return nil, err
		}
		out = append(out, item)
	}
	return out, rows.Err()
}

func (r Memories) ReplaceCheckItems(ctx context.Context, memoryID string, items []memory.CheckItem) error {
	return r.Within(ctx, func(ctx context.Context) error {
		if _, err := r.Get(ctx, memoryID); err != nil {
			return err
		}
		if _, err := r.q(ctx, true).ExecContext(ctx, `DELETE FROM check_items WHERE memory_id=?`, memoryID); err != nil {
			return err
		}
		for _, item := range items {
			if item.MemoryID == "" {
				item.MemoryID = memoryID
			}
			if _, err := r.q(ctx, true).ExecContext(ctx, `
				INSERT INTO check_items(id, memory_id, title, detail, done, sort_order, completed_at, created_at, updated_at)
				VALUES (?,?,?,?,?,?,?,?,?)`,
				item.ID, item.MemoryID, item.Title, item.Detail, boolInt(item.Done), item.SortOrder,
				nullTime(item.CompletedAt), formatTime(item.CreatedAt), formatTime(item.UpdatedAt)); err != nil {
				return mapSQL(err, "check_item", item.ID)
			}
		}
		return r.syncIndex(ctx, memoryID, false)
	})
}

func (r Memories) GetCheckItem(ctx context.Context, id string) (memory.CheckItem, error) {
	item, err := scanCheckItem(r.q(ctx, false).QueryRowContext(ctx, `
		SELECT id, memory_id, title, detail, done, sort_order, completed_at, created_at, updated_at
		FROM check_items WHERE id=?`, id))
	if err != nil {
		return memory.CheckItem{}, mapSQL(err, "check_item", id)
	}
	return item, nil
}

func (r Memories) UpdateCheckItem(ctx context.Context, item memory.CheckItem) error {
	return r.Within(ctx, func(ctx context.Context) error {
		res, err := r.q(ctx, true).ExecContext(ctx, `
			UPDATE check_items SET title=?, detail=?, done=?, sort_order=?, completed_at=?, updated_at=?
			WHERE id=?`,
			item.Title, item.Detail, boolInt(item.Done), item.SortOrder, nullTime(item.CompletedAt),
			formatTime(item.UpdatedAt), item.ID)
		if err != nil {
			return err
		}
		n, _ := res.RowsAffected()
		if n == 0 {
			return fault.NotFound("check_item", item.ID)
		}
		return r.syncIndex(ctx, item.MemoryID, false)
	})
}

func (r Memories) ListTagIDs(ctx context.Context, memoryID string) ([]string, error) {
	rows, err := r.q(ctx, false).QueryContext(ctx, `SELECT tag_id FROM memory_tags WHERE memory_id=? ORDER BY tag_id`, memoryID)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	ids := []string{}
	for rows.Next() {
		var id string
		if err := rows.Scan(&id); err != nil {
			return nil, err
		}
		ids = append(ids, id)
	}
	return ids, rows.Err()
}

func (r Memories) SetTagIDs(ctx context.Context, memoryID string, tagIDs []string) error {
	return r.Within(ctx, func(ctx context.Context) error {
		if _, err := r.Get(ctx, memoryID); err != nil {
			return err
		}
		if _, err := r.q(ctx, true).ExecContext(ctx, `DELETE FROM memory_tags WHERE memory_id=?`, memoryID); err != nil {
			return err
		}
		for _, tagID := range tagIDs {
			if _, err := r.q(ctx, true).ExecContext(ctx, `INSERT INTO memory_tags(memory_id, tag_id) VALUES (?,?)`, memoryID, tagID); err != nil {
				return mapSQL(err, "memory_tag", memoryID+"/"+tagID)
			}
		}
		return r.syncIndex(ctx, memoryID, false)
	})
}

func (r Memories) ListCompletions(ctx context.Context, memoryID string) ([]memory.Completion, error) {
	rows, err := r.q(ctx, false).QueryContext(ctx, `
		SELECT id, memory_id, occurrence_at, completed_at FROM completions WHERE memory_id=? ORDER BY occurrence_at`, memoryID)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	out := []memory.Completion{}
	for rows.Next() {
		c, err := scanCompletion(rows)
		if err != nil {
			return nil, err
		}
		out = append(out, c)
	}
	return out, rows.Err()
}

func (r Memories) InsertCompletion(ctx context.Context, c memory.Completion) error {
	return r.Within(ctx, func(ctx context.Context) error {
		_, err := r.q(ctx, true).ExecContext(ctx, `
			INSERT INTO completions(id, memory_id, occurrence_at, completed_at) VALUES (?,?,?,?)`,
			c.ID, c.MemoryID, formatTime(c.OccurrenceAt), formatTime(c.CompletedAt))
		return mapSQL(err, "completion", c.ID)
	})
}

func (r Memories) DeleteCompletion(ctx context.Context, memoryID string, occurrenceAt time.Time) error {
	return r.Within(ctx, func(ctx context.Context) error {
		res, err := r.q(ctx, true).ExecContext(ctx, `DELETE FROM completions WHERE memory_id=? AND occurrence_at=?`,
			memoryID, formatTime(occurrenceAt))
		if err != nil {
			return err
		}
		n, _ := res.RowsAffected()
		if n == 0 {
			return fault.NotFound("completion", memoryID)
		}
		return nil
	})
}

func (r Memories) CountByMinds(ctx context.Context, mindIDs []string) (int, error) {
	if len(mindIDs) == 0 {
		return 0, nil
	}
	placeholders := strings.Repeat("?,", len(mindIDs))
	placeholders = placeholders[:len(placeholders)-1]
	args := make([]any, len(mindIDs))
	for i, id := range mindIDs {
		args[i] = id
	}
	var n int
	err := r.q(ctx, false).QueryRowContext(ctx, `SELECT COUNT(*) FROM memories WHERE mind_id IN (`+placeholders+`)`, args...).Scan(&n)
	return n, err
}

func scanMemory(row interface{ Scan(...any) error }) (memory.Memory, error) {
	var m memory.Memory
	var mindID, dueAt, completedAt sql.NullString
	var pinned, dueAllDay, auto int
	var priority sql.NullInt64
	var created, updated string
	if err := row.Scan(&m.ID, &mindID, &m.Title, &m.Note, &m.Status, &pinned, &priority, &dueAt, &dueAllDay, &m.SortOrder, &auto, &completedAt, &created, &updated, &m.Rev); err != nil {
		return memory.Memory{}, err
	}
	m.MindID = scanString(mindID)
	m.Pinned = pinned != 0
	m.Priority = scanInt(priority)
	due, err := scanTime(dueAt)
	if err != nil {
		return memory.Memory{}, err
	}
	m.DueAt = due
	m.DueAllDay = dueAllDay != 0
	m.AutoCompleteOnChecklist = auto != 0
	done, err := scanTime(completedAt)
	if err != nil {
		return memory.Memory{}, err
	}
	m.CompletedAt = done
	m.CreatedAt = mustTime(created)
	m.UpdatedAt = mustTime(updated)
	return m, nil
}

func scanCheckItem(row interface{ Scan(...any) error }) (memory.CheckItem, error) {
	var item memory.CheckItem
	var done int
	var completed sql.NullString
	var created, updated string
	if err := row.Scan(&item.ID, &item.MemoryID, &item.Title, &item.Detail, &done, &item.SortOrder, &completed, &created, &updated); err != nil {
		return memory.CheckItem{}, err
	}
	item.Done = done != 0
	t, err := scanTime(completed)
	if err != nil {
		return memory.CheckItem{}, err
	}
	item.CompletedAt = t
	item.CreatedAt = mustTime(created)
	item.UpdatedAt = mustTime(updated)
	return item, nil
}

func scanCompletion(row interface{ Scan(...any) error }) (memory.Completion, error) {
	var c memory.Completion
	var occ, done string
	if err := row.Scan(&c.ID, &c.MemoryID, &occ, &done); err != nil {
		return memory.Completion{}, err
	}
	c.OccurrenceAt = mustTime(occ)
	c.CompletedAt = mustTime(done)
	return c, nil
}

func snippetOf(note string) string {
	note = strings.TrimSpace(strings.ReplaceAll(note, "\n", " "))
	if len([]rune(note)) <= 120 {
		return note
	}
	return string([]rune(note)[:120])
}

func scheduleLabel(freq sql.NullString) string {
	if !freq.Valid {
		return ""
	}
	if freq.String == "" {
		return "once"
	}
	return freq.String
}

func (r Memories) List(ctx context.Context, filter memory.ListFilter) (memory.ListPage, error) {
	limit := clampLimit(filter.Limit, 50, 500)
	order := filter.Order
	if order == "" {
		order = memory.OrderUpdated
	}
	where, args, err := r.listWhere(ctx, filter)
	if err != nil {
		return memory.ListPage{}, err
	}
	cur, err := decodeCursor(filter.Cursor)
	if err != nil {
		return memory.ListPage{}, fault.Invalid(fault.Violation{Field: "cursor", Reason: err.Error()})
	}
	countWhere, countArgs := where, append([]any{}, args...)
	orderSQL, cursorSQL, cursorArgs := listOrder(order, cur)
	if cursorSQL != "" {
		where += " AND " + cursorSQL
		args = append(args, cursorArgs...)
	}
	var total int
	if err := r.q(ctx, false).QueryRowContext(ctx, `SELECT COUNT(*) FROM memories m WHERE `+countWhere, countArgs...).Scan(&total); err != nil {
		return memory.ListPage{}, err
	}
	query := fmt.Sprintf(`
		SELECT m.id, m.mind_id, m.title, m.note, m.status, m.pinned, m.priority, m.due_at, m.due_all_day,
		       m.sort_order, m.created_at, m.updated_at, m.rev,
		       (SELECT freq FROM schedules s WHERE s.memory_id = m.id)
		FROM memories m WHERE %s %s LIMIT ?`, where, orderSQL)
	queryArgs := append(append([]any{}, args...), limit+1)
	rows, err := r.q(ctx, false).QueryContext(ctx, query, queryArgs...)
	if err != nil {
		return memory.ListPage{}, err
	}
	defer rows.Close()
	type row struct {
		sum  memory.Summary
		note string
		key  string
	}
	var collected []row
	for rows.Next() {
		var sum memory.Summary
		var mindID, dueAt, freq sql.NullString
		var pinned, dueAllDay int
		var priority sql.NullInt64
		var created, updated, note string
		if err := rows.Scan(&sum.ID, &mindID, &sum.Title, &note, &sum.Status, &pinned, &priority, &dueAt, &dueAllDay, &sum.SortOrder, &created, &updated, &sum.Rev, &freq); err != nil {
			return memory.ListPage{}, err
		}
		sum.MindID = scanString(mindID)
		sum.Pinned = pinned != 0
		sum.Priority = scanInt(priority)
		due, err := scanTime(dueAt)
		if err != nil {
			return memory.ListPage{}, err
		}
		sum.DueAt = due
		sum.DueAllDay = dueAllDay != 0
		sum.CreatedAt = mustTime(created)
		sum.UpdatedAt = mustTime(updated)
		sum.Snippet = snippetOf(note)
		sum.ScheduleLabel = scheduleLabel(freq)
		collected = append(collected, row{sum: sum, note: note, key: cursorKey(order, sum)})
	}
	if err := rows.Err(); err != nil {
		return memory.ListPage{}, err
	}
	next := ""
	if len(collected) > limit {
		last := collected[limit-1]
		next = encodeCursor(listCursor{Key: last.key, ID: last.sum.ID})
		collected = collected[:limit]
	}
	ids := make([]string, len(collected))
	page := make([]memory.Summary, len(collected))
	for i, c := range collected {
		ids[i] = c.sum.ID
		page[i] = c.sum
		page[i].TagIDs = []string{}
	}
	tags, err := r.tagsFor(ctx, ids)
	if err != nil {
		return memory.ListPage{}, err
	}
	for i := range page {
		if t := tags[page[i].ID]; t != nil {
			page[i].TagIDs = t
		}
	}
	return memory.ListPage{Memories: page, NextCursor: next, Total: total}, nil
}

func (r Memories) tagsFor(ctx context.Context, ids []string) (map[string][]string, error) {
	out := map[string][]string{}
	if len(ids) == 0 {
		return out, nil
	}
	ph := strings.Repeat("?,", len(ids))
	ph = ph[:len(ph)-1]
	args := make([]any, len(ids))
	for i, id := range ids {
		args[i] = id
	}
	rows, err := r.q(ctx, false).QueryContext(ctx, `SELECT memory_id, tag_id FROM memory_tags WHERE memory_id IN (`+ph+`) ORDER BY tag_id`, args...)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	for rows.Next() {
		var mem, tag string
		if err := rows.Scan(&mem, &tag); err != nil {
			return nil, err
		}
		out[mem] = append(out[mem], tag)
	}
	return out, rows.Err()
}

func (r Memories) listWhere(ctx context.Context, filter memory.ListFilter) (string, []any, error) {
	where := "1=1"
	var args []any
	if filter.InboxOnly {
		where += " AND m.mind_id IS NULL"
	} else if filter.MindID != nil {
		if filter.IncludeDescendants {
			where += ` AND m.mind_id IN (
				WITH RECURSIVE tree AS (
					SELECT id FROM minds WHERE id = ?
					UNION ALL
					SELECT x.id FROM minds x JOIN tree t ON x.parent_id = t.id
				) SELECT id FROM tree)`
			args = append(args, *filter.MindID)
		} else {
			where += " AND m.mind_id = ?"
			args = append(args, *filter.MindID)
		}
	}
	if filter.Status != "" {
		where += " AND m.status = ?"
		args = append(args, filter.Status)
	}
	if filter.Pinned != nil {
		where += " AND m.pinned = ?"
		args = append(args, boolInt(*filter.Pinned))
	}
	if len(filter.TagIDs) > 0 {
		ph := strings.Repeat("?,", len(filter.TagIDs))
		ph = ph[:len(ph)-1]
		where += ` AND (SELECT COUNT(DISTINCT tag_id) FROM memory_tags WHERE memory_id = m.id AND tag_id IN (` + ph + `)) = ?`
		for _, id := range filter.TagIDs {
			args = append(args, id)
		}
		args = append(args, len(filter.TagIDs))
	}
	if filter.DueFrom != nil {
		where += " AND m.due_at >= ?"
		args = append(args, formatTime(*filter.DueFrom))
	}
	if filter.DueTo != nil {
		where += " AND m.due_at <= ?"
		args = append(args, formatTime(*filter.DueTo))
	}
	if filter.UpdatedFrom != nil {
		where += " AND m.updated_at >= ?"
		args = append(args, formatTime(*filter.UpdatedFrom))
	}
	if filter.UpdatedTo != nil {
		where += " AND m.updated_at <= ?"
		args = append(args, formatTime(*filter.UpdatedTo))
	}
	if filter.HasSchedule != nil {
		if *filter.HasSchedule {
			where += " AND EXISTS (SELECT 1 FROM schedules s WHERE s.memory_id = m.id)"
		} else {
			where += " AND NOT EXISTS (SELECT 1 FROM schedules s WHERE s.memory_id = m.id)"
		}
	}
	if filter.HasPlace != nil {
		if *filter.HasPlace {
			where += " AND EXISTS (SELECT 1 FROM places p WHERE p.memory_id = m.id)"
		} else {
			where += " AND NOT EXISTS (SELECT 1 FROM places p WHERE p.memory_id = m.id)"
		}
	}
	if q := strings.TrimSpace(filter.Query); q != "" {
		where += " AND m.id IN (SELECT memory_id FROM memory_fts WHERE memory_fts MATCH ?)"
		args = append(args, q)
	}
	_ = ctx
	return where, args, nil
}

func listOrder(order string, cur listCursor) (orderSQL, cursorSQL string, args []any) {
	switch order {
	case memory.OrderDue:
		orderSQL = `ORDER BY (m.due_at IS NULL), m.due_at ASC, m.id ASC`
		if cur.ID != "" {
			if cur.Key == "" {
				cursorSQL = `(m.due_at IS NULL AND m.id > ?)`
				args = []any{cur.ID}
			} else {
				cursorSQL = `(m.due_at IS NULL OR m.due_at > ? OR (m.due_at = ? AND m.id > ?))`
				args = []any{cur.Key, cur.Key, cur.ID}
			}
		}
	case memory.OrderManual:
		orderSQL = `ORDER BY m.sort_order ASC, m.id ASC`
		if cur.ID != "" {
			cursorSQL = `(m.sort_order > ? OR (m.sort_order = ? AND m.id > ?))`
			args = []any{cur.Key, cur.Key, cur.ID}
		}
	case memory.OrderCreated:
		orderSQL = `ORDER BY m.created_at DESC, m.id DESC`
		if cur.ID != "" {
			cursorSQL = `(m.created_at < ? OR (m.created_at = ? AND m.id < ?))`
			args = []any{cur.Key, cur.Key, cur.ID}
		}
	default:
		orderSQL = `ORDER BY m.updated_at DESC, m.id DESC`
		if cur.ID != "" {
			cursorSQL = `(m.updated_at < ? OR (m.updated_at = ? AND m.id < ?))`
			args = []any{cur.Key, cur.Key, cur.ID}
		}
	}
	return orderSQL, cursorSQL, args
}

func cursorKey(order string, sum memory.Summary) string {
	switch order {
	case memory.OrderDue:
		if sum.DueAt == nil {
			return ""
		}
		return formatTime(*sum.DueAt)
	case memory.OrderManual:
		return fmt.Sprintf("%d", sum.SortOrder)
	case memory.OrderCreated:
		return formatTime(sum.CreatedAt)
	default:
		return formatTime(sum.UpdatedAt)
	}
}
