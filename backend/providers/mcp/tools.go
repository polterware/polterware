package mcp

import "fmt"

// Stable wedge MCP read tools (B4).
const (
	ToolContextGet  = "context.get"
	ToolEntityGet   = "entity.get"
	ToolEvidenceGet = "evidence.get"
	ToolDriftCheck  = "drift.check"
)

func toolDefinitions() []map[string]any {
	return []map[string]any{
		{
			"name":        ToolContextGet,
			"description": "Compile a bounded, provenance-aware project context package with machine reasons and evidence refs.",
			"inputSchema": map[string]any{
				"type": "object",
				"properties": map[string]any{
					"project_id":   map[string]any{"type": "string"},
					"query":        map[string]any{"type": "string"},
					"entity_id":    map[string]any{"type": "string"},
					"token_budget": map[string]any{"type": "integer"},
				},
				"required": []string{"project_id"},
			},
		},
		{
			"name":        ToolEntityGet,
			"description": "Fetch one PPM entity by id.",
			"inputSchema": map[string]any{
				"type": "object",
				"properties": map[string]any{
					"id": map[string]any{"type": "string"},
				},
				"required": []string{"id"},
			},
		},
		{
			"name":        ToolEvidenceGet,
			"description": "Fetch one PPM evidence record by id.",
			"inputSchema": map[string]any{
				"type": "object",
				"properties": map[string]any{
					"id": map[string]any{"type": "string"},
				},
				"required": []string{"id"},
			},
		},
		{
			"name":        ToolDriftCheck,
			"description": "Check project model drift. Returns findings when the narrow Drift Checker (B5) is available; empty findings until then.",
			"inputSchema": map[string]any{
				"type": "object",
				"properties": map[string]any{
					"project_id": map[string]any{"type": "string"},
					"entity_id":  map[string]any{"type": "string"},
				},
				"required": []string{"project_id"},
			},
		},
	}
}

func (s *Server) invokeTool(name string, args map[string]any) (string, error) {
	switch name {
	case ToolContextGet:
		return s.contextGet(args)
	case ToolEntityGet:
		return s.entityGet(args)
	case ToolEvidenceGet:
		return s.evidenceGet(args)
	case ToolDriftCheck:
		return s.driftCheck(args)
	default:
		return "", fmt.Errorf("unknown tool %q", name)
	}
}
