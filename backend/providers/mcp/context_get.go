package mcp

import (
	"fmt"

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
	return marshalPretty(pkg)
}
