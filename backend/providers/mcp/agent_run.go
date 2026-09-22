package mcp

import (
	"fmt"
	"time"

	"github.com/polterware/polterware/backend/domains/agentrun"
)

func (s *Server) agentRunStart(args map[string]any) (string, error) {
	projectID := stringFrom(args["project_id"])
	task := stringFrom(args["task"])
	run, err := agentrun.Start(projectID, task, stringFrom(args["agent_label"]), time.Now().UTC())
	if err != nil {
		return "", err
	}
	if err := s.Store.CreateAgentRun(run); err != nil {
		return "", err
	}
	return marshalPretty(runJSON(run, nil))
}

func (s *Server) agentRunAppend(args map[string]any) (string, error) {
	runID := stringFrom(args["agent_run_id"])
	if runID == "" {
		return "", fmt.Errorf("agent_run_id is required")
	}
	kind, err := agentrun.NormalizeKind(stringFrom(args["kind"]))
	if err != nil {
		return "", err
	}
	now := time.Now().UTC()
	err = s.Store.AppendAgentRunStepIfOpen(runID, func(run agentrun.Run) (agentrun.Step, error) {
		switch kind {
		case agentrun.KindChange:
			return agentrun.NewChangeStep(
				run.ID,
				stringFrom(args["summary"]),
				stringFrom(args["rationale"]),
				stringSliceFrom(args["paths"]),
				now,
			)
		case agentrun.KindEvidence:
			return agentrun.NewEvidenceStep(run.ID, stringSliceFrom(args["evidence_ids"]), now)
		case agentrun.KindContext, agentrun.KindDrift:
			return agentrun.Step{}, fmt.Errorf(
				"%s steps are recorded via context.get / drift.check with agent_run_id", kind,
			)
		default:
			return agentrun.Step{}, agentrun.ErrInvalidKind
		}
	})
	if err != nil {
		return "", err
	}
	record, err := s.Store.GetAgentRunRecord(runID)
	if err != nil {
		return "", err
	}
	return marshalPretty(recordJSON(record))
}

func (s *Server) agentRunComplete(args map[string]any) (string, error) {
	runID := stringFrom(args["agent_run_id"])
	if runID == "" {
		return "", fmt.Errorf("agent_run_id is required")
	}
	run, err := s.Store.GetAgentRun(runID)
	if err != nil {
		return "", err
	}
	done, err := agentrun.Complete(run, time.Now().UTC())
	if err != nil {
		return "", err
	}
	if err := s.Store.UpdateAgentRun(done); err != nil {
		return "", err
	}
	record, err := s.Store.GetAgentRunRecord(done.ID)
	if err != nil {
		return "", err
	}
	return marshalPretty(recordJSON(record))
}

func (s *Server) agentRunGet(args map[string]any) (string, error) {
	runID := stringFrom(args["id"])
	if runID == "" {
		runID = stringFrom(args["agent_run_id"])
	}
	if runID == "" {
		return "", fmt.Errorf("id is required")
	}
	record, err := s.Store.GetAgentRunRecord(runID)
	if err != nil {
		return "", err
	}
	return marshalPretty(recordJSON(record))
}

func (s *Server) agentRunList(args map[string]any) (string, error) {
	projectID := stringFrom(args["project_id"])
	if projectID == "" {
		return "", fmt.Errorf("project_id is required")
	}
	runs, err := s.Store.ListAgentRuns(projectID, intFrom(args["limit"], 50))
	if err != nil {
		return "", err
	}
	out := make([]map[string]any, 0, len(runs))
	for _, run := range runs {
		out = append(out, runJSON(run, nil))
	}
	return marshalPretty(map[string]any{
		"project_id": projectID,
		"runs":       out,
	})
}

func runJSON(run agentrun.Run, steps []agentrun.Step) map[string]any {
	out := map[string]any{
		"id":          run.ID,
		"project_id":  run.ProjectID,
		"task":        run.Task,
		"agent_label": run.AgentLabel,
		"status":      run.Status,
		"created_at":  run.CreatedAt.UTC().Format(time.RFC3339),
	}
	if run.CompletedAt != nil {
		out["completed_at"] = run.CompletedAt.UTC().Format(time.RFC3339)
	}
	if steps != nil {
		stepOut := make([]map[string]any, 0, len(steps))
		for _, step := range steps {
			stepOut = append(stepOut, stepJSON(step))
		}
		out["steps"] = stepOut
	}
	return out
}

func recordJSON(record agentrun.Record) map[string]any {
	return runJSON(record.Run, record.Steps)
}

func stepJSON(step agentrun.Step) map[string]any {
	return map[string]any{
		"id":           step.ID,
		"agent_run_id": step.AgentRunID,
		"kind":         step.Kind,
		"summary":      step.Summary,
		"payload":      jsonRawOrObject(step.PayloadJSON),
		"created_at":   step.CreatedAt.UTC().Format(time.RFC3339),
	}
}
