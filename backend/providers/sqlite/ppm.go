package sqlite

import (
	"database/sql"
	"encoding/json"
	"strings"
	"time"

	"github.com/polterware/polterware/backend/domains/knowledge"
	"github.com/polterware/polterware/backend/domains/ppm"
)

func (s *Store) UpsertPpmEntity(entity ppm.Entity) error {
	_, err := s.DB.Exec(`INSERT INTO ppm_entities(
		id,project_id,kind,ref_kind,ref_id,title,truth_class,source,actor,observed_at,created_at,updated_at
	) VALUES(?,?,?,?,?,?,?,?,?,?,?,?)
	ON CONFLICT(project_id, ref_kind, ref_id) DO UPDATE SET
		title=excluded.title,
		truth_class=excluded.truth_class,
		source=excluded.source,
		actor=excluded.actor,
		observed_at=excluded.observed_at,
		updated_at=excluded.updated_at`,
		entity.ID, entity.ProjectID, entity.Kind, entity.RefKind, entity.RefID, entity.Title,
		entity.TruthClass, entity.Source, entity.Actor,
		entity.ObservedAt.Format(time.RFC3339), entity.CreatedAt.Format(time.RFC3339), entity.UpdatedAt.Format(time.RFC3339),
	)
	return err
}

func (s *Store) ListPpmEntities(projectID, kind string) ([]ppm.Entity, error) {
	query := `SELECT id,project_id,kind,ref_kind,ref_id,title,truth_class,source,actor,observed_at,created_at,updated_at
		FROM ppm_entities WHERE project_id=?`
	args := []any{projectID}
	if kind != "" {
		query += ` AND kind=?`
		args = append(args, kind)
	}
	query += ` ORDER BY title`
	rows, err := s.DB.Query(query, args...)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	var result []ppm.Entity
	for rows.Next() {
		entity, err := scanPpmEntity(rows)
		if err != nil {
			return nil, err
		}
		result = append(result, entity)
	}
	return result, rows.Err()
}

func (s *Store) GetPpmEntity(id string) (ppm.Entity, error) {
	row := s.DB.QueryRow(`SELECT id,project_id,kind,ref_kind,ref_id,title,truth_class,source,actor,observed_at,created_at,updated_at
		FROM ppm_entities WHERE id=?`, id)
	return scanPpmEntity(row)
}

func (s *Store) GetPpmEntityByRef(projectID, refKind, refID string) (ppm.Entity, error) {
	row := s.DB.QueryRow(`SELECT id,project_id,kind,ref_kind,ref_id,title,truth_class,source,actor,observed_at,created_at,updated_at
		FROM ppm_entities WHERE project_id=? AND ref_kind=? AND ref_id=?`, projectID, refKind, refID)
	return scanPpmEntity(row)
}

func (s *Store) DeletePpmEntitiesMissingRefs(projectID, refKind string, keepIDs []string) error {
	if len(keepIDs) == 0 {
		_, err := s.DB.Exec(`DELETE FROM ppm_entities WHERE project_id=? AND ref_kind=?`, projectID, refKind)
		return err
	}
	placeholders := make([]string, len(keepIDs))
	args := make([]any, 0, len(keepIDs)+2)
	args = append(args, projectID, refKind)
	for i, id := range keepIDs {
		placeholders[i] = "?"
		args = append(args, id)
	}
	query := `DELETE FROM ppm_entities WHERE project_id=? AND ref_kind=? AND ref_id NOT IN (` + strings.Join(placeholders, ",") + `)`
	_, err := s.DB.Exec(query, args...)
	return err
}

func (s *Store) UpsertPpmRelationship(rel ppm.Relationship) error {
	_, err := s.DB.Exec(`INSERT INTO ppm_relationships(
		id,project_id,subject_entity_id,predicate,object_entity_id,truth_class,source,actor,observed_at,created_at
	) VALUES(?,?,?,?,?,?,?,?,?,?)
	ON CONFLICT(project_id, subject_entity_id, predicate, object_entity_id) DO UPDATE SET
		truth_class=excluded.truth_class,
		source=excluded.source,
		actor=excluded.actor,
		observed_at=excluded.observed_at`,
		rel.ID, rel.ProjectID, rel.SubjectEntityID, rel.Predicate, rel.ObjectEntityID,
		rel.TruthClass, rel.Source, rel.Actor,
		rel.ObservedAt.Format(time.RFC3339), rel.CreatedAt.Format(time.RFC3339),
	)
	return err
}

