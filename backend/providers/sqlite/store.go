package sqlite

import (
	"database/sql"
	"encoding/json"
	"errors"
	"time"

	"github.com/polterware/polterware/backend/domains/architecture"
	"github.com/polterware/polterware/backend/domains/integrations"
	"github.com/polterware/polterware/backend/domains/knowledge"
	"github.com/polterware/polterware/backend/domains/operations"
	"github.com/polterware/polterware/backend/domains/projects"
	"github.com/polterware/polterware/backend/domains/resources"
	"github.com/polterware/polterware/backend/domains/workflows"
	dbsqlite "github.com/polterware/polterware/backend/platform/db/sqlite"
)

type Store struct{ DB *sql.DB }

type ProcessSession struct {
	ID, ProjectID, EnvironmentID, Command, Status string
}

type LogChunk struct {
	OperationID string
	Sequence    int64
	Body        string
	Truncated   bool
	CreatedAt   time.Time
}

func New(path string) (*Store, error) {
	db, err := dbsqlite.Open(path)
	if err != nil {
		return nil, err
	}
	if err := dbsqlite.Migrate(db); err != nil {
		_ = db.Close()
		return nil, err
	}
	return &Store{DB: db}, nil
}

func (s *Store) Close() error { return s.DB.Close() }

func (s *Store) SeedDefaults() error {
	var count int
	if err := s.DB.QueryRow(`SELECT COUNT(*) FROM projects`).Scan(&count); err != nil {
		return err
	}
	if count != 0 {
		return nil
	}
	project, _ := projects.NewProject("project-local", "Local project", ".")
	if err := s.SaveProject(project); err != nil {
		return err
	}
	environment, _ := projects.NewEnvironment("environment-local", project.ID, "local", "local")
	if err := s.SaveEnvironment(environment); err != nil {
		return err
	}
	_, err := s.DB.Exec(`INSERT INTO architecture_layouts(id, project_id, revision, body_json) VALUES(?,?,?,?)`, "layout-local", project.ID, 1, `{"nodes":[],"edges":[]}`)
	return err
}

func (s *Store) SaveProject(project projects.Project) error {
	_, err := s.DB.Exec(`INSERT INTO projects(id,name,root_path,created_at,updated_at) VALUES(?,?,?,?,?)
		ON CONFLICT(id) DO UPDATE SET name=excluded.name,root_path=excluded.root_path,updated_at=excluded.updated_at`,
		project.ID, project.Name, project.RootPath, project.CreatedAt.Format(time.RFC3339), project.UpdatedAt.Format(time.RFC3339))
	return err
}

func (s *Store) ListProjects() ([]projects.Project, error) {
	rows, err := s.DB.Query(`SELECT id,name,root_path,created_at,updated_at FROM projects ORDER BY name`)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	var result []projects.Project
	for rows.Next() {
		var item projects.Project
		var created, updated string
		if err := rows.Scan(&item.ID, &item.Name, &item.RootPath, &created, &updated); err != nil {
			return nil, err
		}
		item.CreatedAt, _ = time.Parse(time.RFC3339, created)
		item.UpdatedAt, _ = time.Parse(time.RFC3339, updated)
		result = append(result, item)
	}
	return result, rows.Err()
}

func (s *Store) SaveEnvironment(environment projects.Environment) error {
	_, err := s.DB.Exec(`INSERT INTO environments(id,project_id,name,kind) VALUES(?,?,?,?)
		ON CONFLICT(id) DO UPDATE SET project_id=excluded.project_id,name=excluded.name,kind=excluded.kind`,
		environment.ID, environment.ProjectID, environment.Name, environment.Kind)
	return err
}

func (s *Store) ListEnvironments(projectID string) ([]projects.Environment, error) {
	rows, err := s.DB.Query(`SELECT id,project_id,name,kind FROM environments WHERE project_id=? ORDER BY name`, projectID)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	var result []projects.Environment
	for rows.Next() {
		var item projects.Environment
		if err := rows.Scan(&item.ID, &item.ProjectID, &item.Name, &item.Kind); err != nil {
			return nil, err
		}
		result = append(result, item)
	}
	return result, rows.Err()
}

func (s *Store) GetLayout(projectID string) (architecture.Layout, error) {
	var layout architecture.Layout
	err := s.DB.QueryRow(`SELECT id,project_id,revision,body_json FROM architecture_layouts WHERE project_id=?`, projectID).
		Scan(&layout.ID, &layout.ProjectID, &layout.Revision, &layout.BodyJSON)
	return layout, err
}

