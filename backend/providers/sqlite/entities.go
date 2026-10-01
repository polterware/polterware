package sqlite

import (
	"encoding/json"
	"time"

	"github.com/polterware/polterware/backend/domains/architecture"
	"github.com/polterware/polterware/backend/domains/integrations"
	"github.com/polterware/polterware/backend/domains/projects"
)

func (s *Store) SaveProjectSettings(settings projects.Settings) error {
	_, err := s.DB.Exec(`INSERT INTO project_settings(project_id,description,default_environment_id,repository) VALUES(?,?,?,?)
		ON CONFLICT(project_id) DO UPDATE SET description=excluded.description,default_environment_id=excluded.default_environment_id,repository=excluded.repository`,
		settings.ProjectID, settings.Description, nullable(settings.DefaultEnvironmentID), settings.Repository)
	return err
}

func (s *Store) GetProjectSettings(projectID string) (projects.Settings, error) {
	var item projects.Settings
	var defaultEnv string
	err := s.DB.QueryRow(`SELECT project_id,description,COALESCE(default_environment_id,''),repository FROM project_settings WHERE project_id=?`, projectID).
		Scan(&item.ProjectID, &item.Description, &defaultEnv, &item.Repository)
	item.DefaultEnvironmentID = defaultEnv
	return item, err
}

func (s *Store) SaveView(view architecture.View) error {
	if view.UpdatedAt == "" {
		view.UpdatedAt = time.Now().UTC().Format(time.RFC3339)
	}
	environments, _ := json.Marshal(view.Environments)
	tags, _ := json.Marshal(view.Tags)
	tx, err := s.DB.Begin()
	if err != nil {
		return err
	}
	if _, err := tx.Exec(`INSERT INTO architecture_views(id,project_id,name,description,owner,purpose,notes,filter_summary,environments_json,tags_json,updated_at) VALUES(?,?,?,?,?,?,?,?,?,?,?)
		ON CONFLICT(id) DO UPDATE SET project_id=excluded.project_id,name=excluded.name,description=excluded.description,owner=excluded.owner,purpose=excluded.purpose,notes=excluded.notes,filter_summary=excluded.filter_summary,environments_json=excluded.environments_json,tags_json=excluded.tags_json,updated_at=excluded.updated_at`,
		view.ID, view.ProjectID, view.Name, view.Description, view.Owner, view.Purpose, view.Notes, view.FilterSummary, string(environments), string(tags), view.UpdatedAt); err != nil {
		_ = tx.Rollback()
		return err
	}
	if _, err := tx.Exec(`DELETE FROM architecture_view_members WHERE view_id=?`, view.ID); err != nil {
		_ = tx.Rollback()
		return err
	}
	for _, member := range view.Members {
		if _, err := tx.Exec(`INSERT INTO architecture_view_members(view_id,node_id,name,type,layer) VALUES(?,?,?,?,?)`, view.ID, member.NodeID, member.Name, member.Type, member.Layer); err != nil {
			_ = tx.Rollback()
			return err
		}
	}
	return tx.Commit()
}

func (s *Store) ListViews(projectID string) ([]architecture.View, error) {
	rows, err := s.DB.Query(`SELECT id,project_id,name,description,owner,purpose,notes,filter_summary,environments_json,tags_json,updated_at FROM architecture_views WHERE project_id=? ORDER BY name`, projectID)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	var result []architecture.View
	for rows.Next() {
		var item architecture.View
		var environments, tags string
		if err := rows.Scan(&item.ID, &item.ProjectID, &item.Name, &item.Description, &item.Owner, &item.Purpose, &item.Notes, &item.FilterSummary, &environments, &tags, &item.UpdatedAt); err != nil {
			return nil, err
		}
		_ = json.Unmarshal([]byte(environments), &item.Environments)
		_ = json.Unmarshal([]byte(tags), &item.Tags)
		members, err := s.listViewMembers(item.ID)
		if err != nil {
			return nil, err
		}
		item.Members = members
		result = append(result, item)
	}
	return result, rows.Err()
}

