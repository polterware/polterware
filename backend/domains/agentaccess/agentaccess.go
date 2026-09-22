package agentaccess

import (
	"crypto/rand"
	"encoding/hex"
	"time"
)

type ToolCall struct {
	ID            string
	ProjectID     string
	AgentRunID    string
	ToolName      string
	ArgumentsJSON string
	ResultSummary string
	OK            bool
	Source        string
	CreatedAt     time.Time
}

func NewCallID() string {
	var buf [12]byte
	_, _ = rand.Read(buf[:])
	return "acall-" + hex.EncodeToString(buf[:])
}

func SummarizeResult(text string, max int) string {
	if max <= 0 {
		max = 240
	}
	runes := []rune(text)
	if len(runes) <= max {
		return text
	}
	return string(runes[:max]) + "…"
}
