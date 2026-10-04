package mcp

import (
	"context"
	"strings"
	"time"

	mcpsdk "github.com/modelcontextprotocol/go-sdk/mcp"
	"github.com/polterware/polterware/sparky/backend/domains/memory"
)

func (s *Server) addResourcesAndPrompts() {
	s.sdk.AddResource(&mcpsdk.Resource{URI: "sparky://minds", Name: "minds", MIMEType: "text/plain", Description: "Mind tree."}, s.readMinds)
	s.sdk.AddResource(&mcpsdk.Resource{URI: "sparky://today", Name: "today", MIMEType: "text/plain", Description: "Now and today's plate."}, s.readToday)
	s.sdk.AddResourceTemplate(&mcpsdk.ResourceTemplate{URITemplate: "sparky://memory/{id}", Name: "memory", MIMEType: "text/plain", Description: "One memory."}, s.readMemory)
	s.sdk.AddPrompt(&mcpsdk.Prompt{Name: "plan_day", Description: "Plan today from Inbox and occurrences."}, promptText("List today's occurrences and Inbox, then propose a short plan. Use list_occurrences and list_memories."))
	s.sdk.AddPrompt(&mcpsdk.Prompt{Name: "weekly_review", Description: "Weekly review from get_review."}, promptText("Call get_review for this week. Summarize streak, completed, and focus minutes. Do not start a timer."))
	s.sdk.AddPrompt(&mcpsdk.Prompt{Name: "capture", Description: "Turn free text into create_memories arguments."}, promptText("Turn the user's notes into create_memories items. Use mind and tag names. Confirm schedule next occurrences."))
}

func promptText(text string) mcpsdk.PromptHandler {
	return func(context.Context, *mcpsdk.GetPromptRequest) (*mcpsdk.GetPromptResult, error) {
		return &mcpsdk.GetPromptResult{Messages: []*mcpsdk.PromptMessage{{Role: "user", Content: &mcpsdk.TextContent{Text: text}}}}, nil
	}
}

func (s *Server) readMinds(ctx context.Context, _ *mcpsdk.ReadResourceRequest) (*mcpsdk.ReadResourceResult, error) {
	rows, err := s.deps.Minds.List(ctx)
	if err != nil {
		return nil, err
	}
	text := mindTreeText(rows, nil)
	if text == "" {
		text = "0 minds"
	}
	return &mcpsdk.ReadResourceResult{Contents: []*mcpsdk.ResourceContents{{URI: "sparky://minds", MIMEType: "text/plain", Text: text}}}, nil
}

func (s *Server) readToday(ctx context.Context, _ *mcpsdk.ReadResourceRequest) (*mcpsdk.ReadResourceResult, error) {
	now := s.deps.now()
	text := now.Format("Mon 2006-01-02 15:04 MST")
	if s.deps.Schedules != nil {
		from := startOfDay(now)
		rows, err := s.deps.Schedules.ListOccurrences(ctx, from, from.Add(24*time.Hour), nil, true, DefaultLimit)
		if err == nil && len(rows) > 0 {
			titles := make([]string, 0, len(rows))
			for _, row := range rows {
				titles = append(titles, row.Title)
			}
			text += "\n" + strings.Join(titles, ", ")
		}
	}
	return &mcpsdk.ReadResourceResult{Contents: []*mcpsdk.ResourceContents{{URI: "sparky://today", MIMEType: "text/plain", Text: text}}}, nil
}

func (s *Server) readMemory(ctx context.Context, req *mcpsdk.ReadResourceRequest) (*mcpsdk.ReadResourceResult, error) {
	id := strings.TrimPrefix(req.Params.URI, "sparky://memory/")
	got, err := s.deps.Memories.Get(ctx, id, memory.Include{Note: true, Checklist: true})
	if err != nil {
		return nil, err
	}
	return &mcpsdk.ReadResourceResult{Contents: []*mcpsdk.ResourceContents{{URI: req.Params.URI, MIMEType: "text/plain", Text: got.Memory.Title}}}, nil
}

func startOfDay(now time.Time) time.Time {
	loc := now.Location()
	if loc == nil {
		loc = time.UTC
	}
	return time.Date(now.Year(), now.Month(), now.Day(), 0, 0, 0, 0, loc)
}
