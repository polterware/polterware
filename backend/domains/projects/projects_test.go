package projects

import "testing"

func TestNewProjectRequiresLocalPath(t *testing.T) {
	if _, err := NewProject("project", "Project", ""); err == nil {
		t.Fatal("expected root path validation")
	}
}