func (s *Store) ListPpmRelationships(projectID, entityID string) ([]ppm.Relationship, error) {
	query := `SELECT id,project_id,subject_entity_id,predicate,object_entity_id,truth_class,source,actor,observed_at,created_at
		FROM ppm_relationships WHERE project_id=?`
	args := []any{projectID}
	if entityID != "" {
		query += ` AND (subject_entity_id=? OR object_entity_id=?)`
		args = append(args, entityID, entityID)
	}
	query += ` ORDER BY created_at`
	rows, err := s.DB.Query(query, args...)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	var result []ppm.Relationship
	for rows.Next() {
		rel, err := scanPpmRelationship(rows)
		if err != nil {
			return nil, err
		}
		result = append(result, rel)
	}
	return result, rows.Err()
}

func (s *Store) DeletePpmRelationshipsMissing(projectID, source string, keepIDs []string) error {
	if len(keepIDs) == 0 {
		_, err := s.DB.Exec(`DELETE FROM ppm_relationships WHERE project_id=? AND source=?`, projectID, source)
		return err
	}
	placeholders := make([]string, len(keepIDs))
	args := make([]any, 0, len(keepIDs)+2)
	args = append(args, projectID, source)
	for i, id := range keepIDs {
		placeholders[i] = "?"
		args = append(args, id)
	}
	query := `DELETE FROM ppm_relationships WHERE project_id=? AND source=? AND id NOT IN (` + strings.Join(placeholders, ",") + `)`
	_, err := s.DB.Exec(query, args...)
	return err
}

// SyncProjectModel projects architecture nodes, resources, environments, and canvas edges into PPM.
func (s *Store) SyncProjectModel(projectID string) (int, error) {
	now := time.Now().UTC()
	count := 0

	layout, err := s.GetLayout(projectID)
	if err != nil && err != sql.ErrNoRows {
		return 0, err
	}
	var keepNodes []string
	if err == nil {
		for _, node := range ppm.ParseLayoutNodes(layout.BodyJSON) {
			entity, err := ppm.NewEntity(projectID, ppm.KindComponent, ppm.RefArchitectureNode, node.ID, node.Title, ppm.TruthDeclared, ppm.SourceArchitectureLayout, "system")
			if err != nil {
				return 0, err
			}
			entity.ObservedAt = now
			entity.UpdatedAt = now
			if err := s.UpsertPpmEntity(entity); err != nil {
				return 0, err
			}
			keepNodes = append(keepNodes, node.ID)
			count++
		}
	}
	if err := s.DeletePpmEntitiesMissingRefs(projectID, ppm.RefArchitectureNode, keepNodes); err != nil {
		return 0, err
	}

	keepRelIDs := []string{}
	if err == nil {
		nodeSet := make(map[string]struct{}, len(keepNodes))
		for _, id := range keepNodes {
			nodeSet[id] = struct{}{}
		}
		for _, edge := range ppm.ParseLayoutEdges(layout.BodyJSON) {
			if _, ok := nodeSet[edge.SourceNodeID]; !ok {
				continue
			}
			if _, ok := nodeSet[edge.TargetNodeID]; !ok {
				continue
			}
			subjectID := ppm.EntityID(projectID, ppm.RefArchitectureNode, edge.SourceNodeID)
			objectID := ppm.EntityID(projectID, ppm.RefArchitectureNode, edge.TargetNodeID)
			truth := ppm.TruthDeclared
			if !edge.Evidence {
				truth = ppm.TruthInferred
			}
			rel, err := ppm.NewRelationship(projectID, subjectID, edge.Predicate, objectID, truth, ppm.SourceArchitectureLayout, "system")
			if err != nil {
				return 0, err
			}
			rel.ObservedAt = now
			if err := s.UpsertPpmRelationship(rel); err != nil {
				return 0, err
			}
			keepRelIDs = append(keepRelIDs, rel.ID)
			count++
		}
	}
	if err := s.DeletePpmRelationshipsMissing(projectID, ppm.SourceArchitectureLayout, keepRelIDs); err != nil {
		return 0, err
	}

	resources, err := s.ListResources(projectID)
	if err != nil {
		return 0, err
	}
	keepResources := make([]string, 0, len(resources))
	for _, resource := range resources {
		entity, err := ppm.NewEntity(projectID, ppm.KindResource, ppm.RefResource, resource.ID, resource.Name, ppm.TruthObserved, "resources", "system")
		if err != nil {
			return 0, err
		}
		entity.ObservedAt = now
		entity.UpdatedAt = now
		if err := s.UpsertPpmEntity(entity); err != nil {
			return 0, err
		}
		keepResources = append(keepResources, resource.ID)
		count++
	}
	if err := s.DeletePpmEntitiesMissingRefs(projectID, ppm.RefResource, keepResources); err != nil {
		return 0, err
	}

	envs, err := s.ListEnvironments(projectID)
	if err != nil {
		return 0, err
	}
	keepEnvs := make([]string, 0, len(envs))
	for _, env := range envs {
		entity, err := ppm.NewEntity(projectID, ppm.KindEnvironment, ppm.RefEnvironment, env.ID, env.Name, ppm.TruthDeclared, "environments", "system")
		if err != nil {
			return 0, err
		}
		entity.ObservedAt = now
		entity.UpdatedAt = now
		if err := s.UpsertPpmEntity(entity); err != nil {
			return 0, err
		}
		keepEnvs = append(keepEnvs, env.ID)
		count++
	}
	if err := s.DeletePpmEntitiesMissingRefs(projectID, ppm.RefEnvironment, keepEnvs); err != nil {
		return 0, err
	}
	return count, nil
}