func (s *Store) UpdateLayout(projectID string, expectedRevision int64, body string) (architecture.Layout, error) {
	current, err := s.GetLayout(projectID)
	if errors.Is(err, sql.ErrNoRows) {
		current = architecture.Layout{ID: "layout-" + projectID, ProjectID: projectID, Revision: 0, BodyJSON: `{"nodes":[],"edges":[]}`}
	} else if err != nil {
		return current, err
	}
	updated, err := architecture.UpdateLayout(current, expectedRevision, body)
	if err != nil {
		return current, err
	}
	_, err = s.DB.Exec(`INSERT INTO architecture_layouts(id,project_id,revision,body_json) VALUES(?,?,?,?)
		ON CONFLICT(project_id) DO UPDATE SET revision=excluded.revision,body_json=excluded.body_json`, updated.ID, updated.ProjectID, updated.Revision, updated.BodyJSON)
	return updated, err
}

func (s *Store) SaveResource(resource resources.Resource) error {
	existing, err := s.ListResources(resource.ProjectID)
	if err != nil {
		return err
	}
	if err := resources.EnsureUniqueExternalKey(existing, resource); err != nil {
		return err
	}
	_, err = s.DB.Exec(`INSERT INTO resources(id,project_id,environment_id,name,type,external_key,source) VALUES(?,?,?,?,?,?,?)
		ON CONFLICT(id) DO UPDATE SET project_id=excluded.project_id,environment_id=excluded.environment_id,name=excluded.name,type=excluded.type,external_key=excluded.external_key,source=excluded.source`,
		resource.ID, resource.ProjectID, nullable(resource.EnvironmentID), resource.Name, resource.Type, nullable(resource.ExternalKey), resource.Source)
	return err
}

func (s *Store) ListResources(projectID string) ([]resources.Resource, error) {
	rows, err := s.DB.Query(`SELECT id,project_id,COALESCE(environment_id,''),name,type,COALESCE(external_key,''),source FROM resources WHERE project_id=? ORDER BY name`, projectID)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	var result []resources.Resource
	for rows.Next() {
		var item resources.Resource
		if err := rows.Scan(&item.ID, &item.ProjectID, &item.EnvironmentID, &item.Name, &item.Type, &item.ExternalKey, &item.Source); err != nil {
			return nil, err
		}
		result = append(result, item)
	}
	return result, rows.Err()
}

func (s *Store) SaveKnowledge(item knowledge.Item) error {
	tags, _ := json.Marshal(item.Tags)
	_, err := s.DB.Exec(`INSERT INTO knowledge_items(id,project_id,title,body,kind,tags_json,created_at,updated_at) VALUES(?,?,?,?,?,?,?,?)
		ON CONFLICT(id) DO UPDATE SET project_id=excluded.project_id,title=excluded.title,body=excluded.body,kind=excluded.kind,tags_json=excluded.tags_json,updated_at=excluded.updated_at`,
		item.ID, item.ProjectID, item.Title, item.Body, item.Kind, string(tags), item.CreatedAt.Format(time.RFC3339), item.UpdatedAt.Format(time.RFC3339))
	return err
}

func (s *Store) ListKnowledge(projectID, kind string) ([]knowledge.Item, error) {
	query := `SELECT id,project_id,title,body,kind,tags_json,created_at,updated_at FROM knowledge_items WHERE project_id=?`
	args := []any{projectID}
	if kind != "" {
		query += ` AND kind=?`
		args = append(args, kind)
	}
	query += ` ORDER BY updated_at DESC`
	rows, err := s.DB.Query(query, args...)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	var result []knowledge.Item
	for rows.Next() {
		var item knowledge.Item
		var tags, created, updated string
		if err := rows.Scan(&item.ID, &item.ProjectID, &item.Title, &item.Body, &item.Kind, &tags, &created, &updated); err != nil {
			return nil, err
		}
		_ = json.Unmarshal([]byte(tags), &item.Tags)
		item.CreatedAt, _ = time.Parse(time.RFC3339, created)
		item.UpdatedAt, _ = time.Parse(time.RFC3339, updated)
		result = append(result, item)
	}
	return result, rows.Err()
}

func (s *Store) DeleteKnowledge(id string) error {
	_, err := s.DB.Exec(`DELETE FROM knowledge_items WHERE id=?`, id)
	return err
}

