package sqlite

import (
	"database/sql"
	"encoding/json"
	"fmt"
	"strings"

	"github.com/polterware/polterware/backend/domains/atlas"
	"github.com/polterware/polterware/backend/domains/work"
)

func (s *Store) loadAtlasNodes(projectID string) ([]atlas.Node, error) {
	rows, err := s.DB.Query(`SELECT id,project_id,COALESCE(parent_id,''),level,slug,path,title,intent,status,decisions_json,position,repo,folder,migration_from,migration_to
		FROM atlas_nodes WHERE project_id=? ORDER BY position, created_at`, projectID)
	if err != nil {
		return nil, err
	}
	var nodes []atlas.Node
	index := map[string]int{}
	for rows.Next() {
		var node atlas.Node
		var decisions string
		if err := rows.Scan(&node.ID, &node.ProjectID, &node.ParentID, &node.Level, &node.Slug, &node.Path,
			&node.Title, &node.Intent, &node.Status, &decisions, &node.Position, &node.Repo, &node.Folder, &node.MigrationFrom, &node.MigrationTo); err != nil {
			_ = rows.Close()
			return nil, err
		}
		_ = json.Unmarshal([]byte(decisions), &node.Decisions)
		index[node.ID] = len(nodes)
		nodes = append(nodes, node)
	}
	if err := rows.Close(); err != nil {
		return nil, err
	}
	if len(nodes) == 0 {
		return nil, nil
	}
	err = s.eachRow(`SELECT id,node_id,text,state,position FROM atlas_invariants
		WHERE node_id IN (SELECT id FROM atlas_nodes WHERE project_id=?) ORDER BY position`,
		projectID, func(rows *sql.Rows) error {
			var inv atlas.Invariant
			if err := rows.Scan(&inv.ID, &inv.NodeID, &inv.Text, &inv.State, &inv.Position); err != nil {
				return err
			}
			if i, ok := index[inv.NodeID]; ok {
				nodes[i].Invariants = append(nodes[i].Invariants, inv)
			}
			return nil
		})
	return nodes, err
}

func applyAtlasOp(tx *sql.Tx, projectID string, op work.Op, now string) error {
	if op.NodeID != "" {
		return applyAtlasInvariant(tx, op, now)
	}
	parent, _ := op.Fields["parent_id"].(string)
	switch op.Kind {
	case work.OpCreate:
		decisions, err := columnValue(op.Fields["decisions"])
		if err != nil {
			return err
		}
		repo, _ := op.Fields["repo"].(string)
		folder, _ := op.Fields["folder"].(string)
		migrationFrom, _ := op.Fields["migration_from"].(string)
		migrationTo, _ := op.Fields["migration_to"].(string)
		_, err = tx.Exec(`INSERT INTO atlas_nodes(id,project_id,parent_id,level,slug,path,title,intent,status,decisions_json,position,repo,folder,migration_from,migration_to,created_at,updated_at)
			VALUES(?,?,NULLIF(?,''),?,?,?,?,?,?,?,?,?,?,?,?,?,?)`,
			op.EntityID, projectID, parent, op.Fields["level"], op.Fields["slug"], op.Fields["path"],
			op.Fields["title"], op.Fields["intent"], op.Fields["status"], decisions, op.Fields["position"], repo, folder, migrationFrom, migrationTo, now, now)
		return err
	case work.OpUpdate, work.OpRemove:
		return updateAtlasNode(tx, op, now)
	default:
		return fmt.Errorf("unknown op %q", op.Kind)
	}
}

func updateAtlasNode(tx *sql.Tx, op work.Op, now string) error {
	var sets []string
	var values []any
	for field, value := range op.Fields {
		switch field {
		case "parent_id":
			sets = append(sets, "parent_id=NULLIF(?,'')")
			values = append(values, value)
		case "decisions":
			encoded, err := columnValue(value)
			if err != nil {
				return err
			}
			sets = append(sets, "decisions_json=?")
			values = append(values, encoded)
		case "slug", "path", "title", "intent", "level", "status", "position", "repo", "folder", "migration_from", "migration_to":
			sets = append(sets, field+"=?")
			values = append(values, value)
		default:
			return fmt.Errorf("unknown field %q", field)
		}
	}
	if len(sets) == 0 {
		return nil
	}
	sets = append(sets, "updated_at=?")
	values = append(values, now, op.EntityID)
	_, err := tx.Exec(`UPDATE atlas_nodes SET `+strings.Join(sets, ",")+` WHERE id=?`, values...)
	return err
}

func applyAtlasInvariant(tx *sql.Tx, op work.Op, now string) error {
	switch op.Kind {
	case work.OpCreate:
		_, err := tx.Exec(`INSERT INTO atlas_invariants(id,node_id,text,state,position,created_at,updated_at)
			VALUES(?,?,?,?,?,?,?)`,
			op.EntityID, op.NodeID, op.Fields["text"], op.Fields["state"], op.Fields["position"], now, now)
		return err
	case work.OpUpdate, work.OpRemove:
		var sets []string
		var values []any
		for _, field := range []string{"text", "state", "position"} {
			value, ok := op.Fields[field]
			if !ok {
				continue
			}
			sets = append(sets, field+"=?")
			values = append(values, value)
		}
		if len(sets) == 0 {
			return nil
		}
		sets = append(sets, "updated_at=?")
		values = append(values, now, op.EntityID)
		_, err := tx.Exec(`UPDATE atlas_invariants SET `+strings.Join(sets, ",")+` WHERE id=?`, values...)
		return err
	default:
		return fmt.Errorf("unknown op %q", op.Kind)
	}
}
