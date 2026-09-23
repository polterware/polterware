package mcp

import (
	"github.com/polterware/polterware/backend/compositions/wedge"
	"github.com/polterware/polterware/backend/domains/agentrun"
)

type auditedEnvelope struct {
	Summary    string
	Rationale  string
	AgentRunID string
	ProjectID  string
	Paths      []string
}

func parseAuditedEnvelope(args map[string]any) (auditedEnvelope, error) {
	env := auditedEnvelope{
		Summary:    stringFrom(args["summary"]),
		Rationale:  stringFrom(args["rationale"]),
		AgentRunID: stringFrom(args["agent_run_id"]),
		ProjectID:  stringFrom(args["project_id"]),
		Paths:      stringSliceFrom(args["paths"]),
	}
	if err := agentrun.ValidateAuditedWrite(env.Summary, env.Rationale, env.AgentRunID); err != nil {
		return auditedEnvelope{}, err
	}
	if env.ProjectID == "" {
		return auditedEnvelope{}, agentrun.ErrEmptyProject
	}
	return env, nil
}

// withAuditedWrite validates the open AgentRun, runs the mutation, then appends a change step.
func (s *Server) withAuditedWrite(
	env auditedEnvelope,
	tool string,
	mutate func(run agentrun.Run) ([]string, any, error),
) (string, error) {
	result, err := wedge.ExecuteAuditedWrite(
		s.Store,
		wedge.ChangeEnvelope{
			Summary:    env.Summary,
			Rationale:  env.Rationale,
			AgentRunID: env.AgentRunID,
			ProjectID:  env.ProjectID,
			Paths:      env.Paths,
		},
		agentrun.ActorKindAgent,
		tool,
		mutate,
	)
	if err != nil {
		return "", err
	}
	return marshalPretty(map[string]any{
		"ok":           true,
		"tool":         tool,
		"agent_run_id": result.AgentRunID,
		"subject_ids":  result.SubjectIDs,
		"result":       result.Result,
	})
}