func (s *Store) DeleteView(id string) error {
	_, err := s.DB.Exec(`DELETE FROM architecture_views WHERE id=?`, id)
	return err
}

func (s *Store) listViewMembers(viewID string) ([]architecture.ViewMember, error) {
	rows, err := s.DB.Query(`SELECT node_id,name,type,layer FROM architecture_view_members WHERE view_id=? ORDER BY name`, viewID)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	var result []architecture.ViewMember
	for rows.Next() {
		var item architecture.ViewMember
		if err := rows.Scan(&item.NodeID, &item.Name, &item.Type, &item.Layer); err != nil {
			return nil, err
		}
		result = append(result, item)
	}
	return result, rows.Err()
}

func (s *Store) SaveIntegrationResource(resource integrations.Resource) error {
	_, err := s.DB.Exec(`INSERT INTO integration_resources(id,connection_id,name,kind,external_id,environment_id,status) VALUES(?,?,?,?,?,?,?)
		ON CONFLICT(id) DO UPDATE SET connection_id=excluded.connection_id,name=excluded.name,kind=excluded.kind,external_id=excluded.external_id,environment_id=excluded.environment_id,status=excluded.status`,
		resource.ID, resource.ConnectionID, resource.Name, resource.Kind, resource.ExternalID, nullable(resource.EnvironmentID), resource.Status)
	return err
}

func (s *Store) DeleteIntegrationResourcesByConnection(connectionID string) error {
	_, err := s.DB.Exec(`DELETE FROM integration_resources WHERE connection_id=?`, connectionID)
	return err
}

func (s *Store) CountIntegrationResourcesByConnection(connectionID string) (int, error) {
	var count int
	err := s.DB.QueryRow(`SELECT COUNT(*) FROM integration_resources WHERE connection_id=?`, connectionID).Scan(&count)
	return count, err
}

func (s *Store) ListIntegrationResources(projectID string) ([]integrations.Resource, error) {
	rows, err := s.DB.Query(`SELECT r.id,r.connection_id,r.name,r.kind,r.external_id,COALESCE(r.environment_id,''),r.status
		FROM integration_resources r
		JOIN integration_connections c ON c.id = r.connection_id
		WHERE c.project_id=? ORDER BY r.name`, projectID)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	var result []integrations.Resource
	for rows.Next() {
		var item integrations.Resource
		if err := rows.Scan(&item.ID, &item.ConnectionID, &item.Name, &item.Kind, &item.ExternalID, &item.EnvironmentID, &item.Status); err != nil {
			return nil, err
		}
		result = append(result, item)
	}
	return result, rows.Err()
}

func (s *Store) SaveIntegrationSyncRun(run integrations.SyncRun) error {
	_, err := s.DB.Exec(`INSERT INTO integration_sync_runs(id,connection_id,started_at,finished_at,result,imported,changed,message) VALUES(?,?,?,?,?,?,?,?)
		ON CONFLICT(id) DO UPDATE SET connection_id=excluded.connection_id,started_at=excluded.started_at,finished_at=excluded.finished_at,result=excluded.result,imported=excluded.imported,changed=excluded.changed,message=excluded.message`,
		run.ID, run.ConnectionID, run.StartedAt, run.FinishedAt, run.Result, run.Imported, run.Changed, run.Message)
	return err
}

func (s *Store) ListIntegrationSyncRuns(projectID string) ([]integrations.SyncRun, error) {
	rows, err := s.DB.Query(`SELECT r.id,r.connection_id,r.started_at,r.finished_at,r.result,r.imported,r.changed,r.message
		FROM integration_sync_runs r
		JOIN integration_connections c ON c.id = r.connection_id
		WHERE c.project_id=? ORDER BY r.started_at DESC`, projectID)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	var result []integrations.SyncRun
	for rows.Next() {
		var item integrations.SyncRun
		if err := rows.Scan(&item.ID, &item.ConnectionID, &item.StartedAt, &item.FinishedAt, &item.Result, &item.Imported, &item.Changed, &item.Message); err != nil {
			return nil, err
		}
		result = append(result, item)
	}
	return result, rows.Err()
}
