package knowledge

import "testing"

func TestKnowledgeKinds(t *testing.T) {
	if _, err := NewItem("doc", "project", "Title", "Body", "decision", nil); err == nil {
		t.Fatal("expected unsupported kind")
	}
	if _, err := NewItem("runbook", "project", "Title", "Body", "runbook", nil); err != nil {
		t.Fatal(err)
	}
	for _, kind := range []string{"adr", "source"} {
		if _, err := NewItem(kind+"-1", "project", "Title", "Body", kind, nil); err != nil {
			t.Fatalf("expected kind %s to be valid: %v", kind, err)
		}
	}
	// The five-document Work model moved to the Work tables (ADR 0006).
	for _, kind := range []string{"spec", "plan", "tasks", "clarify", "checklist", "constitution"} {
		if _, err := NewItem(kind+"-1", "project", "Title", "Body", kind, nil); err == nil {
			t.Fatalf("expected kind %s to be rejected", kind)
		}
	}
}