func scanPpmEntity(scanner interface {
	Scan(dest ...any) error
}) (ppm.Entity, error) {
	var entity ppm.Entity
	var observed, created, updated string
	if err := scanner.Scan(
		&entity.ID, &entity.ProjectID, &entity.Kind, &entity.RefKind, &entity.RefID, &entity.Title,
		&entity.TruthClass, &entity.Source, &entity.Actor, &observed, &created, &updated,
	); err != nil {
		return ppm.Entity{}, err
	}
	entity.ObservedAt, _ = time.Parse(time.RFC3339, observed)
	entity.CreatedAt, _ = time.Parse(time.RFC3339, created)
	entity.UpdatedAt, _ = time.Parse(time.RFC3339, updated)
	return entity, nil
}

func scanPpmRelationship(scanner interface {
	Scan(dest ...any) error
}) (ppm.Relationship, error) {
	var rel ppm.Relationship
	var observed, created string
	if err := scanner.Scan(
		&rel.ID, &rel.ProjectID, &rel.SubjectEntityID, &rel.Predicate, &rel.ObjectEntityID,
		&rel.TruthClass, &rel.Source, &rel.Actor, &observed, &created,
	); err != nil {
		return ppm.Relationship{}, err
	}
	rel.ObservedAt, _ = time.Parse(time.RFC3339, observed)
	rel.CreatedAt, _ = time.Parse(time.RFC3339, created)
	return rel, nil
}

func (s *Store) SaveKnowledgeLink(link knowledge.EntityLink) error {
	_, err := s.DB.Exec(`INSERT INTO knowledge_entity_links(
		id,project_id,knowledge_id,entity_id,predicate,truth_class,source,actor,observed_at,created_at
	) VALUES(?,?,?,?,?,?,?,?,?,?)
	ON CONFLICT(knowledge_id, entity_id, predicate) DO UPDATE SET
		truth_class=excluded.truth_class,
		source=excluded.source,
		actor=excluded.actor,
		observed_at=excluded.observed_at`,
		link.ID, link.ProjectID, link.KnowledgeID, link.EntityID, link.Predicate,
		link.TruthClass, link.Source, link.Actor,
		link.ObservedAt.Format(time.RFC3339), link.CreatedAt.Format(time.RFC3339),
	)
	return err
}

func (s *Store) ListKnowledgeLinks(projectID, knowledgeID, entityID string) ([]knowledge.EntityLink, error) {
	query := `SELECT id,project_id,knowledge_id,entity_id,predicate,truth_class,source,actor,observed_at,created_at
		FROM knowledge_entity_links WHERE project_id=?`
	args := []any{projectID}
	if knowledgeID != "" {
		query += ` AND knowledge_id=?`
		args = append(args, knowledgeID)
	}
	if entityID != "" {
		query += ` AND entity_id=?`
		args = append(args, entityID)
	}
	query += ` ORDER BY created_at`
	rows, err := s.DB.Query(query, args...)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	var result []knowledge.EntityLink
	for rows.Next() {
		var link knowledge.EntityLink
		var observed, created string
		if err := rows.Scan(
			&link.ID, &link.ProjectID, &link.KnowledgeID, &link.EntityID, &link.Predicate,
			&link.TruthClass, &link.Source, &link.Actor, &observed, &created,
		); err != nil {
			return nil, err
		}
		link.ObservedAt, _ = time.Parse(time.RFC3339, observed)
		link.CreatedAt, _ = time.Parse(time.RFC3339, created)
		result = append(result, link)
	}
	return result, rows.Err()
}

