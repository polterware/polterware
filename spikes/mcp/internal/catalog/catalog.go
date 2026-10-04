// Package catalog holds the spike tool copy shared by both MCP servers.
// Descriptions stay inside the token budgets in plans/sparky-next/04-mcp.md.
package catalog

const (
	ServerName    = "sparky"
	ServerVersion = "0.0.0-spike"

	NameTime   = "get_current_time"
	NameList   = "list_memories"
	NameCreate = "create_memories"

	DescTime   = "Return now as RFC3339, the timezone name, the weekday, and the ISO week number."
	DescList   = "List memory summaries (id, title, status, snippet, rev) with total and nextCursor. Default limit is 20 and the maximum is 100."
	DescCreate = "Create up to 50 memories. Each item needs a title and may include a note. Returns one ok or error result per item."

	DescLimit  = "Page size from 1 to 100. Defaults to 20."
	DescCursor = "Opaque cursor from the previous page."
	DescItems  = "Memories to create, at most 50."
	DescTitle  = "Memory title."
	DescNote   = "Optional Markdown note."

	ResourceTodayURI  = "sparky://today"
	ResourceTodayName = "today"
	DescToday         = "How many memories are stored right now."

	PromptCapture     = "capture"
	DescPromptCapture = "Turn free text into create_memories arguments."
	PromptCaptureText = "Turn the user's text into create_memories arguments. Call create_memories with one item per thought. Do not invent due dates."

	MaxLimit = 100
	MaxBatch = 50
)

// ToolNames is the stable tool order advertised by both servers.
func ToolNames() []string {
	return []string{NameTime, NameList, NameCreate}
}
