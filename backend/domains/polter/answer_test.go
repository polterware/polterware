package polter

import (
	"strings"
	"testing"

	"github.com/polterware/polterware/backend/domains/work"
)

func answerSnapshot() work.Snapshot {
	return work.Snapshot{ProjectID: "p1", Cards: []work.Card{{
		ID: "c1", Slug: "git-import", Title: "Git import", Stage: work.StagePlan, Status: work.CardActive,
		Questions: []work.Question{
			{ID: "q1", CardID: "c1", Text: "Include merge commits?", Blocking: true, Options: []string{"yes", "no", "first-parent"}, State: work.QuestionOpen},
			{ID: "q2", CardID: "c1", Text: "How many commits back?", State: work.QuestionAnswered, Answer: "20"},
		},
	}}}
}

func TestAnswerDecode(t *testing.T) {
	doc, diags := Decode("---\npolter: 1\n---\n## git-import\n? Include merge commits? [blocking] (yes | no) -> first-parent\n? Bare question\n")
	if len(diags) != 0 {
		t.Fatalf("diags = %+v", diags)
	}
	q := doc.Cards[0].Questions[0]
	if q.Text != "Include merge commits?" || !q.Blocking || len(q.Options) != 2 || q.Answer != "first-parent" {
		t.Fatalf("question = %+v", q)
	}
	if doc.Cards[0].Questions[1].Answer != "" {
		t.Fatalf("bare question has no answer: %+v", doc.Cards[0].Questions[1])
	}

	_, diags = Decode("---\npolter: 1\n---\n## git-import\n? Empty ->\n? Both -> yes [removed]\n")
	if len(diags) != 1 || !strings.Contains(diags[0].Message, "answer is empty") {
		t.Fatalf("empty answer: %+v", diags)
	}
	_, diags = Decode("---\npolter: 1\n---\n## git-import\n? Both [removed] -> yes\n")
	if len(diags) != 1 || !strings.Contains(diags[0].Message, "answered and removed") {
		t.Fatalf("answered and removed: %+v", diags)
	}
}

func TestAnswerPlan(t *testing.T) {
	snapshot := answerSnapshot()

	ops, diags := Plan(mustDecode(t, "---\npolter: 1\n---\n## git-import\n? Include merge commits? [blocking] -> first-parent\n"), snapshot)
	if HasErrors(diags) || len(diags) != 0 || len(ops) != 1 {
		t.Fatalf("answer: ops=%+v diags=%+v", ops, diags)
	}
	if ops[0].Kind != work.OpUpdate || ops[0].Fields["state"] != work.QuestionAnswered || ops[0].Fields["answer"] != "first-parent" {
		t.Fatalf("answer op = %+v", ops[0])
	}
	if _, changed := ops[0].Fields["blocking"]; changed {
		t.Fatalf("repeating [blocking] must keep the flag: %+v", ops[0].Fields)
	}

	// An answer outside the options is a hint, not an error.
	ops, diags = Plan(mustDecode(t, "---\npolter: 1\n---\n## git-import\n? Include merge commits? [blocking] -> maybe\n"), snapshot)
	if HasErrors(diags) || len(ops) != 1 || len(diags) != 1 || diags[0].Severity != SeverityHint || !strings.Contains(diags[0].Message, "not one of the options") {
		t.Fatalf("free answer: ops=%+v diags=%+v", ops, diags)
	}

	// The same answer again is a no-op; a bare line never reopens it.
	for _, line := range []string{"? How many commits back? -> 20", "? How many commits back?"} {
		ops, diags = Plan(mustDecode(t, "---\npolter: 1\n---\n## git-import\n"+line+"\n"), snapshot)
		if HasErrors(diags) || len(ops) != 0 {
			t.Fatalf("%q: ops=%+v diags=%+v", line, ops, diags)
		}
	}

	// A different answer updates it.
	ops, _ = Plan(mustDecode(t, "---\npolter: 1\n---\n## git-import\n? How many commits back? -> 50\n"), snapshot)
	if len(ops) != 1 || ops[0].Fields["answer"] != "50" || ops[0].Before["answer"] != "20" {
		t.Fatalf("re-answer: %+v", ops)
	}
	if _, ok := ops[0].Fields["state"]; ok {
		t.Fatalf("re-answer keeps the state: %+v", ops[0].Fields)
	}

	// A new question written already answered is created answered.
	ops, _ = Plan(mustDecode(t, "---\npolter: 1\n---\n## git-import\n? Keep tags? -> no\n"), snapshot)
	if len(ops) != 1 || ops[0].Kind != work.OpCreate || ops[0].Fields["state"] != work.QuestionAnswered || ops[0].Fields["answer"] != "no" {
		t.Fatalf("new answered question: %+v", ops)
	}
}

func TestAnswerRoundTrip(t *testing.T) {
	snapshot := answerSnapshot()
	text := EncodeCard(snapshot.Cards[0])
	if !strings.Contains(text, "? How many commits back? -> 20\n") {
		t.Fatalf("encode should keep answered questions:\n%s", text)
	}
	ops, diags := Plan(mustDecode(t, text), snapshot)
	if HasErrors(diags) || len(ops) != 0 {
		t.Fatalf("re-applying a read must be a no-op: ops=%+v diags=%+v", ops, diags)
	}
}
