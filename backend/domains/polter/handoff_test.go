package polter

import (
	"strings"
	"testing"

	"github.com/polterware/polterware/backend/domains/work"
)

func TestHandoffHintsAndStage(t *testing.T) {
	doc := mustDecode(t, `---
polter: 1
---
## git-import: Git import
handoff:
branch: feat/other
touched: docs/spec.md, missing.go
`)
	diags := HandoffHints(doc, GitView{Available: true, Branch: "feat/git-import", Files: []string{"docs/spec.md", "backend/import.go"}})
	if HasErrors(diags) {
		t.Fatalf("mismatches must be hints: %+v", diags)
	}
	if !hasDiag(diags, 5, `does not match git branch "feat/git-import"`) {
		t.Errorf("missing branch hint: %+v", diags)
	}
	if !hasDiag(diags, 5, "not declared backend/import.go") || !hasDiag(diags, 5, "not in the diff missing.go") {
		t.Errorf("missing touched hint: %+v", diags)
	}
	if hints := HandoffHints(doc, GitView{}); len(hints) != 0 {
		t.Fatalf("no repository should not hint: %+v", hints)
	}
	if DisplayStage(work.StageDelivered, false) != work.StageVerify {
		t.Fatal("unconfirmed delivered must read as verify")
	}
	if DisplayStage(work.StageDelivered, true) != work.StageDelivered {
		t.Fatal("git-confirmed delivered stays delivered")
	}
	if DisplayStage(work.StageBuild, true) != work.StageBuild {
		t.Fatal("git must not promote a card to delivered")
	}
}

func TestParseHandoffNote(t *testing.T) {
	note := ParseHandoffNote("still wiring the importer")
	if note.InProgress != "still wiring the importer" || note.Goal != "" {
		t.Fatalf("free note = %+v", note)
	}
	note = ParseHandoffNote("Goal: import commits\nDone: parser\nIn progress: status\nNext: docs\nTouched: a.go, b.go\nOpen question: include merges? [blocking]\nBranch: other\n")
	if note.Goal != "import commits" || note.Done != "parser" || note.InProgress != "status" || note.Next != "docs" {
		t.Fatalf("note = %+v", note)
	}
	if len(note.Touched) != 2 || note.Branch != "other" || len(note.Questions) != 1 || !note.Questions[0].Blocking {
		t.Fatalf("note fields = %+v", note)
	}
	status := EncodeStatus(work.Card{
		Slug: "git-import", Stage: work.StageBuild,
		Questions: []work.Question{
			{Text: "later?", State: work.QuestionOpen},
			{Text: "include merges?", Blocking: true, State: work.QuestionOpen},
		},
	}, "feat/git-import", work.StageVerify, &work.Handoff{Done: "parser", InProgress: "status", Next: "docs"})
	if !strings.Contains(status, "git-import  verify  branch feat/git-import") || !strings.Contains(status, "Done: parser") {
		t.Fatalf("status = %q", status)
	}
	blocking := strings.Index(status, "include merges? [blocking]")
	later := strings.Index(status, "later?")
	if blocking < 0 || later < 0 || blocking > later {
		t.Fatalf("blocking question should come first:\n%s", status)
	}
}
