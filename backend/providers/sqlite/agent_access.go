package sqlite

import (
	"time"

	"github.com/polterware/polterware/backend/domains/agentaccess"
)

func (s *Store) AppendAgentToolCall(call agentaccess.ToolCall) error {
	if call.ID == "" {
		call.ID = agentaccess.NewCallID()
	}
	if call.CreatedAt.IsZero() {
		call.CreatedAt = time.Now().UTC()
	}
	if call.Source == "" {
		call.Source = "mcp"
	}
	if call.ArgumentsJSON == "" {
		call.ArgumentsJSON = "{}"
	}
	ok := 0
	if call.OK {
		ok = 1
	}
	_, err := s.DB.Exec(
		`INSERT INTO agent_tool_calls(id,project_id,tool_name,arguments_json,result_summary,ok,source,created_at)
		 VALUES(?,?,?,?,?,?,?,?)`,
		call.ID, call.ProjectID, call.ToolName, call.ArgumentsJSON, call.ResultSummary, ok, call.Source,
		call.CreatedAt.Format(time.RFC3339),
	)
	return err
}

func (s *Store) ListAgentToolCalls(limit int) ([]agentaccess.ToolCall, error) {
	if limit <= 0 || limit > 200 {
		limit = 50
	}
	rows, err := s.DB.Query(
		`SELECT id,project_id,tool_name,arguments_json,result_summary,ok,source,created_at
		 FROM agent_tool_calls ORDER BY created_at DESC LIMIT ?`, limit,
	)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	var out []agentaccess.ToolCall
	for rows.Next() {
		var call agentaccess.ToolCall
		var okFlag int
		var created string
		if err := rows.Scan(
			&call.ID, &call.ProjectID, &call.ToolName, &call.ArgumentsJSON, &call.ResultSummary,
			&okFlag, &call.Source, &created,
		); err != nil {
			return nil, err
		}
		call.OK = okFlag != 0
		call.CreatedAt, _ = time.Parse(time.RFC3339, created)
		out = append(out, call)
	}
	return out, rows.Err()
}
