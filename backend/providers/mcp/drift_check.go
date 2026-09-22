package mcp

import "fmt"

// DriftCheckResult is the stable B4 stub shape. B5 fills findings.
type DriftCheckResult struct {
	ProjectID string `json:"project_id"`
	EntityID  string `json:"entity_id,omitempty"`
	Status    string `json:"status"`
	Checker   string `json:"checker"`
	Findings  []any  `json:"findings"`
}

func (s *Server) driftCheck(args map[string]any) (string, error) {
	projectID := stringFrom(args["project_id"])
	if projectID == "" {
		return "", fmt.Errorf("project_id is required")
	}
	result := DriftCheckResult{
		ProjectID: projectID,
		EntityID:  stringFrom(args["entity_id"]),
		Status:    "ok",
		Checker:   "none",
		Findings:  []any{},
	}
	return marshalPretty(result)
}
