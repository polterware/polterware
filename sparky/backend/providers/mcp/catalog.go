package mcp

import (
	"embed"
	"encoding/json"
	"fmt"
	"strings"
)

//go:embed schemas/*.json
var contractFiles embed.FS

// Tool is the compact public descriptor, without SDK-inferred output schemas.
// InputSchema is copied directly from the frozen contract.
type Tool struct {
	Name        string          `json:"name"`
	Description string          `json:"description"`
	InputSchema json.RawMessage `json:"inputSchema"`
}

// Catalog reads the 18 frozen contracts in deterministic order. Only display
// descriptions are shortened; input validation is not changed.
func Catalog() ([]Tool, error) {
	entries, err := contractFiles.ReadDir("schemas")
	if err != nil {
		return nil, fmt.Errorf("read MCP contracts: %w", err)
	}
	tools := make([]Tool, 0, 18)
	for _, entry := range entries {
		if strings.HasSuffix(entry.Name(), ".schema.json") {
			continue
		}
		data, err := contractFiles.ReadFile("schemas/" + entry.Name())
		if err != nil {
			return nil, fmt.Errorf("read MCP tool: %w", err)
		}
		var tool Tool
		if err := json.Unmarshal(data, &tool); err != nil {
			return nil, fmt.Errorf("decode MCP tool: %w", err)
		}
		switch tool.Name {
		case "list_memories":
			tool.Description = "Filter memory summaries."
		case "get_current_time":
			tool.Description = "Current time and zone."
		case "create_mind":
			tool.Description = "Create a mind under parent (name or id)."
		case "get_memory":
			tool.Description = "Get a memory; default: note,checklist,schedule."
		case "list_occurrences":
			tool.Description = "Occurrences, completion, overdue."
		case "update_mind":
			tool.Description = "Patch by id/name; null clears parent/color/icon."
		case "get_review":
			tool.Description = "Streak, counts, focus minutes."
		case "delete_mind":
			tool.Description = "Delete mind tree; memories to Inbox. confirm."
		case "add_attachments":
			tool.Description = "Add links or files. files needs allowFileAttach."
		case "set_memory_status":
			tool.Description = "Set active or completed. occurrence for one repeat."
		case "update_memories":
			tool.Description = "Patch up to 50. null clears. tags replace; add/removeTags."
		case "create_memories":
			tool.Description = "Create up to 50 memories; per-item ok or error."
		case "delete_memories":
			tool.Description = "Delete memories; confirm true."
		case "list_tags":
			tool.Description = "Tags, colors, counts."
		case "search_memories":
			tool.Description = "Ranked full-text."
		case "toggle_check_item":
			tool.Description = "Toggle checklist item by id or title."
		}
		tools = append(tools, tool)
	}
	return tools, nil
}