func (s *Store) SaveOperation(operation *operations.Operation) error {
	params, _ := json.Marshal(operation.Params)
	_, err := s.DB.Exec(`INSERT INTO operations(id,correlation_id,project_id,environment_id,type,status,result_summary,error_message,params_json,idempotency_key,requested_at,started_at,finished_at) VALUES(?,?,?,?,?,?,?,?,?,?,?,?,?)
		ON CONFLICT(id) DO UPDATE SET status=excluded.status,result_summary=excluded.result_summary,error_message=excluded.error_message,started_at=excluded.started_at,finished_at=excluded.finished_at`,
		operation.ID, operation.CorrelationID, operation.ProjectID, nullable(operation.EnvironmentID), operation.Type, string(operation.Status), operation.ResultSummary, operation.ErrorMessage, string(params), nullable(operation.IdempotencyKey), operation.RequestedAt.Format(time.RFC3339), timeValue(operation.StartedAt), timeValue(operation.FinishedAt))
	return err
}

func (s *Store) LoadOperation(id string) (*operations.Operation, error) {
	return scanOperation(s.DB.QueryRow(`SELECT id,correlation_id,project_id,COALESCE(environment_id,''),type,status,result_summary,error_message,params_json,COALESCE(idempotency_key,''),requested_at,started_at,finished_at FROM operations WHERE id=?`, id))
}

func (s *Store) ListOperations(projectID string, limit int) ([]*operations.Operation, error) {
	if limit <= 0 || limit > 200 {
		limit = 50
	}
	rows, err := s.DB.Query(`SELECT id,correlation_id,project_id,COALESCE(environment_id,''),type,status,result_summary,error_message,params_json,COALESCE(idempotency_key,''),requested_at,started_at,finished_at FROM operations WHERE project_id=? ORDER BY requested_at DESC LIMIT ?`, projectID, limit)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	var result []*operations.Operation
	for rows.Next() {
		item, err := scanOperation(rows)
		if err != nil {
			return nil, err
		}
		result = append(result, item)
	}
	return result, rows.Err()
}

type scanner interface{ Scan(dest ...any) error }

func scanOperation(row scanner) (*operations.Operation, error) {
	item := &operations.Operation{}
	var status, params, requested string
	var started, finished sql.NullString
	if err := row.Scan(&item.ID, &item.CorrelationID, &item.ProjectID, &item.EnvironmentID, &item.Type, &status, &item.ResultSummary, &item.ErrorMessage, &params, &item.IdempotencyKey, &requested, &started, &finished); err != nil {
		return nil, err
	}
	item.Status = operations.Status(status)
	_ = json.Unmarshal([]byte(params), &item.Params)
	item.RequestedAt, _ = time.Parse(time.RFC3339, requested)
	item.StartedAt = parseNullableTime(started)
	item.FinishedAt = parseNullableTime(finished)
	return item, nil
}

func (s *Store) SaveWorkflow(workflow workflows.Workflow) error {
	if err := workflows.Validate(workflow); err != nil {
		return err
	}
	steps, _ := json.Marshal(workflow.Steps)
	_, err := s.DB.Exec(`INSERT INTO workflows(id,project_id,name,status,steps_json) VALUES(?,?,?,?,?) ON CONFLICT(id) DO UPDATE SET project_id=excluded.project_id,name=excluded.name,status=excluded.status,steps_json=excluded.steps_json`, workflow.ID, workflow.ProjectID, workflow.Name, workflow.Status, string(steps))
	return err
}

func (s *Store) GetWorkflow(id string) (workflows.Workflow, error) {
	var item workflows.Workflow
	var steps string
	err := s.DB.QueryRow(`SELECT id,project_id,name,status,steps_json FROM workflows WHERE id=?`, id).Scan(&item.ID, &item.ProjectID, &item.Name, &item.Status, &steps)
	_ = json.Unmarshal([]byte(steps), &item.Steps)
	return item, err
}

func (s *Store) ListWorkflows(projectID string) ([]workflows.Workflow, error) {
	rows, err := s.DB.Query(`SELECT id FROM workflows WHERE project_id=? ORDER BY name`, projectID)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	var result []workflows.Workflow
	for rows.Next() {
		var id string
		if err := rows.Scan(&id); err != nil {
			return nil, err
		}
		item, err := s.GetWorkflow(id)
		if err != nil {
			return nil, err
		}
		result = append(result, item)
	}
	return result, rows.Err()
}

func (s *Store) SaveConnection(connection integrations.Connection) error {
	_, err := s.DB.Exec(`INSERT INTO integration_connections(id,project_id,provider_id,status,label) VALUES(?,?,?,?,?) ON CONFLICT(id) DO UPDATE SET project_id=excluded.project_id,provider_id=excluded.provider_id,status=excluded.status,label=excluded.label`, connection.ID, connection.ProjectID, connection.ProviderID, connection.Status, connection.Label)
	return err
}

