package structure

import "strings"

// ComponentKinds are what a component is, in palette order. The canvas uses the
// same list for its semantic components. Only components carry a kind.
var ComponentKinds = []string{
	"frontend", "mobile", "desktop", "cli",
	"service", "api", "gateway", "worker", "function", "scheduler",
	"database", "cache", "queue", "stream", "storage", "search", "warehouse",
	"auth", "cdn", "cluster", "monitoring",
	"external", "llm", "payment", "email",
	"user", "team",
	"agent", "mcp", "vector",
	"secrets", "firewall", "kms",
	"logs", "tracing", "alerting",
	"pipeline", "ci", "webhook",
	"config", "spec", "dataset",
}

// ValidComponentKind reports whether kind is a component kind. Empty is no kind.
// ListValues splits a comma list (`service, database`) into trimmed values, dropping
// empty ones and repeats. A component may have several kinds and technologies.
func ListValues(text string) []string {
	var out []string
	seen := map[string]bool{}
	for _, part := range strings.Split(text, ",") {
		value := strings.TrimSpace(part)
		if value == "" || seen[strings.ToLower(value)] {
			continue
		}
		seen[strings.ToLower(value)] = true
		out = append(out, value)
	}
	return out
}

// JoinValues writes a list the way the store keeps it: `a, b`.
func JoinValues(values []string) string {
	return strings.Join(values, ", ")
}

func ValidComponentKind(kind string) bool {
	if kind == "" {
		return true
	}
	for _, item := range ComponentKinds {
		if item == kind {
			return true
		}
	}
	return false
}
