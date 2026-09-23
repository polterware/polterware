package mcp

import (
	"github.com/polterware/polterware/backend/compositions/wedge"
)

func (s *Server) driftCheck(args map[string]any) (string, error) {
	result, err := wedge.CheckDrift(s.Store, wedge.DriftInput{
		ProjectID:  stringFrom(args["project_id"]),
		EntityID:   stringFrom(args["entity_id"]),
		AgentRunID: stringFrom(args["agent_run_id"]),
	})
	if err != nil {
		return "", err
	}
	return marshalPretty(result)
}
