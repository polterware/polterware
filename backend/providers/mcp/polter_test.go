package mcp

import (
	"strings"
	"testing"

	"github.com/polterware/polterware/backend/domains/agentrun"
	"github.com/polterware/polterware/backend/domains/polter"
	"github.com/polterware/polterware/backend/providers/seed"
)

const polterRoadmap = `---
polter: 1
summary: Plan git import
rationale: Slice 5 needs a card before coding
---

## git-import: Git metadata import
stage: plan

- [ ] T1 Upsert commit evidence
? Include merge commits? [blocking] (yes | no)
`

func TestPolterFormatReturnsSpec(t *testing.T) {
	server := testServer(t)
	text, err := server.callTool(ToolPolterFormat, nil)
	if err != nil {
		t.Fatal(err)
	}
	if text != polter.Spec {
		t.Fatalf("polter.format returned %q", text)
	}
}

func TestPolterApplyUsesOneSessionRun(t *testing.T) {
	server := testServer(t)
	server.session.Label = "claude-code"

	plan, err := server.callTool(ToolPolterApply, map[string]any{
		"project_id": seed.ProjectID, "text": polterRoadmap, "dry_run": true,
	})
	if err != nil {
		t.Fatal(err)
	}
	if !strings.Contains(plan, "+ card git-import") {
		t.Fatalf("dry run plan: %s", plan)
	}
	runs, _ := server.Store.ListAgentRuns(seed.ProjectID, 50)
	if len(runs) != 0 {
		t.Fatalf("dry run opened %d runs", len(runs))
	}

	if _, err := server.callTool(ToolPolterApply, map[string]any{"project_id": seed.ProjectID, "text": polterRoadmap}); err != nil {
		t.Fatal(err)
	}
	stage := "---\npolter: 1\nsummary: Start build\nrationale: Plan approved\n---\n\n## git-import\nstage: build\n"
	if _, err := server.callTool(ToolPolterApply, map[string]any{"project_id": seed.ProjectID, "text": stage}); err != nil {
		t.Fatal(err)
	}
	again, err := server.callTool(ToolPolterApply, map[string]any{"project_id": seed.ProjectID, "text": stage})
	if err != nil {
		t.Fatal(err)
	}
	if !strings.Contains(again, "no changes") {
		t.Fatalf("reapply: %s", again)
	}

	runs, _ = server.Store.ListAgentRuns(seed.ProjectID, 50)
	if len(runs) != 1 {
		t.Fatalf("expected one session run, got %d", len(runs))
	}
	if runs[0].Status != agentrun.StatusOpen || runs[0].AgentLabel != "claude-code" {
		t.Fatalf("session run: %+v", runs[0])
	}
	record, err := server.Store.GetAgentRunRecord(runs[0].ID)
	if err != nil {
		t.Fatal(err)
	}
	if len(record.Steps) != 2 {
		t.Fatalf("expected 2 change steps, got %d", len(record.Steps))
	}

	card, err := server.callTool(ToolPolterQuery, map[string]any{"project_id": seed.ProjectID, "query": "card git-import"})
	if err != nil {
		t.Fatal(err)
	}
	if !strings.Contains(card, "stage: build") {
		t.Fatalf("query card: %s", card)
	}

	server.session.Close(server.Store)
	run, _ := server.Store.GetAgentRun(runs[0].ID)
	if run.Status != agentrun.StatusCompleted {
		t.Fatalf("session run not completed: %s", run.Status)
	}
}

func TestPolterApplyRejectsInvalidTextWithoutRun(t *testing.T) {
	server := testServer(t)
	bad := "---\npolter: 1\nsummary: x\nrationale: y\n---\n\n- [ ] orphan task\n"
	_, err := server.callTool(ToolPolterApply, map[string]any{"project_id": seed.ProjectID, "text": bad})
	if err == nil || !strings.Contains(err.Error(), "line 7") {
		t.Fatalf("expected line diagnostic, got %v", err)
	}
	missing := strings.Replace(polterRoadmap, "summary: Plan git import\n", "", 1)
	if _, err := server.callTool(ToolPolterApply, map[string]any{"project_id": seed.ProjectID, "text": missing}); err == nil {
		t.Fatal("expected missing summary error")
	}
	runs, _ := server.Store.ListAgentRuns(seed.ProjectID, 50)
	if len(runs) != 0 {
		t.Fatalf("invalid writes opened %d runs", len(runs))
	}
}
