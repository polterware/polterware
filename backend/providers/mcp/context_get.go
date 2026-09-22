package mcp

import (
	"fmt"
	"time"

	"github.com/polterware/polterware/backend/domains/agentrun"
	contextpkg "github.com/polterware/polterware/backend/domains/context"
)

func (s *Server) contextGet(args map[string]any) (string, error) {
	projectID := stringFrom(args["project_id"])
	if projectID == "" {
		return "", fmt.Errorf("project_id is required")
	}
	_, _ = s.Store.SyncProjectModel(projectID)
	pkg, err := s.Store.CompileContext(contextpkg.Request{
		ProjectID:   projectID,
		Query:       stringFrom(args["query"]),
		EntityID:    stringFrom(args["entity_id"]),
		TokenBudget: intFrom(args["token_budget"], contextpkg.DefaultTokenBudget),
	})
	if err != nil {
		return "", err
	}
	if runID := stringFrom(args["agent_run_id"]); runID != "" {
		now := time.Now().UTC()
		if err := s.Store.AppendAgentRunStepIfOpen(runID, func(run agentrun.Run) (agentrun.Step, error) {
			return agentrun.NewContextStep(run.ID, pkg, now)
		}); err != nil {
			return "", err
		}
	}
	return marshalPretty(pkg)
}
