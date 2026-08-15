package knowledge

import "testing"

func TestKnowledgeKinds(t *testing.T) {
	if _, err := NewItem("doc", "project", "Title", "Body", "decision", nil); err == nil {
		t.Fatal("expected unsupported kind")
	}
	if _, err := NewItem("runbook", "project", "Title", "Body", "runbook", nil); err != nil {
		t.Fatal(err)
	}
}
