package mcp

import "fmt"

// Stable wedge MCP tools (B4 reads + B6 AgentRun audit).
const (
	ToolContextGet       = "context.get"
	ToolEntityGet        = "entity.get"
	ToolEvidenceGet      = "evidence.get"
	ToolDriftCheck       = "drift.check"
	ToolAgentRunStart    = "agent_run.start"
	ToolAgentRunAppend   = "agent_run.append"
	ToolAgentRunComplete = "agent_run.complete"
	ToolAgentRunGet      = "agent_run.get"
	ToolAgentRunList     = "agent_run.list"
)

func toolDefinitions() []map[string]any {
	return []map[string]any{
		{
			"name":        ToolContextGet,
			"description": "Compile a bounded, provenance-aware project context package with machine reasons and evidence refs. Pass agent_run_id to append a context step on an open AgentRun.",
			"inputSchema": map[string]any{
				"type": "object",
				"properties": map[string]any{
					"project_id":    map[string]any{"type": "string"},
					"query":         map[string]any{"type": "string"},
					"entity_id":     map[string]any{"type": "string"},
					"token_budget":  map[string]any{"type": "integer"},
					"agent_run_id":  map[string]any{"type": "string"},
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
			"description": "Run the narrow Drift Checker: import dependency diffs, edges under accepted ADRs without evidence, and declared-versus-observed fact conflicts. Pass agent_run_id to append a drift step on an open AgentRun.",
			"inputSchema": map[string]any{
				"type": "object",
				"properties": map[string]any{
					"project_id":   map[string]any{"type": "string"},
					"entity_id":    map[string]any{"type": "string"},
					"agent_run_id": map[string]any{"type": "string"},
				},
				"required": []string{"project_id"},
			},
		},
		{
			"name":        ToolAgentRunStart,
			"description": "Start a minimal AgentRun audit trail (context → task → change → evidence → drift).",
			"inputSchema": map[string]any{
				"type": "object",
				"properties": map[string]any{
					"project_id":   map[string]any{"type": "string"},
					"task":         map[string]any{"type": "string"},
					"agent_label":  map[string]any{"type": "string"},
				},
				"required": []string{"project_id", "task"},
			},
		},
		{
			"name":        ToolAgentRunAppend,
			"description": "Append a change or evidence step to an open AgentRun. Change requires summary and rationale. Context and drift steps come from context.get / drift.check with agent_run_id.",
			"inputSchema": map[string]any{
				"type": "object",
				"properties": map[string]any{
					"agent_run_id":  map[string]any{"type": "string"},
					"kind":          map[string]any{"type": "string", "enum": []string{"change", "evidence"}},
					"summary":       map[string]any{"type": "string"},
					"rationale":     map[string]any{"type": "string"},
					"paths":         map[string]any{"type": "array", "items": map[string]any{"type": "string"}},
					"evidence_ids":  map[string]any{"type": "array", "items": map[string]any{"type": "string"}},
				},
				"required": []string{"agent_run_id", "kind"},
			},
		},
		{
			"name":        ToolAgentRunComplete,
			"description": "Mark an open AgentRun as completed.",
			"inputSchema": map[string]any{
				"type": "object",
				"properties": map[string]any{
					"agent_run_id": map[string]any{"type": "string"},
				},
				"required": []string{"agent_run_id"},
			},
		},
		{
			"name":        ToolAgentRunGet,
			"description": "Fetch one AgentRun with ordered steps for audit reconstruction.",
			"inputSchema": map[string]any{
				"type": "object",
				"properties": map[string]any{
					"id":           map[string]any{"type": "string"},
					"agent_run_id": map[string]any{"type": "string"},
				},
				"required": []string{"id"},
			},
		},
		{
			"name":        ToolAgentRunList,
			"description": "List AgentRuns for a project (newest first).",
			"inputSchema": map[string]any{
				"type": "object",
				"properties": map[string]any{
					"project_id": map[string]any{"type": "string"},
					"limit":      map[string]any{"type": "integer"},
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
	case ToolAgentRunStart:
		return s.agentRunStart(args)
	case ToolAgentRunAppend:
		return s.agentRunAppend(args)
	case ToolAgentRunComplete:
		return s.agentRunComplete(args)
	case ToolAgentRunGet:
		return s.agentRunGet(args)
	case ToolAgentRunList:
		return s.agentRunList(args)
	default:
		return "", fmt.Errorf("unknown tool %q", name)
	}
}
