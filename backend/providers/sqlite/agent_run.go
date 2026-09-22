package sqlite

import (
	"database/sql"
	"errors"
	"time"

	"github.com/polterware/polterware/backend/domains/agentrun"
)

func (s *Store) CreateAgentRun(run agentrun.Run) error {
	if run.ID == "" {
		run.ID = agentrun.NewRunID()
	}
	if run.CreatedAt.IsZero() {
		run.CreatedAt = time.Now().UTC()
	}
	if run.Status == "" {
		run.Status = agentrun.StatusOpen
	}
	var completed any
	if run.CompletedAt != nil {
		completed = run.CompletedAt.UTC().Format(time.RFC3339)
	}
	_, err := s.DB.Exec(
		`INSERT INTO agent_runs(id,project_id,task,agent_label,status,created_at,completed_at)
		 VALUES(?,?,?,?,?,?,?)`,
		run.ID, run.ProjectID, run.Task, run.AgentLabel, run.Status,
		run.CreatedAt.UTC().Format(time.RFC3339), completed,
	)
	return err
}

func (s *Store) GetAgentRun(id string) (agentrun.Run, error) {
	var run agentrun.Run
	var created string
	var completed sql.NullString
	err := s.DB.QueryRow(
		`SELECT id,project_id,task,agent_label,status,created_at,completed_at
		 FROM agent_runs WHERE id=?`, id,
	).Scan(&run.ID, &run.ProjectID, &run.Task, &run.AgentLabel, &run.Status, &created, &completed)
	if errors.Is(err, sql.ErrNoRows) {
		return agentrun.Run{}, agentrun.ErrUnknownRun
	}
	if err != nil {
		return agentrun.Run{}, err
	}
	run.CreatedAt, _ = time.Parse(time.RFC3339, created)
	if completed.Valid && completed.String != "" {
		t, _ := time.Parse(time.RFC3339, completed.String)
		run.CompletedAt = &t
	}
	return run, nil
}

func (s *Store) UpdateAgentRun(run agentrun.Run) error {
	var completed any
	if run.CompletedAt != nil {
		completed = run.CompletedAt.UTC().Format(time.RFC3339)
	}
	res, err := s.DB.Exec(
		`UPDATE agent_runs SET task=?, agent_label=?, status=?, completed_at=? WHERE id=?`,
		run.Task, run.AgentLabel, run.Status, completed, run.ID,
	)
	if err != nil {
		return err
	}
	n, _ := res.RowsAffected()
	if n == 0 {
		return agentrun.ErrUnknownRun
	}
	return nil
}

func (s *Store) AppendAgentRunStep(step agentrun.Step) error {
	if step.ID == "" {
		step.ID = agentrun.NewStepID()
	}
	if step.CreatedAt.IsZero() {
		step.CreatedAt = time.Now().UTC()
	}
	if step.PayloadJSON == "" {
		step.PayloadJSON = "{}"
	}
	_, err := s.DB.Exec(
		`INSERT INTO agent_run_steps(id,agent_run_id,kind,summary,payload_json,created_at)
		 VALUES(?,?,?,?,?,?)`,
		step.ID, step.AgentRunID, step.Kind, step.Summary, step.PayloadJSON,
		step.CreatedAt.UTC().Format(time.RFC3339),
	)
	return err
}

func (s *Store) ListAgentRunSteps(runID string) ([]agentrun.Step, error) {
	rows, err := s.DB.Query(
		`SELECT id,agent_run_id,kind,summary,payload_json,created_at
		 FROM agent_run_steps WHERE agent_run_id=? ORDER BY created_at ASC, id ASC`, runID,
	)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	var out []agentrun.Step
	for rows.Next() {
		var step agentrun.Step
		var created string
		if err := rows.Scan(
			&step.ID, &step.AgentRunID, &step.Kind, &step.Summary, &step.PayloadJSON, &created,
		); err != nil {
			return nil, err
		}
		step.CreatedAt, _ = time.Parse(time.RFC3339, created)
		out = append(out, step)
	}
	return out, rows.Err()
}

func (s *Store) GetAgentRunRecord(id string) (agentrun.Record, error) {
	run, err := s.GetAgentRun(id)
	if err != nil {
		return agentrun.Record{}, err
	}
	steps, err := s.ListAgentRunSteps(id)
	if err != nil {
		return agentrun.Record{}, err
	}
	if steps == nil {
		steps = []agentrun.Step{}
	}
	return agentrun.Record{Run: run, Steps: steps}, nil
}

func (s *Store) ListAgentRuns(projectID string, limit int) ([]agentrun.Run, error) {
	if limit <= 0 || limit > 200 {
		limit = 50
	}
	rows, err := s.DB.Query(
		`SELECT id,project_id,task,agent_label,status,created_at,completed_at
		 FROM agent_runs WHERE project_id=? ORDER BY created_at DESC LIMIT ?`,
		projectID, limit,
	)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	var out []agentrun.Run
	for rows.Next() {
		var run agentrun.Run
		var created string
		var completed sql.NullString
		if err := rows.Scan(
			&run.ID, &run.ProjectID, &run.Task, &run.AgentLabel, &run.Status, &created, &completed,
		); err != nil {
			return nil, err
		}
		run.CreatedAt, _ = time.Parse(time.RFC3339, created)
		if completed.Valid && completed.String != "" {
			t, _ := time.Parse(time.RFC3339, completed.String)
			run.CompletedAt = &t
		}
		out = append(out, run)
	}
	return out, rows.Err()
}

// AppendAgentRunStepIfOpen loads the run, ensures it is open, and appends the step.
func (s *Store) AppendAgentRunStepIfOpen(runID string, build func(run agentrun.Run) (agentrun.Step, error)) error {
	run, err := s.GetAgentRun(runID)
	if err != nil {
		return err
	}
	if err := agentrun.CanAppend(run); err != nil {
		return err
	}
	step, err := build(run)
	if err != nil {
		return err
	}
	return s.AppendAgentRunStep(step)
}