func (s *Store) ListConnections(projectID string) ([]integrations.Connection, error) {
	rows, err := s.DB.Query(`SELECT id,project_id,provider_id,status,label FROM integration_connections WHERE project_id=? ORDER BY label`, projectID)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	var result []integrations.Connection
	for rows.Next() {
		var item integrations.Connection
		if err := rows.Scan(&item.ID, &item.ProjectID, &item.ProviderID, &item.Status, &item.Label); err != nil {
			return nil, err
		}
		result = append(result, item)
	}
	return result, rows.Err()
}

func (s *Store) SaveProcess(session ProcessSession) error {
	_, err := s.DB.Exec(`INSERT INTO process_sessions(id,project_id,environment_id,command,status,created_at) VALUES(?,?,?,?,?,?) ON CONFLICT(id) DO UPDATE SET status=excluded.status`, session.ID, session.ProjectID, nullable(session.EnvironmentID), session.Command, session.Status, time.Now().UTC().Format(time.RFC3339))
	return err
}

func (s *Store) GetProcess(id string) (ProcessSession, error) {
	var item ProcessSession
	err := s.DB.QueryRow(`SELECT id,project_id,COALESCE(environment_id,''),command,status FROM process_sessions WHERE id=?`, id).Scan(&item.ID, &item.ProjectID, &item.EnvironmentID, &item.Command, &item.Status)
	return item, err
}

func (s *Store) ListProcesses(projectID string) ([]ProcessSession, error) {
	rows, err := s.DB.Query(`SELECT id,project_id,COALESCE(environment_id,''),command,status FROM process_sessions WHERE project_id=? ORDER BY created_at DESC`, projectID)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	var result []ProcessSession
	for rows.Next() {
		var item ProcessSession
		if err := rows.Scan(&item.ID, &item.ProjectID, &item.EnvironmentID, &item.Command, &item.Status); err != nil {
			return nil, err
		}
		result = append(result, item)
	}
	return result, rows.Err()
}

func (s *Store) AppendLog(operationID, body string, truncated bool) (LogChunk, error) {
	var sequence int64
	if err := s.DB.QueryRow(`SELECT COALESCE(MAX(sequence),0)+1 FROM log_chunks WHERE operation_id=?`, operationID).Scan(&sequence); err != nil {
		return LogChunk{}, err
	}
	now := time.Now().UTC()
	_, err := s.DB.Exec(`INSERT INTO log_chunks(operation_id,sequence,body,truncated,created_at) VALUES(?,?,?,?,?)`, operationID, sequence, body, truncated, now.Format(time.RFC3339))
	return LogChunk{OperationID: operationID, Sequence: sequence, Body: body, Truncated: truncated, CreatedAt: now}, err
}

func (s *Store) QueryLogs(operationID string, limit int) ([]LogChunk, error) {
	if limit <= 0 || limit > 1000 {
		limit = 200
	}
	rows, err := s.DB.Query(`SELECT operation_id,sequence,body,truncated,created_at FROM log_chunks WHERE operation_id=? ORDER BY sequence LIMIT ?`, operationID, limit)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	var result []LogChunk
	for rows.Next() {
		var item LogChunk
		var truncated int
		var created string
		if err := rows.Scan(&item.OperationID, &item.Sequence, &item.Body, &truncated, &created); err != nil {
			return nil, err
		}
		item.Truncated = truncated != 0
		item.CreatedAt, _ = time.Parse(time.RFC3339, created)
		result = append(result, item)
	}
	return result, rows.Err()
}

func (s *Store) StoreHealth() (bool, string, error) {
	var quarantined int
	var message string
	err := s.DB.QueryRow(`SELECT quarantined,message FROM store_health WHERE id=1`).Scan(&quarantined, &message)
	return quarantined != 0, message, err
}

func (s *Store) Reset() error {
	tables := []string{"log_chunks", "process_sessions", "integration_connections", "workflows", "operations", "knowledge_items", "architecture_layouts", "resources", "environments", "projects"}
	tx, err := s.DB.Begin()
	if err != nil {
		return err
	}
	for _, table := range tables {
		if _, err := tx.Exec(`DELETE FROM ` + table); err != nil {
			_ = tx.Rollback()
			return err
		}
	}
	if err := tx.Commit(); err != nil {
		return err
	}
	return s.SeedDefaults()
}

func nullable(value string) any {
	if value == "" {
		return nil
	}
	return value
}
func timeValue(value *time.Time) any {
	if value == nil {
		return nil
	}
	return value.Format(time.RFC3339)
}
func parseNullableTime(value sql.NullString) *time.Time {
	if !value.Valid {
		return nil
	}
	parsed, err := time.Parse(time.RFC3339, value.String)
	if err != nil {
		return nil
	}
	return &parsed
}