func (s *Store) DeleteKnowledgeLink(id string) error {
	_, err := s.DB.Exec(`DELETE FROM knowledge_entity_links WHERE id=?`, id)
	return err
}

func (s *Store) indexKnowledgeFTS(item knowledge.Item) error {
	tags, _ := json.Marshal(item.Tags)
	tagText := strings.Trim(string(tags), "[]")
	tagText = strings.ReplaceAll(tagText, `"`, "")
	if _, err := s.DB.Exec(`DELETE FROM knowledge_fts WHERE knowledge_id=?`, item.ID); err != nil {
		return err
	}
	_, err := s.DB.Exec(
		`INSERT INTO knowledge_fts(knowledge_id, project_id, kind, title, body, summary, tags) VALUES(?,?,?,?,?,?,?)`,
		item.ID, item.ProjectID, item.Kind, item.Title, item.Body, item.Summary, tagText,
	)
	return err
}

func (s *Store) SearchKnowledge(projectID, query, kind string, limit int) ([]knowledge.SearchHit, error) {
	if limit <= 0 || limit > 50 {
		limit = 20
	}
	query = strings.TrimSpace(query)
	if query == "" {
		return nil, nil
	}

	// Exact id match first.
	var hits []knowledge.SearchHit
	if item, err := s.getKnowledgeByID(query); err == nil && item.ProjectID == projectID {
		if kind == "" || item.Kind == kind {
			hits = append(hits, knowledge.SearchHit{
				Item: item, Score: 100, Snippet: item.Summary, TruthClass: "declared", Source: "knowledge",
			})
		}
	}

	ftsQuery := sanitizeFTSQuery(query)
	if ftsQuery == "" {
		return hits, nil
	}
	sqlQuery := `SELECT knowledge_id, bm25(knowledge_fts) AS rank
		FROM knowledge_fts
		WHERE knowledge_fts MATCH ? AND project_id=?`
	args := []any{ftsQuery, projectID}
	if kind != "" {
		sqlQuery += ` AND kind=?`
		args = append(args, kind)
	}
	sqlQuery += ` ORDER BY rank LIMIT ?`
	args = append(args, limit)
	rows, err := s.DB.Query(sqlQuery, args...)
	if err != nil {
		return hits, err
	}
	defer rows.Close()
	seen := map[string]struct{}{}
	for _, hit := range hits {
		seen[hit.Item.ID] = struct{}{}
	}
	for rows.Next() {
		var id string
		var rank float64
		if err := rows.Scan(&id, &rank); err != nil {
			return nil, err
		}
		if _, ok := seen[id]; ok {
			continue
		}
		item, err := s.getKnowledgeByID(id)
		if err != nil {
			continue
		}
		snippet := item.Summary
		if snippet == "" {
			snippet = item.Title
		}
		hits = append(hits, knowledge.SearchHit{
			Item: item, Score: -rank, Snippet: snippet, TruthClass: "declared", Source: "knowledge_fts",
		})
		seen[id] = struct{}{}
	}
	return hits, rows.Err()
}

func (s *Store) getKnowledgeByID(id string) (knowledge.Item, error) {
	var item knowledge.Item
	var tags, created, updated string
	err := s.DB.QueryRow(
		`SELECT id,project_id,title,body,kind,tags_json,summary,owner,status,extra_json,created_at,updated_at
		 FROM knowledge_items WHERE id=?`, id,
	).Scan(&item.ID, &item.ProjectID, &item.Title, &item.Body, &item.Kind, &tags, &item.Summary, &item.Owner, &item.Status, &item.ExtraJSON, &created, &updated)
	if err != nil {
		return knowledge.Item{}, err
	}
	_ = json.Unmarshal([]byte(tags), &item.Tags)
	item.CreatedAt, _ = time.Parse(time.RFC3339, created)
	item.UpdatedAt, _ = time.Parse(time.RFC3339, updated)
	return item, nil
}

func sanitizeFTSQuery(query string) string {
	parts := strings.Fields(query)
	cleaned := make([]string, 0, len(parts))
	for _, part := range parts {
		part = strings.Map(func(r rune) rune {
			if (r >= 'a' && r <= 'z') || (r >= 'A' && r <= 'Z') || (r >= '0' && r <= '9') || r == '_' || r == '-' {
				return r
			}
			return -1
		}, part)
		if part == "" {
			continue
		}
		cleaned = append(cleaned, part+"*")
	}
	return strings.Join(cleaned, " ")
}
