package sqlite

import (
	"context"
	"database/sql"
	"time"

	"github.com/polterware/polterware/sparky/backend/domains/fault"
	"github.com/polterware/polterware/sparky/backend/domains/mind"
)

const mindCols = `id, parent_id, name, color, icon, sort_order, created_at, updated_at, rev`

func (r Minds) List(ctx context.Context) ([]mind.Mind, error) {
	rows, err := r.q(ctx, false).QueryContext(ctx, `SELECT `+mindCols+` FROM minds ORDER BY parent_id, sort_order, id`)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	out := []mind.Mind{}
	for rows.Next() {
		m, err := scanMind(rows)
		if err != nil {
			return nil, err
		}
		out = append(out, m)
	}
	return out, rows.Err()
}

func (r Minds) Get(ctx context.Context, id string) (mind.Mind, error) {
	m, err := scanMind(r.q(ctx, false).QueryRowContext(ctx, `SELECT `+mindCols+` FROM minds WHERE id=?`, id))
	if err != nil {
		return mind.Mind{}, mapSQL(err, "mind", id)
	}
	return m, nil
}

func (r Minds) Insert(ctx context.Context, m mind.Mind) error {
	return r.Within(ctx, func(ctx context.Context) error {
		_, err := r.q(ctx, true).ExecContext(ctx, `
			INSERT INTO minds(id, parent_id, name, color, icon, sort_order, created_at, updated_at, rev)
			VALUES (?,?,?,?,?,?,?,?,?)`,
			m.ID, nullString(m.ParentID), m.Name, nullString(m.Color), nullString(m.Icon),
			m.SortOrder, formatTime(m.CreatedAt), formatTime(m.UpdatedAt), m.Rev)
		return mapSQL(err, "mind", m.ID)
	})
}

func (r Minds) Update(ctx context.Context, expectedRev int64, m mind.Mind) error {
	return r.Within(ctx, func(ctx context.Context) error {
		current, err := r.Get(ctx, m.ID)
		if err != nil {
			return err
		}
		if expectedRev != 0 && current.Rev != expectedRev {
			return updateConflict("mind", m.ID, current.Rev, current)
		}
		res, err := r.q(ctx, true).ExecContext(ctx, `
			UPDATE minds SET parent_id=?, name=?, color=?, icon=?, sort_order=?, updated_at=?, rev=?
			WHERE id=?`,
			nullString(m.ParentID), m.Name, nullString(m.Color), nullString(m.Icon),
			m.SortOrder, formatTime(m.UpdatedAt), m.Rev, m.ID)
		if err != nil {
			return err
		}
		n, _ := res.RowsAffected()
		if n == 0 {
			return fault.NotFound("mind", m.ID)
		}
		return r.reindexMindMemories(ctx, m.ID)
	})
}

func (r Minds) Delete(ctx context.Context, expectedRev int64, id string) error {
	return r.Within(ctx, func(ctx context.Context) error {
		current, err := r.Get(ctx, id)
		if err != nil {
			return err
		}
		if expectedRev != 0 && current.Rev != expectedRev {
			return updateConflict("mind", id, current.Rev, current)
		}
		ids, err := r.mindMemoryIDs(ctx, id)
		if err != nil {
			return err
		}
		at := formatTime(time.Now().UTC())
		for _, memoryID := range ids {
			// Move explicitly so the FK cannot silently preserve an old revision.
			if _, err := r.q(ctx, true).ExecContext(ctx, `UPDATE memories SET mind_id=NULL, rev=rev+1, updated_at=? WHERE id=?`, at, memoryID); err != nil {
				return err
			}
		}
		if _, err := r.q(ctx, true).ExecContext(ctx, `DELETE FROM minds WHERE id=?`, id); err != nil {
			return err
		}
		for _, memoryID := range ids {
			if err := r.syncIndex(ctx, memoryID, false); err != nil {
				return err
			}
		}
		return nil
	})
}

func scanMind(row interface{ Scan(...any) error }) (mind.Mind, error) {
	var m mind.Mind
	var parent, color, icon sql.NullString
	var created, updated string
	if err := row.Scan(&m.ID, &parent, &m.Name, &color, &icon, &m.SortOrder, &created, &updated, &m.Rev); err != nil {
		return mind.Mind{}, err
	}
	m.ParentID = scanString(parent)
	m.Color = scanString(color)
	m.Icon = scanString(icon)
	m.CreatedAt = mustTime(created)
	m.UpdatedAt = mustTime(updated)
	return m, nil
}
